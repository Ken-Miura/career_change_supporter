// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::mfa::generate_base64_encoded_qr_code;
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::mfa::{ensure_mfa_is_not_enabled, get_latest_temp_mfa_secret, USER_TOTP_ISSUER};
use crate::mfa::{filter_temp_mfa_secret_order_by_dsc, TempMfaSecret};
use crate::util::session::user::User;

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

    let qr_code = generate_base64_encoded_qr_code(
        account_id,
        temp_mfa_secret.base32_encoded_secret.as_str(),
        USER_TOTP_ISSUER.as_str(),
    )?;

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
        filter_temp_mfa_secret_order_by_dsc(account_id, current_date_time, &self.pool).await
    }
}

#[cfg(test)]
mod tests {}
