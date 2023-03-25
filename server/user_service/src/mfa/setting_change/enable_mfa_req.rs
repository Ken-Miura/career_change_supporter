// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::mfa::hash_recovery_code;
use common::util::validator::pass_code_validator::validate_pass_code;
use common::util::validator::uuid_validator::validate_uuid;
use common::{ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionError, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::err::{unexpected_err_resp, Code};
use crate::mfa::{
    ensure_mfa_is_not_enabled, filter_temp_mfa_secret_order_by_dsc, verify_pass_code,
};
use crate::mfa::{get_latest_temp_mfa_secret, TempMfaSecret};
use crate::util::find_user_account_by_user_account_id_with_exclusive_lock;
use crate::util::session::user::User;

pub(crate) async fn post_enable_mfa_req(
    User { user_info }: User,
    State(pool): State<DatabaseConnection>,
    Json(enable_mfa_req): Json<EnableMfaReq>,
) -> RespResult<EnableMfaReqResult> {
    let account_id = user_info.account_id;
    let mfa_enabled = user_info.mfa_enabled_at.is_some();
    let pass_code = enable_mfa_req.pass_code;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let uuid = Uuid::new_v4().simple().to_string();
    let op = EnableMfaReqOperationImpl { pool };
    handle_enable_mfa_req(
        account_id,
        mfa_enabled,
        pass_code,
        current_date_time,
        uuid,
        op,
    )
    .await
}

#[derive(Deserialize)]
pub(crate) struct EnableMfaReq {
    pass_code: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct EnableMfaReqResult {
    recovery_code: String,
}

async fn handle_enable_mfa_req(
    account_id: i64,
    mfa_enabled: bool,
    pass_code: String,
    current_date_time: DateTime<FixedOffset>,
    recovery_code: String,
    op: impl EnableMfaReqOperation,
) -> RespResult<EnableMfaReqResult> {
    validate_pass_code(pass_code.as_str()).map_err(|e| {
        error!("invalid pass code format: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidPassCode as u32,
            }),
        )
    })?;
    validate_uuid(recovery_code.as_str()).map_err(|e| {
        // recovery_codeは自身で生成する値ため、問題が発生する場合はunexpected_err_resp
        error!("failed to validate recovery_code: {}", e);
        unexpected_err_resp()
    })?;
    ensure_mfa_is_not_enabled(mfa_enabled)?;

    let temp_mfa_secrets = op
        .filter_temp_mfa_secret_order_by_dsc(account_id, current_date_time)
        .await?;
    let temp_mfa_secret = get_latest_temp_mfa_secret(temp_mfa_secrets)?;

    verify_pass_code(
        account_id,
        &temp_mfa_secret.base32_encoded_secret,
        &current_date_time,
        &pass_code,
    )?;

    let hashed_recovery_code = hash_recovery_code(recovery_code.as_str()).map_err(|e| {
        error!("failed to hash recovery code: {}", e);
        unexpected_err_resp()
    })?;

    op.enable_mfa(
        account_id,
        temp_mfa_secret.base32_encoded_secret,
        hashed_recovery_code,
        current_date_time,
    )
    .await?;

    Ok((StatusCode::OK, Json(EnableMfaReqResult { recovery_code })))
}

#[async_trait]
trait EnableMfaReqOperation {
    async fn filter_temp_mfa_secret_order_by_dsc(
        &self,
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<TempMfaSecret>, ErrResp>;

    async fn enable_mfa(
        &self,
        account_id: i64,
        base32_encoded_secret: String,
        hashed_recovery_code: Vec<u8>,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct EnableMfaReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl EnableMfaReqOperation for EnableMfaReqOperationImpl {
    async fn filter_temp_mfa_secret_order_by_dsc(
        &self,
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<TempMfaSecret>, ErrResp> {
        filter_temp_mfa_secret_order_by_dsc(account_id, current_date_time, &self.pool).await
    }

    async fn enable_mfa(
        &self,
        account_id: i64,
        base32_encoded_secret: String,
        hashed_recovery_code: Vec<u8>,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
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

                    // 設定が有効化されたら見えなくなるため、temp_mfa_secretの削除は実施しない
                    // temp_mfa_secretの削除は定期実行処理に任せる
                    // 補足:
                    // 有効化 -> 無効化を短期間で実施後、temp_mfa_secretを直接取得するAPIを叩くと、短い期間の間以前設定した秘密鍵が見えることがある
                    // しかし、下記の理由から問題ないと判断した。
                    // - 見えるのは短い期間のみ、かつログイン済みのユーザーに対してのみ
                    // - 見えるのは有効化 -> 無効化のタイミングで既に廃棄された秘密鍵

                    let mfa_info_active_model = entity::mfa_info::ActiveModel {
                        user_account_id: Set(account_id),
                        base32_encoded_secret: Set(base32_encoded_secret),
                        hashed_recovery_code: Set(hashed_recovery_code)
                    };
                    let _ = entity::mfa_info::Entity::insert(mfa_info_active_model).exec(txn).await.map_err(|e|{
                        error!(
                            "failed to insert mfa_info (user_account_id: {}): {}",
                            account_id, e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let mut user_active_model: entity::user_account::ActiveModel = user_model.into();
                    user_active_model.mfa_enabled_at = Set(Some(current_date_time));
                    let _ = user_active_model.update(txn).await.map_err(|e| {
                        error!(
                            "failed to update mfa_enabled_at in user_account (user_account_id: {}, current_date_time: {}): {}",
                            account_id, current_date_time, e
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
                    error!("failed to enable_mfa: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}
