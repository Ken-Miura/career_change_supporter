// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, Set,
};
use serde::Serialize;
use tracing::error;

use crate::err::Code;
use crate::{err::unexpected_err_resp, util::session::user::User};

const MAX_NUM_OF_TEMP_MFA_SECRETS: u64 = 8;
const VALID_PERIOD_IN_MINUTE: i64 = 15;

pub(crate) async fn post_temp_mfa_secret(
    User { user_info }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<PostTempMfaSecretResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let base32_encoded_secret = "".to_string();
    let op = TempMfaSecretResultOperationImpl { pool };
    handle_temp_mfp_secret(
        user_info.account_id,
        user_info.mfa_enabled_at.is_some(),
        base32_encoded_secret,
        current_date_time,
        op,
    )
    .await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct PostTempMfaSecretResult {
    // QRコード
    base64_encoded_image: String,
    // QRコードを読み込めない場合に使うシークレットキー
    base32_encoded_secret: String,
}

async fn handle_temp_mfp_secret(
    account_id: i64,
    mfa_enabled: bool,
    _base32_encoded_secret: String,
    current_date_time: DateTime<FixedOffset>,
    op: impl TempMfaSecretResultOperation,
) -> RespResult<PostTempMfaSecretResult> {
    ensure_mfa_is_not_enabled(mfa_enabled)?;
    ensure_temp_mfa_secre_does_not_reach_max_count(account_id, MAX_NUM_OF_TEMP_MFA_SECRETS, &op)
        .await?;

    let _expiry_date_time = current_date_time + chrono::Duration::minutes(VALID_PERIOD_IN_MINUTE);
    // temp_mfa_secretを作成
    todo!()
}

fn ensure_mfa_is_not_enabled(mfa_enabled: bool) -> Result<(), ErrResp> {
    if mfa_enabled {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::MfaHasAlreadyBeenEnabled as u32,
            }),
        ));
    }
    Ok(())
}

async fn ensure_temp_mfa_secre_does_not_reach_max_count(
    account_id: i64,
    max_count: u64,
    op: &impl TempMfaSecretResultOperation,
) -> Result<(), ErrResp> {
    let count = op.count_temp_mfa_secret(account_id).await?;
    if count >= max_count {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ReachTempMfaSecretLimit as u32,
            }),
        ));
    }
    Ok(())
}

#[async_trait]
trait TempMfaSecretResultOperation {
    async fn count_temp_mfa_secret(&self, account_id: i64) -> Result<u64, ErrResp>;

    async fn create_temp_mfa_secret(
        &self,
        account_id: i64,
        base32_encoded_secret: String,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct TempMfaSecretResultOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl TempMfaSecretResultOperation for TempMfaSecretResultOperationImpl {
    async fn count_temp_mfa_secret(&self, account_id: i64) -> Result<u64, ErrResp> {
        let result = entity::temp_mfa_secret::Entity::find()
            .filter(entity::temp_mfa_secret::Column::UserAccountId.eq(account_id))
            .count(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to count temp_mfa_secret (account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(result)
    }

    async fn create_temp_mfa_secret(
        &self,
        account_id: i64,
        base32_encoded_secret: String,
        expiry_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        let active_model = entity::temp_mfa_secret::ActiveModel {
            temp_mfa_secret_id: NotSet,
            user_account_id: Set(account_id),
            base32_encoded_secret: Set(base32_encoded_secret),
            expired_at: Set(expiry_date_time),
        };
        let _ = active_model.insert(&self.pool).await.map_err(|e| {
            error!(
                "failed to insert temp_mfa_secret (user_account_id: {}, expired_at: {}): {}",
                account_id, expiry_date_time, e
            );
            unexpected_err_resp()
        })?;
        Ok(())
    }
}
