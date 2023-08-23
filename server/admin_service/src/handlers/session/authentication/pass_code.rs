// Copyright 2023 Ken Miura

use std::env;
use std::time::Duration;

use async_fred_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::SignedCookieJar;
use chrono::{DateTime, FixedOffset, Utc};
use common::admin::KEY_TO_ADMIN_TOTP_ISSUER;
use common::mfa::check_if_pass_code_matches;
use common::util::validator::pass_code_validator::validate_pass_code;
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::authentication::get_session_by_session_id;
use crate::handlers::session::ADMIN_SESSION_ID_COOKIE_NAME;

use super::admin_operation::{AdminInfo, FindAdminInfoOperationImpl};
use super::{LoginStatus, KEY_TO_ADMIN_ACCOUNT_ID, KEY_TO_LOGIN_STATUS, LOGIN_SESSION_EXPIRY};

static ADMIN_TOTP_ISSUER: Lazy<String> = Lazy::new(|| {
    let issuer = env::var(KEY_TO_ADMIN_TOTP_ISSUER).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_ADMIN_TOTP_ISSUER
        )
    });
    if issuer.contains(':') {
        panic!("ADMIN_TOTP_ISSUER must not contain \":\": {}", issuer);
    }
    issuer
});

pub(crate) async fn post_pass_code(
    jar: SignedCookieJar,
    State(pool): State<DatabaseConnection>,
    State(store): State<RedisSessionStore>,
    Json(req): Json<PassCodeReq>,
) -> RespResult<PassCodeReqResult> {
    let option_cookie = jar.get(ADMIN_SESSION_ID_COOKIE_NAME);
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
        ADMIN_TOTP_ISSUER.as_str(),
        &op,
        &store,
    )
    .await
}

fn extract_session_id_from_cookie(cookie: Option<Cookie>) -> Result<String, ErrResp> {
    let session_id = match cookie {
        Some(s) => s.value().to_string(),
        None => {
            error!("no sessoin cookie found");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: Code::Unauthorized as u32,
                }),
            ));
        }
    };
    Ok(session_id)
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PassCodeReq {
    pass_code: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct PassCodeReqResult {}

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
                code: Code::InvalidPassCodeFormat as u32,
            }),
        )
    })?;
    let mut session = get_session_by_session_id(session_id, store).await?;
    let account_id = get_admin_account_id_from_session(&session)?;
    let admin_info = op.get_admin_info_by_account_id(account_id).await?;
    ensure_mfa_is_enabled(admin_info.mfa_enabled_at.is_some())?;

    let mi = op.get_admin_mfa_info_by_account_id(account_id).await?;
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

fn get_admin_account_id_from_session(session: &Session) -> Result<i64, ErrResp> {
    let account_id = match session.get::<i64>(KEY_TO_ADMIN_ACCOUNT_ID) {
        Some(id) => id,
        None => {
            error!("failed to get admin account id from session");
            return Err(unexpected_err_resp());
        }
    };
    Ok(account_id)
}

fn ensure_mfa_is_enabled(mfa_enabled: bool) -> Result<(), ErrResp> {
    if !mfa_enabled {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::MfaIsNotEnabled as u32,
            }),
        ));
    }
    Ok(())
}

fn verify_pass_code(
    account_id: i64,
    base32_encoded_secret: &str,
    issuer: &str,
    current_date_time: &DateTime<FixedOffset>,
    pass_code: &str,
) -> Result<(), ErrResp> {
    let matched = check_if_pass_code_matches(
        account_id,
        base32_encoded_secret,
        issuer,
        current_date_time,
        pass_code,
    )?;
    if !matched {
        error!(
            "failed to check pass code (account_id: {}, current_date_time: {})",
            account_id, current_date_time
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PassCodeDoesNotMatch as u32,
            }),
        ));
    }
    Ok(())
}

fn update_login_status(session: &mut Session, ls: LoginStatus) -> Result<(), ErrResp> {
    session
        .insert(KEY_TO_LOGIN_STATUS, ls.clone())
        .map_err(|e| {
            error!(
                "failed to insert login_status ({}) into session: {}",
                String::from(ls),
                e
            );
            unexpected_err_resp()
        })?;
    Ok(())
}

#[async_trait]
trait PassCodeOperation {
    async fn get_admin_info_by_account_id(&self, account_id: i64) -> Result<AdminInfo, ErrResp>;

    async fn get_admin_mfa_info_by_account_id(&self, account_id: i64) -> Result<MfaInfo, ErrResp>;

    fn set_login_session_expiry(&self, session: &mut Session);

