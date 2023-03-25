// Copyright 2023 Ken Miura

use std::env;

use axum::{http::StatusCode, Json};
use chrono::{DateTime, FixedOffset};
use common::{ApiError, ErrResp};
use entity::sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use once_cell::sync::Lazy;
use totp_rs::{Algorithm, Secret, TOTP};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

pub(crate) mod setting_change;
pub(crate) mod temp_secret;

const PASS_CODE_DIGITS: usize = 6;
const SKEW: u8 = 1;
const ONE_STEP_IN_SECOND: u64 = 30;

pub(crate) const KEY_TO_TOTP_ISSUER: &str = "TOTP_ISSUER";
static TOTP_ISSUER: Lazy<String> = Lazy::new(|| {
    let issuer = env::var(KEY_TO_TOTP_ISSUER).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_TOTP_ISSUER
        )
    });
    if issuer.contains(':') {
        panic!("TOTP_ISSUER must not contain \":\": {}", issuer);
    }
    issuer
});

fn create_totp(account_id: i64, base32_encoded_secret: String) -> Result<TOTP, ErrResp> {
    // 1. Google Authenticatorの実装に合わせた値
    // 2. rfc-6238の推奨値
    // の優先順位順にパラメータを決定した
    let totp = TOTP::new(
        Algorithm::SHA1,
        PASS_CODE_DIGITS,
        SKEW,
        ONE_STEP_IN_SECOND,
        Secret::Encoded(base32_encoded_secret).to_bytes().unwrap(),
        Some(TOTP_ISSUER.to_string()),
        account_id.to_string(),
    )
    .map_err(|e| {
        error!("failed to create TOTP: {}", e);
        unexpected_err_resp()
    })?;
    Ok(totp)
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

const MAX_NUM_OF_TEMP_MFA_SECRETS: u64 = 8;

#[derive(Clone)]
struct TempMfaSecret {
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
    let totp = create_totp(account_id, base32_encoded_secret.to_string())?;
    let ts = create_timestamp(current_date_time)?;
    let is_valid = totp.check(pass_code, ts);
    if !is_valid {
        error!(
            "failed to check pass code (account_id: {}, current_date_time: {})",
            account_id, current_date_time
        );
        // TODO: UNAUTHORIZEDとどちらを使うか検討する
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PassCodeDoesNotMatch as u32,
            }),
        ));
    }
    Ok(())
}

fn create_timestamp(current_date_time: &DateTime<FixedOffset>) -> Result<u64, ErrResp> {
    // chronoのタイムスタンプはi64のため、他のタイムスタンプでよく使われるu64に変換する必要がある
    // https://github.com/chronotope/chrono/issues/326
    // 上記によると、chronoのタイムスタンプがi64であるのはUTC 1970年1月1日午前0時より前の時間を表すため。
    // 従って、現代に生きる我々にとってi64の値が負の値になることはなく、u64へのキャストが失敗することはない。
    let chrono_ts = current_date_time.timestamp();
    let ts = u64::try_from(current_date_time.timestamp()).map_err(|e| {
        error!("failed to convert {} to type u64: {}", chrono_ts, e);
        unexpected_err_resp()
    })?;
    Ok(ts)
}

#[cfg(test)]
mod tests {}
