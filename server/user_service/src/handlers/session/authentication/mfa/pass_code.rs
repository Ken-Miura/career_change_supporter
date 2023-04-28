// Copyright 2023 Ken Miura

use std::time::Duration;

use async_fred_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::{async_trait, extract::State, http::StatusCode, Json};
use axum_extra::extract::SignedCookieJar;
use chrono::{DateTime, FixedOffset, Utc};
use common::{
    util::validator::pass_code_validator::validate_pass_code, ApiError, ErrResp, RespResult,
    JAPANESE_TIME_ZONE,
};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::handlers::session::authentication::mfa::get_session_by_session_id;
use crate::handlers::session::authentication::mfa::USER_TOTP_ISSUER;
use crate::handlers::session::authentication::user_operation::FindUserInfoOperationImpl;
use crate::handlers::session::authentication::user_operation::UserInfo;
use crate::handlers::session::LoginStatus;
use crate::handlers::session::{LOGIN_SESSION_EXPIRY, SESSION_ID_COOKIE_NAME};
use crate::{
    err::{unexpected_err_resp, Code},
    handlers::session::authentication::mfa::{ensure_mfa_is_enabled, verify_pass_code},
};

use super::{
    extract_session_id_from_cookie, get_account_id_from_session, get_mfa_info_by_account_id,
    update_login_status, MfaInfo,
};

