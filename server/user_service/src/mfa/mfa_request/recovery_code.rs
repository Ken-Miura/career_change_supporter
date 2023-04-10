// Copyright 2023 Ken Miura

use std::time::Duration;

use async_redis_session::RedisSessionStore;
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
use crate::mfa::ensure_mfa_is_enabled;
use crate::mfa::mfa_request::get_session_by_session_id;
use crate::util::login_status::LoginStatus;
use crate::util::session::LOGIN_SESSION_EXPIRY;
use crate::util::user_info::{FindUserInfoOperationImpl, UserInfo};
use crate::{err::Code, util::session::SESSION_ID_COOKIE_NAME};

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
        let user_info = crate::util::get_user_info_if_available(account_id, &op).await?;
        Ok(user_info)
    }

    async fn get_mfa_info_by_account_id(&self, account_id: i64) -> Result<MfaInfo, ErrResp> {
        get_mfa_info_by_account_id(account_id, &self.pool).await
    }

    async fn disable_mfa(&self, account_id: i64) -> Result<(), ErrResp> {
        crate::mfa::disable_mfa(account_id, &self.pool).await
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
                code: common::err::Code::InvalidUuidFormat as u32,
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
    use axum::async_trait;
    use chrono::{DateTime, FixedOffset};
    use common::{ErrResp, RespResult};
    use once_cell::sync::Lazy;

    use crate::{
        mfa::mfa_request::MfaInfo,
        util::{
            login_status::LoginStatus,
            session::{tests::prepare_session, KEY_TO_LOGIN_STATUS, KEY_TO_USER_ACCOUNT_ID},
            user_info::UserInfo,
        },
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

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| vec![]);

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
