// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use common::{ErrResp, RespResult};
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::handlers::session::authentication::authenticated_handlers::authenticated_users::user::User;
use crate::handlers::session::authentication::ensure_mfa_is_enabled;

pub(crate) async fn post_disable_mfa_req(
    User { user_info }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<DisableMfaReqResult> {
    let account_id = user_info.account_id;
    let mfa_enabled = user_info.mfa_enabled_at.is_some();
    let op = DisableMfaReqOperationImpl { pool };
    handle_disable_mfa_req(account_id, mfa_enabled, op).await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct DisableMfaReqResult {}

async fn handle_disable_mfa_req(
    account_id: i64,
    mfa_enabled: bool,
    op: impl DisableMfaReqOperation,
) -> RespResult<DisableMfaReqResult> {
    ensure_mfa_is_enabled(mfa_enabled)?;

    op.disable_mfa(account_id).await?;

    Ok((StatusCode::OK, Json(DisableMfaReqResult {})))
}

#[async_trait]
trait DisableMfaReqOperation {
    async fn disable_mfa(&self, account_id: i64) -> Result<(), ErrResp>;
}

struct DisableMfaReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl DisableMfaReqOperation for DisableMfaReqOperationImpl {
    async fn disable_mfa(&self, account_id: i64) -> Result<(), ErrResp> {
        crate::handlers::session::authentication::disable_mfa(account_id, &self.pool).await
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use common::{ApiError, ErrResp, RespResult};
    use once_cell::sync::Lazy;

    use crate::err::Code;

    use super::{handle_disable_mfa_req, DisableMfaReqOperation, DisableMfaReqResult};

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<DisableMfaReqResult>,
    }

    #[derive(Debug)]
    struct Input {
        account_id: i64,
        mfa_enabled: bool,
        op: DisableMfaReqOperationMock,
    }

    impl Input {
        fn new(account_id: i64, mfa_enabled: bool) -> Self {
            Input {
                account_id,
                mfa_enabled,
                op: DisableMfaReqOperationMock { account_id },
            }
        }
    }

    #[derive(Clone, Debug)]
    struct DisableMfaReqOperationMock {
        account_id: i64,
    }

    #[async_trait]
    impl DisableMfaReqOperation for DisableMfaReqOperationMock {
        async fn disable_mfa(&self, account_id: i64) -> Result<(), ErrResp> {
            assert_eq!(self.account_id, account_id);
            Ok(())
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        let account_id = 51321;
        let mfa_enabled = true;
        vec![
            TestCase {
                name: "success".to_string(),
                input: Input::new(account_id, mfa_enabled),
                expected: Ok((StatusCode::OK, Json(DisableMfaReqResult {}))),
            },
            TestCase {
                name: "fail MfaIsNotEnabled".to_string(),
                input: Input::new(account_id, false),
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::MfaIsNotEnabled as u32,
                    }),
                )),
            },
        ]
    });

    #[tokio::test]
    async fn handle_disable_mfa_req_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let account_id = test_case.input.account_id;
            let mfa_enabled = test_case.input.mfa_enabled;
            let op = test_case.input.op.clone();

            let result = handle_disable_mfa_req(account_id, mfa_enabled, op).await;

            let message = format!("test case \"{}\" failed", test_case.name.clone());
            if test_case.expected.is_ok() {
                let resp = result.expect("failed to get Ok");
                let expected = test_case.expected.as_ref().expect("failed to get Ok");
                assert_eq!(expected.0, resp.0, "{}", message);
                assert_eq!(expected.1 .0, resp.1 .0, "{}", message);
            } else {
                let resp = result.expect_err("failed to get Err");
                let expected = test_case.expected.as_ref().expect_err("failed to get Err");
                assert_eq!(expected.0, resp.0, "{}", message);
                assert_eq!(expected.1 .0, resp.1 .0, "{}", message);
            }
        }
    }
}
