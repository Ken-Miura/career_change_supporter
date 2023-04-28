// Copyright 2023 Ken Miura

use std::time::Duration;

use async_fred_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use axum_extra::extract::SignedCookieJar;
use chrono::{DateTime, FixedOffset, Utc};
use common::mfa::is_recovery_code_match;
use common::util::validator::uuid_validator::validate_uuid;
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::unexpected_err_resp;
use crate::err::Code;
use crate::handlers::session::authentication::mfa::ensure_mfa_is_enabled;
use crate::handlers::session::authentication::mfa::get_session_by_session_id;
use crate::handlers::session::authentication::user_operation::{
    FindUserInfoOperationImpl, UserInfo,
};
use crate::handlers::session::{LOGIN_SESSION_EXPIRY, SESSION_ID_COOKIE_NAME};
use crate::util::login_status::LoginStatus;

use super::{
    extract_session_id_from_cookie, get_account_id_from_session, get_mfa_info_by_account_id,
    update_login_status, MfaInfo,
};

pub(crate) async fn post_recovery_code(
    jar: SignedCookieJar,
    State(pool): State<DatabaseConnection>,
    State(store): State<RedisSessionStore>,
    Json(req): Json<RecoveryCodeReq>,
) -> RespResult<RecoveryCodeReqResult> {
    let option_cookie = jar.get(SESSION_ID_COOKIE_NAME);
    let session_id = extract_session_id_from_cookie(option_cookie)?;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = RecoveryCodeOperationImpl {
        pool,
        expiry: LOGIN_SESSION_EXPIRY,
    };

    handle_recovery_code(
        session_id.as_str(),
        &current_date_time,
        req.recovery_code.as_str(),
        &op,
        &store,
    )
    .await
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RecoveryCodeReq {
    recovery_code: String,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct RecoveryCodeReqResult {}

#[async_trait]
trait RecoveryCodeOperation {
    async fn get_user_info_if_available(&self, account_id: i64) -> Result<UserInfo, ErrResp>;

    async fn get_mfa_info_by_account_id(&self, account_id: i64) -> Result<MfaInfo, ErrResp>;

    async fn disable_mfa(&self, account_id: i64) -> Result<(), ErrResp>;

    fn set_login_session_expiry(&self, session: &mut Session);

    async fn update_last_login(
        &self,
        account_id: i64,
        login_time: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct RecoveryCodeOperationImpl {
    pool: DatabaseConnection,
    expiry: Duration,
}

#[async_trait]
impl RecoveryCodeOperation for RecoveryCodeOperationImpl {
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

    async fn disable_mfa(&self, account_id: i64) -> Result<(), ErrResp> {
        crate::handlers::session::authentication::mfa::disable_mfa(account_id, &self.pool).await
    }

    fn set_login_session_expiry(&self, session: &mut Session) {
        session.expire_in(self.expiry);
    }

    async fn update_last_login(
        &self,
        account_id: i64,
        login_time: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        crate::util::update_last_login(account_id, login_time, &self.pool).await
    }
}

async fn handle_recovery_code(
    session_id: &str,
    current_date_time: &DateTime<FixedOffset>,
    recovery_code: &str,
    op: &impl RecoveryCodeOperation,
    store: &impl SessionStore,
) -> RespResult<RecoveryCodeReqResult> {
    validate_uuid(recovery_code).map_err(|e| {
        error!("failed to validate {}: {}", recovery_code, e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidRecoveryCode as u32,
            }),
        )
    })?;
    let mut session = get_session_by_session_id(session_id, store).await?;
    let account_id = get_account_id_from_session(&session)?;
    let user_info = op.get_user_info_if_available(account_id).await?;
    ensure_mfa_is_enabled(user_info.mfa_enabled_at.is_some())?;

    let mi = op.get_mfa_info_by_account_id(account_id).await?;
    verify_recovery_code(recovery_code, &mi.hashed_recovery_code)?;

    op.disable_mfa(account_id).await?;

    update_login_status(&mut session, LoginStatus::Finish)?;
    op.set_login_session_expiry(&mut session);
    let _ = store.store_session(session).await.map_err(|e| {
        error!("failed to store session: {}", e);
        unexpected_err_resp()
    })?;

    op.update_last_login(account_id, current_date_time).await?;

    Ok((StatusCode::OK, Json(RecoveryCodeReqResult {})))
}

fn verify_recovery_code(recovery_code: &str, hashed_recovery_code: &[u8]) -> Result<(), ErrResp> {
    let matched = is_recovery_code_match(recovery_code, hashed_recovery_code).map_err(|e| {
        error!("failed is_recovery_code_match: {}", e);
        unexpected_err_resp()
    })?;
    if !matched {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::RecoveryCodeDoesNotMatch as u32,
            }),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use async_session::{MemoryStore, Session, SessionStore};
    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use chrono::{DateTime, FixedOffset, TimeZone};
    use common::ApiError;
    use common::{mfa::hash_recovery_code, ErrResp, RespResult, JAPANESE_TIME_ZONE};
    use once_cell::sync::Lazy;

    use crate::err::Code;
    use crate::handlers::session::authentication::user_operation::UserInfo;
    use crate::{
        handlers::session::authentication::mfa::MfaInfo,
        handlers::session::{tests::prepare_session, KEY_TO_LOGIN_STATUS, KEY_TO_USER_ACCOUNT_ID},
        util::login_status::LoginStatus,
    };

    use super::{handle_recovery_code, RecoveryCodeOperation, RecoveryCodeReqResult};

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<RecoveryCodeReqResult>,
    }

    #[derive(Debug)]
    struct Input {
        session_exists: bool,
        ls: LoginStatus,
        current_date_time: DateTime<FixedOffset>,
        recovery_code: String,
        op: RecoveryCodeOperationMock,
    }

    impl Input {
        fn new(
            session_exists: bool,
            ls: LoginStatus,
            current_date_time: DateTime<FixedOffset>,
            recovery_code: String,
            user_info: UserInfo,
            mfa_info: MfaInfo,
        ) -> Self {
            Input {
                session_exists,
                ls,
                current_date_time,
                recovery_code,
                op: RecoveryCodeOperationMock {
                    user_info,
                    mfa_info,
                    login_time: current_date_time,
                },
            }
        }
    }

    #[derive(Debug, Clone)]
    struct RecoveryCodeOperationMock {
        user_info: UserInfo,
        mfa_info: MfaInfo,
        login_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl RecoveryCodeOperation for RecoveryCodeOperationMock {
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

        async fn disable_mfa(&self, account_id: i64) -> Result<(), ErrResp> {
            assert_eq!(self.user_info.account_id, account_id);
            Ok(())
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
        let recovery_code = "41ae5398f71c424e910d85734c204f1e";
        let mfa_info = MfaInfo {
            base32_encoded_secret: "NKQHIV55R4LJV3MD6YSC4Z4UCMT3NDYD".to_string(),
            hashed_recovery_code: hash_recovery_code(recovery_code).expect("failed to get Ok"),
        };

        vec![
            TestCase {
                name: "success".to_string(),
                input: Input::new(
                    session_exists,
                    ls.clone(),
                    current_date_time,
                    recovery_code.to_string(),
                    user_info.clone(),
                    mfa_info.clone(),
                ),
                expected: Ok((StatusCode::OK, Json(RecoveryCodeReqResult {}))),
            },
            TestCase {
                name: "fail InvalidRecoveryCode".to_string(),
                input: Input::new(
                    session_exists,
                    ls.clone(),
                    current_date_time,
                    "abcdEFGH1234!\"#$".to_string(),
                    user_info.clone(),
                    mfa_info.clone(),
                ),
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidRecoveryCode as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail Unauthorized".to_string(),
                input: Input::new(
                    false,
                    ls.clone(),
                    current_date_time,
                    recovery_code.to_string(),
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
                    recovery_code.to_string(),
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
                name: "fail RecoveryCodeDoesNotMatch".to_string(),
                input: Input::new(
                    session_exists,
                    ls,
                    current_date_time,
                    "51ae5398f71c424e910d85734c204f1f".to_string(),
                    user_info,
                    mfa_info,
                ),
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::RecoveryCodeDoesNotMatch as u32,
                    }),
                )),
            },
        ]
    });

    #[tokio::test]
    async fn handle_recovery_code_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let current_date_time = test_case.input.current_date_time;
            let recovery_code = test_case.input.recovery_code.clone();
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

            let result = handle_recovery_code(
                session_id.as_str(),
                &current_date_time,
                recovery_code.as_str(),
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
