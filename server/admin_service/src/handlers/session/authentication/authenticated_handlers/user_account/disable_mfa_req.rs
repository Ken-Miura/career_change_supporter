// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::{ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionError, TransactionTrait,
};
use serde::Deserialize;
use tracing::error;

use crate::err::unexpected_err_resp;
use crate::handlers::session::authentication::authenticated_handlers::user_account_operation::find_user_account_model_by_user_account_id_with_exclusive_lock;

use super::super::admin::Admin;
use super::{validate_account_id_is_positive, UserAccount, UserAccountRetrievalResult};

pub(crate) async fn post_disable_mfa_req(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<DisableMfaReq>,
) -> RespResult<UserAccountRetrievalResult> {
    let op = DisableMfaReqOperationImpl { pool };
    handle_disable_mfa_req(req.user_account_id, &op).await
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct DisableMfaReq {
    user_account_id: i64,
}

async fn handle_disable_mfa_req(
    user_account_id: i64,
    op: &impl DisableMfaReqOperation,
) -> RespResult<UserAccountRetrievalResult> {
    validate_account_id_is_positive(user_account_id)?;
    let ua = op.disable_mfa(user_account_id).await?;
    Ok((
        StatusCode::OK,
        Json(UserAccountRetrievalResult {
            user_account: Some(ua),
        }),
    ))
}

#[async_trait]
trait DisableMfaReqOperation {
    async fn disable_mfa(&self, user_account_id: i64) -> Result<UserAccount, ErrResp>;
}

struct DisableMfaReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl DisableMfaReqOperation for DisableMfaReqOperationImpl {
    async fn disable_mfa(&self, user_account_id: i64) -> Result<UserAccount, ErrResp> {
        let result = self.pool
            .transaction::<_, entity::user_account::Model, ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let user_model =
                        find_user_account_model_by_user_account_id_with_exclusive_lock(
                            txn,
                            user_account_id,
                        )
                        .await?;
                    let user_model = user_model.ok_or_else(|| {
                        error!(
                            "failed to find user_account (user_account_id: {})",
                            user_account_id
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let _ = entity::mfa_info::Entity::delete_by_id(user_account_id)
                        .exec(txn)
                        .await
                        .map_err(|e| {
                            error!(
                                "failed to delete mfa_info (user_account_id: {}): {}",
                                user_account_id, e
                            );
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;

                    let mut user_active_model: entity::user_account::ActiveModel = user_model.into();
                    user_active_model.mfa_enabled_at = Set(None);
                    let result = user_active_model.update(txn).await.map_err(|e| {
                        error!("failed to update mfa_enabled_at in user_account (user_account_id: {}): {}", user_account_id, e);
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    Ok(result)
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(db_err) => {
                    error!("connection error: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    error!("failed to disable_mfa: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;

        Ok(UserAccount {
            user_account_id: result.user_account_id,
            email_address: result.email_address,
            last_login_time: result
                .last_login_time
                .map(|m| m.with_timezone(&(*JAPANESE_TIME_ZONE)).to_rfc3339()),
            created_at: result
                .created_at
                .with_timezone(&(*JAPANESE_TIME_ZONE))
                .to_rfc3339(),
            mfa_enabled_at: result
                .mfa_enabled_at
                .map(|m| m.with_timezone(&(*JAPANESE_TIME_ZONE)).to_rfc3339()),
            disabled_at: result
                .disabled_at
                .map(|m| m.with_timezone(&(*JAPANESE_TIME_ZONE)).to_rfc3339()),
        })
    }
}

#[cfg(test)]
mod tests {

    use crate::err::Code;

    use super::*;

    struct DisableMfaReqOperationMock {
        user_account_id: i64,
        user_account: UserAccount,
    }

    #[async_trait]
    impl DisableMfaReqOperation for DisableMfaReqOperationMock {
        async fn disable_mfa(&self, user_account_id: i64) -> Result<UserAccount, ErrResp> {
            assert!(self.user_account_id == user_account_id);
            Ok(self.user_account.clone())
        }
    }

    fn create_dummy_user_account(user_account_id: i64) -> UserAccount {
        UserAccount {
            user_account_id,
            email_address: "test0@test.com".to_string(),
            last_login_time: Some("2023-04-15T14:12:53.4242+09:00 ".to_string()),
            created_at: "2023-04-13T14:12:53.4242+09:00 ".to_string(),
            mfa_enabled_at: None,
            disabled_at: None,
        }
    }

    #[tokio::test]
    async fn handle_disable_mfa_req_success() {
        let user_account_id = 57301;
        let user_account = create_dummy_user_account(user_account_id);
        let op_mock = DisableMfaReqOperationMock {
            user_account_id,
            user_account: user_account.clone(),
        };

        let result = handle_disable_mfa_req(user_account_id, &op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(resp.0, StatusCode::OK);
        assert_eq!(resp.1 .0.user_account, Some(user_account))
    }

    #[tokio::test]
    async fn handle_disable_mfa_req_fail_user_account_id_is_zero() {
        let user_account_id = 0;
        let op_mock = DisableMfaReqOperationMock {
            user_account_id,
            user_account: create_dummy_user_account(user_account_id),
        };

        let result = handle_disable_mfa_req(user_account_id, &op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn handle_disable_mfa_req_fail_user_account_id_is_negative() {
        let user_account_id = -1;
        let op_mock = DisableMfaReqOperationMock {
            user_account_id,
            user_account: create_dummy_user_account(user_account_id),
        };

        let result = handle_disable_mfa_req(user_account_id, &op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }
}