pub(crate) async fn post_pass_code(
    jar: SignedCookieJar,
    State(pool): State<DatabaseConnection>,
    State(store): State<RedisSessionStore>,
    Json(req): Json<PassCodeReq>,
) -> RespResult<PassCodeReqResult> {
    let option_cookie = jar.get(SESSION_ID_COOKIE_NAME);
    let session_id = extract_session_id_from_cookie(option_cookie)?;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = PassCodeOperationImpl {
        pool,
        expiry: LOGIN_SESSION_EXPIRY,
    };

    handle_pass_code_req(
        session_id.as_str(),
        &current_date_time,
        req.pass_code.as_str(),
        USER_TOTP_ISSUER.as_str(),
        &op,
        &store,
    )
    .await
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PassCodeReq {
    pass_code: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct PassCodeReqResult {}

#[async_trait]
trait PassCodeOperation {
    async fn get_user_info_if_available(&self, account_id: i64) -> Result<UserInfo, ErrResp>;

    async fn get_mfa_info_by_account_id(&self, account_id: i64) -> Result<MfaInfo, ErrResp>;

    fn set_login_session_expiry(&self, session: &mut Session);

    async fn update_last_login(
        &self,
        account_id: i64,
        login_time: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct PassCodeOperationImpl {
    pool: DatabaseConnection,
    expiry: Duration,
}

#[async_trait]
impl PassCodeOperation for PassCodeOperationImpl {
    async fn get_user_info_if_available(&self, account_id: i64) -> Result<UserInfo, ErrResp> {
        let op = FindUserInfoOperationImpl::new(&self.pool);
        let user_info =
            crate::handlers::session::authentication::user_operation::get_user_info_if_available(
                account_id, &op,
            )
            .await?;
        Ok(user_info)
    }

    async fn get_mfa_info_by_account_id(&self, account_id: i64) -> Result<MfaInfo, ErrResp> {
        get_mfa_info_by_account_id(account_id, &self.pool).await
    }

    fn set_login_session_expiry(&self, session: &mut Session) {
        session.expire_in(self.expiry);
    }

    async fn update_last_login(
        &self,
        account_id: i64,
        login_time: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        crate::handlers::session::authentication::update_last_login(
            account_id, login_time, &self.pool,
        )
        .await
    }
}

async fn handle_pass_code_req(
    session_id: &str,
    current_date_time: &DateTime<FixedOffset>,
    pass_code: &str,
    issuer: &str,
    op: &impl PassCodeOperation,
    store: &impl SessionStore,
) -> RespResult<PassCodeReqResult> {
    validate_pass_code(pass_code).map_err(|e| {
        error!("invalid pass code format: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidPassCode as u32,
            }),
        )
    })?;
    let mut session = get_session_by_session_id(session_id, store).await?;
    let account_id = get_account_id_from_session(&session)?;
    let user_info = op.get_user_info_if_available(account_id).await?;
    ensure_mfa_is_enabled(user_info.mfa_enabled_at.is_some())?;

    let mi = op.get_mfa_info_by_account_id(account_id).await?;
    verify_pass_code(
        account_id,
        &mi.base32_encoded_secret,
        issuer,
        current_date_time,
        pass_code,
    )?;

    update_login_status(&mut session, LoginStatus::Finish)?;
    op.set_login_session_expiry(&mut session);
    let _ = store.store_session(session).await.map_err(|e| {
        error!("failed to store session: {}", e);
        unexpected_err_resp()
    })?;

    op.update_last_login(account_id, current_date_time).await?;

    Ok((StatusCode::OK, Json(PassCodeReqResult {})))
}

#[cfg(test)]
mod tests {

    use async_session::MemoryStore;
    use chrono::TimeZone;
    use common::mfa::hash_recovery_code;
    use once_cell::sync::Lazy;

    use crate::handlers::session::{
        tests::prepare_session, KEY_TO_LOGIN_STATUS, KEY_TO_USER_ACCOUNT_ID,
    };

    use super::*;

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<PassCodeReqResult>,
    }

    #[derive(Debug)]
    struct Input {
        session_exists: bool,
        ls: LoginStatus,
        current_date_time: DateTime<FixedOffset>,
        pass_code: String,
        issuer: String,
        op: PassCodeOperationMock,
    }

    impl Input {
        fn new(
            session_exists: bool,
            ls: LoginStatus,
            current_date_time: DateTime<FixedOffset>,
            pass_code: String,
            issuer: String,
            user_info: UserInfo,
            mfa_info: MfaInfo,
        ) -> Self {
            Input {
                session_exists,
                ls,
                current_date_time,
                pass_code,
                issuer,
                op: PassCodeOperationMock {
                    user_info,
                    mfa_info,
                    login_time: current_date_time,
                },
            }
        }
    }

    #[derive(Clone, Debug)]
    struct PassCodeOperationMock {
        user_info: UserInfo,
        mfa_info: MfaInfo,
        login_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl PassCodeOperation for PassCodeOperationMock {
        async fn get_user_info_if_available(&self, account_id: i64) -> Result<UserInfo, ErrResp> {
            // セッションがない場合や無効化されている場合はエラーとなる。
            // その動作はこの実装で実際に呼び出している関数のテストでされているのでここでは実施しない。
            assert_eq!(self.user_info.account_id, account_id);
            Ok(self.user_info.clone())
        }

        async fn get_mfa_info_by_account_id(&self, account_id: i64) -> Result<MfaInfo, ErrResp> {
            assert_eq!(self.user_info.account_id, account_id);
            Ok(self.mfa_info.clone())
        }

        fn set_login_session_expiry(&self, _session: &mut Session) {
            // テスト実行中に有効期限が過ぎるケースを考慮し、有効期限は設定しない
        }

        async fn update_last_login(
            &self,
            account_id: i64,
            login_time: &DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.user_info.account_id, account_id);
            assert_eq!(self.login_time, *login_time);
            Ok(())
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        let session_exists = true;
        let ls = LoginStatus::NeedMoreVerification;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 4, 5, 0, 1, 7)
            .unwrap();
        let user_info = UserInfo {
            account_id: 413,
            email_address: "test@test.com".to_string(),
            mfa_enabled_at: Some(current_date_time - chrono::Duration::days(3)),
            disabled_at: None,
        };
        let mfa_info = MfaInfo {
            base32_encoded_secret: "NKQHIV55R4LJV3MD6YSC4Z4UCMT3NDYD".to_string(),
            hashed_recovery_code: hash_recovery_code("41ae5398f71c424e910d85734c204f1e")
                .expect("failed to get Ok"),
        };
        // 上記のbase32_encoded_secretとcurrent_date_timeでGoogle Authenticatorが実際に算出した値
        let pass_code = "540940";
        let issuer = "Issuer";

        vec![
            TestCase {
                name: "success".to_string(),
                input: Input::new(
                    session_exists,
                    ls.clone(),
                    current_date_time,
                    pass_code.to_string(),
                    issuer.to_string(),
                    user_info.clone(),
                    mfa_info.clone(),
                ),
                expected: Ok((StatusCode::OK, Json(PassCodeReqResult {}))),
            },
            TestCase {
                name: "fail InvalidPassCode".to_string(),
                input: Input::new(
                    session_exists,
                    ls.clone(),
                    current_date_time,
                    "Acd#%&".to_string(),
                    issuer.to_string(),
                    user_info.clone(),
                    mfa_info.clone(),
                ),
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidPassCode as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail Unauthorized (no session found)".to_string(),
                input: Input::new(
                    false,
                    ls.clone(),
                    current_date_time,
                    pass_code.to_string(),
                    issuer.to_string(),
                    user_info.clone(),
                    mfa_info.clone(),
                ),
                expected: Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ApiError {
                        code: Code::Unauthorized as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail MfaIsNotEnabled".to_string(),
                input: Input::new(
                    session_exists,
                    ls.clone(),
                    current_date_time,
                    pass_code.to_string(),
                    issuer.to_string(),
                    UserInfo {
                        account_id: 413,
                        email_address: "test@test.com".to_string(),
                        mfa_enabled_at: None,
                        disabled_at: None,
                    },
                    mfa_info.clone(),
                ),
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::MfaIsNotEnabled as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail PassCodeDoesNotMatch".to_string(),
                input: Input::new(
                    session_exists,
                    ls,
                    current_date_time,
                    "123456".to_string(),
                    issuer.to_string(),
                    user_info,
                    mfa_info,
                ),
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PassCodeDoesNotMatch as u32,
                    }),
                )),
            },
        ]
    });

    #[tokio::test]
    async fn handle_pass_code_req_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let current_date_time = test_case.input.current_date_time;
            let pass_code = test_case.input.pass_code.clone();
            let issuer = test_case.input.issuer.clone();
            let op = test_case.input.op.clone();
            let store = MemoryStore::new();
            let session_id = if test_case.input.session_exists {
                prepare_session(
                    test_case.input.op.user_info.account_id,
                    test_case.input.ls.clone(),
                    &store,
                )
                .await
            } else {
                // 適当なセッションIDをSessionStoreに入れずに用意する
                "4d/UQZs+7mY0kF16rdf8qb07y2TzyHM2LCooSqBJB4GuF5LHw8h5jFLoJmbR3wYbwpy9bGQB2DExLM4lxvD62A==".to_string()
            };

            let result = handle_pass_code_req(
                session_id.as_str(),
                &current_date_time,
                pass_code.as_str(),
                issuer.as_str(),
                &op,
                &store,
            )
            .await;

            let message = format!("test case \"{}\" failed", test_case.name.clone());
            if test_case.expected.is_ok() {
                let resp = result.expect("failed to get Ok");
                let expected = test_case.expected.as_ref().expect("failed to get Ok");
                assert_eq!(expected.0, resp.0, "{}", message);
                assert_eq!(expected.1 .0, resp.1 .0, "{}", message);

                let session = store
                    .load_session(session_id.to_string())
                    .await
                    .expect("failed to get Ok")
                    .expect("failed to get value");
                let result1 = session
                    .get::<i64>(KEY_TO_USER_ACCOUNT_ID)
                    .expect("failed to get Ok");
                assert_eq!(result1, test_case.input.op.user_info.account_id);
                let result2 = session
                    .get::<String>(KEY_TO_LOGIN_STATUS)
                    .expect("failed to get Ok");
                assert_eq!(result2, String::from(LoginStatus::Finish));
            } else {
                let resp = result.expect_err("failed to get Err");
                let expected = test_case.expected.as_ref().expect_err("failed to get Err");
                assert_eq!(expected.0, resp.0, "{}", message);
                assert_eq!(expected.1 .0, resp.1 .0, "{}", message);

                let option_session = store
                    .load_session(session_id.to_string())
                    .await
                    .expect("failed to get Ok");
                if let Some(session) = option_session {
                    let result1 = session
                        .get::<i64>(KEY_TO_USER_ACCOUNT_ID)
                        .expect("failed to get Ok");
                    assert_eq!(result1, test_case.input.op.user_info.account_id);
                    let result2 = session
                        .get::<String>(KEY_TO_LOGIN_STATUS)
                        .expect("failed to get Ok");
                    assert_ne!(result2, String::from(LoginStatus::Finish));
                }
            }
        }
    }
}
