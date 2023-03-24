// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use entity::sea_orm::{QueryOrder, QuerySelect};
use serde::Serialize;
use tracing::{error, info};

use crate::err::Code;
use crate::mfa::{create_totp, ensure_mfa_is_not_enabled};
use crate::{err::unexpected_err_resp, util::session::user::User};

use super::MAX_NUM_OF_TEMP_MFA_SECRETS;

pub(crate) async fn get_temp_mfa_secret(
    User { user_info }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<GetTempMfaSecretResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = TempMfaSecretResultOperationImpl { pool };
    handle_temp_mfp_secret(
        user_info.account_id,
        user_info.mfa_enabled_at.is_some(),
        current_date_time,
        op,
    )
    .await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct GetTempMfaSecretResult {
    // QRコード
    base64_encoded_image: String,
    // QRコードを読み込めない場合に使うシークレットキー
    base32_encoded_secret: String,
}

async fn handle_temp_mfp_secret(
    account_id: i64,
    mfa_enabled: bool,
    current_date_time: DateTime<FixedOffset>,
    op: impl TempMfaSecretResultOperation,
) -> RespResult<GetTempMfaSecretResult> {
    ensure_mfa_is_not_enabled(mfa_enabled)?;

    let temp_mfa_secrets = op
        .filter_temp_mfa_secret_order_by_dsc(account_id, current_date_time)
        .await?;
    let temp_mfa_secret = get_latest_temp_mfa_secret(temp_mfa_secrets)?;

    let totp = create_totp(account_id, temp_mfa_secret.base32_encoded_secret.clone())?;
    let qr_code = totp.get_qr().map_err(|e| {
        error!("failed to create QR code (base64 encoded png img): {}", e);
        unexpected_err_resp()
    })?;

    Ok((
        StatusCode::OK,
        Json(GetTempMfaSecretResult {
            base64_encoded_image: qr_code,
            base32_encoded_secret: temp_mfa_secret.base32_encoded_secret,
        }),
    ))
}

#[async_trait]
trait TempMfaSecretResultOperation {
    async fn filter_temp_mfa_secret_order_by_dsc(
        &self,
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<TempMfaSecret>, ErrResp>;
}

#[derive(Clone)]
struct TempMfaSecret {
    temp_mfa_secret_id: i64,
    base32_encoded_secret: String,
    expired_at: DateTime<FixedOffset>,
}

struct TempMfaSecretResultOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl TempMfaSecretResultOperation for TempMfaSecretResultOperationImpl {
    async fn filter_temp_mfa_secret_order_by_dsc(
        &self,
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<TempMfaSecret>, ErrResp> {
        let models = entity::temp_mfa_secret::Entity::find()
            .filter(entity::temp_mfa_secret::Column::UserAccountId.eq(account_id))
            .filter(entity::temp_mfa_secret::Column::ExpiredAt.lt(current_date_time))
            .limit(MAX_NUM_OF_TEMP_MFA_SECRETS)
            .order_by_desc(entity::temp_mfa_secret::Column::ExpiredAt)
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter temp_mfa_secret (account_id: {}, current_date_time: {}): {}",
                    account_id, current_date_time, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| TempMfaSecret {
                temp_mfa_secret_id: m.temp_mfa_secret_id,
                base32_encoded_secret: m.base32_encoded_secret,
                expired_at: m.expired_at,
            })
            .collect::<Vec<TempMfaSecret>>())
    }
}

fn get_latest_temp_mfa_secret(
    temp_mfa_secrets: Vec<TempMfaSecret>,
) -> Result<TempMfaSecret, ErrResp> {
    if temp_mfa_secrets.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoTempMfaSecretFound as u32,
            }),
        ));
    }
    let secret = temp_mfa_secrets.get(0).ok_or_else(|| {
        error!("there are no temp_mfa_secrets");
        unexpected_err_resp()
    })?;
    info!(
        "returns temp_mfa_secret_id ({}) expired at {}",
        secret.temp_mfa_secret_id, secret.expired_at
    );
    Ok(secret.clone())
}

#[cfg(test)]
mod tests {}
