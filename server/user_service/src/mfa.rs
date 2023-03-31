// Copyright 2023 Ken Miura

use std::env;

use axum::{http::StatusCode, Json};
use chrono::{DateTime, FixedOffset};
use common::{mfa::check_if_pass_code_matches, ApiError, ErrResp};
use entity::sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use once_cell::sync::Lazy;
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

pub(crate) mod pass_code;
pub(crate) mod setting_change;
pub(crate) mod temp_secret;

const MAX_NUM_OF_TEMP_MFA_SECRETS: u64 = 8;

pub(crate) const KEY_TO_USER_TOTP_ISSUER: &str = "USER_TOTP_ISSUER";
static USER_TOTP_ISSUER: Lazy<String> = Lazy::new(|| {
    let issuer = env::var(KEY_TO_USER_TOTP_ISSUER).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_USER_TOTP_ISSUER
        )
    });
    if issuer.contains(':') {
        panic!("USER_TOTP_ISSUER must not contain \":\": {}", issuer);
    }
    issuer
});

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

#[derive(Clone)]
struct TempMfaSecret {
    temp_mfa_secret_id: i64,
    base32_encoded_secret: String,
}

async fn filter_temp_mfa_secret_order_by_dsc(
    account_id: i64,
    current_date_time: DateTime<FixedOffset>,
    pool: &DatabaseConnection,
) -> Result<Vec<TempMfaSecret>, ErrResp> {
    let models = entity::temp_mfa_secret::Entity::find()
        .filter(entity::temp_mfa_secret::Column::UserAccountId.eq(account_id))
        .filter(entity::temp_mfa_secret::Column::ExpiredAt.gt(current_date_time))
        .limit(MAX_NUM_OF_TEMP_MFA_SECRETS)
        .order_by_desc(entity::temp_mfa_secret::Column::ExpiredAt)
        .all(pool)
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
        })
        .collect::<Vec<TempMfaSecret>>())
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
    Ok(secret.clone())
}

pub(crate) fn verify_pass_code(
    account_id: i64,
    base32_encoded_secret: &str,
    current_date_time: &DateTime<FixedOffset>,
    pass_code: &str,
) -> Result<(), ErrResp> {
    let matched = check_if_pass_code_matches(
        account_id,
        base32_encoded_secret,
        USER_TOTP_ISSUER.as_str(),
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

#[cfg(test)]
mod tests {}