    async fn update_last_login(
        &self,
        account_id: i64,
        login_time: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

#[derive(Clone, Debug)]
struct MfaInfo {
    base32_encoded_secret: String,
}

struct PassCodeOperationImpl {
    pool: DatabaseConnection,
    expiry: Duration,
}

#[async_trait]
impl PassCodeOperation for PassCodeOperationImpl {
    async fn get_admin_info_by_account_id(&self, account_id: i64) -> Result<AdminInfo, ErrResp> {
        let op = FindAdminInfoOperationImpl::new(&self.pool);
        let admin_info =
            crate::handlers::session::authentication::admin_operation::get_admin_info_by_account_id(
                account_id, &op,
            )
            .await?;
        Ok(admin_info)
    }

    async fn get_admin_mfa_info_by_account_id(&self, account_id: i64) -> Result<MfaInfo, ErrResp> {
        let result = entity::admin_mfa_info::Entity::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find admin_mfa_info (admin_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        let mi = result.ok_or_else(|| {
            error!("no admin_mfa_info (admin_account_id: {}) found", account_id);
            unexpected_err_resp()
        })?;
        Ok(MfaInfo {
            base32_encoded_secret: mi.base32_encoded_secret,
        })
    }

    fn set_login_session_expiry(&self, session: &mut Session) {
        session.expire_in(self.expiry);
    }

    async fn update_last_login(
        &self,
        account_id: i64,
        login_time: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        super::update_last_login(account_id, login_time, &self.pool).await
    }
}

#[cfg(test)]
mod tests {

    use async_session::MemoryStore;
    use chrono::TimeZone;

    use crate::handlers::session::authentication::tests::prepare_login_session;

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
            admin_info: AdminInfo,
            mfa_info: MfaInfo,
        ) -> Self {
            Input {
                session_exists,
                ls,
                current_date_time,
                pass_code,
                issuer,
                op: PassCodeOperationMock {
                    admin_info,
                    mfa_info,
                    login_time: current_date_time,
                },
            }
        }
    }

    #[derive(Clone, Debug)]
    struct PassCodeOperationMock {
        admin_info: AdminInfo,
        mfa_info: MfaInfo,
        login_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl PassCodeOperation for PassCodeOperationMock {
        async fn get_admin_info_by_account_id(
            &self,
            account_id: i64,
        ) -> Result<AdminInfo, ErrResp> {
            // セッションがない場合はエラーとなる。
            // その動作はこの実装で実際に呼び出している関数のテストでされているのでここでは実施しない。
            assert_eq!(self.admin_info.account_id, account_id);
            Ok(self.admin_info.clone())
        }

        async fn get_admin_mfa_info_by_account_id(
            &self,
            account_id: i64,
        ) -> Result<MfaInfo, ErrResp> {
            assert_eq!(self.admin_info.account_id, account_id);
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
            assert_eq!(self.admin_info.account_id, account_id);
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
        let admin_info = AdminInfo {
            account_id: 413,
            email_address: "test@test.com".to_string(),
            mfa_enabled_at: Some(current_date_time - chrono::Duration::days(3)),
        };
        let mfa_info = MfaInfo {
            base32_encoded_secret: "NKQHIV55R4LJV3MD6YSC4Z4UCMT3NDYD".to_string(),
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
                    admin_info.clone(),
                    mfa_info.clone(),
                ),
                expected: Ok((StatusCode::OK, Json(PassCodeReqResult {}))),
            },
            TestCase {
                name: "fail InvalidPassCodeFormat".to_string(),
                input: Input::new(
                    session_exists,
                    ls.clone(),
                    current_date_time,
                    "Acd#%&".to_string(),
                    issuer.to_string(),
                    admin_info.clone(),
                    mfa_info.clone(),
                ),
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidPassCodeFormat as u32,
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
                    admin_info.clone(),
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
                    AdminInfo {
                        account_id: 413,
                        email_address: "test@test.com".to_string(),
                        mfa_enabled_at: None,
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
                    admin_info,
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
                prepare_login_session(
                    test_case.input.op.admin_info.account_id,
                    test_case.input.ls.clone(),
                    &store,
                )
                .await
            } else {
                // 適当なセッションIDをSessionStoreに入れずに用意する
                "zRzdhSMOThIkeF6F8WjOXhIMIXhdSW/bFSAzZq8a40cg7cn9JssNRIR6+9xKdPn6S4h0jkTgYdgEhD/aTYG0wg==".to_string()
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
                    .get::<i64>(KEY_TO_ADMIN_ACCOUNT_ID)
                    .expect("failed to get Ok");
                assert_eq!(result1, test_case.input.op.admin_info.account_id);
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
                        .get::<i64>(KEY_TO_ADMIN_ACCOUNT_ID)
                        .expect("failed to get Ok");
                    assert_eq!(result1, test_case.input.op.admin_info.account_id);
                    let result2 = session
                        .get::<String>(KEY_TO_LOGIN_STATUS)
                        .expect("failed to get Ok");
                    assert_ne!(result2, String::from(LoginStatus::Finish));
                }
            }
        }
    }
}
