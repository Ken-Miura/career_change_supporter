// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use common::{ApiError, ErrResp, ErrRespStruct, RespResult};
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionError, TransactionTrait,
};
use serde::Serialize;
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::util::find_user_account_by_user_account_id_with_exclusive_lock;
use crate::util::session::user::User;

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
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let user_model =
                        find_user_account_by_user_account_id_with_exclusive_lock(txn, account_id)
                            .await?;
                    let user_model = user_model.ok_or_else(|| {
                        error!(
                            "failed to find user_account (user_account_id: {})",
                            account_id
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    // temp_mfa_secretの削除は実施せず、temp_mfa_secretの削除は定期実行処理に任せる
                    // 補足:
                    // 有効化 -> 無効化を短期間で実施後、temp_mfa_secretを直接取得するAPIを叩くと、短い期間の間以前設定した秘密鍵が見えることがある
                    // しかし、下記の理由から問題ないと判断した。
                    // - 見えるのは短い期間のみ、かつログイン済みのユーザーに対してのみ
                    // - 見えるのは有効化 -> 無効化のタイミングで既に廃棄された秘密鍵

                    let _ = entity::mfa_info::Entity::delete_by_id(account_id).exec(txn).await.map_err(|e|{
                        error!(
                            "failed to delete mfa_info (user_account_id: {}): {}",
                            account_id, e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let mut user_active_model: entity::user_account::ActiveModel = user_model.into();
                    user_active_model.mfa_enabled_at = Set(None);
                    let _ = user_active_model.update(txn).await.map_err(|e| {
                        error!(
                            "failed to update mfa_enabled_at in user_account (user_account_id: {}): {}",
                            account_id, e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    Ok(())
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
        Ok(())
    }
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
