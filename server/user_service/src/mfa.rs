// Copyright 2023 Ken Miura

use std::env;

use axum::{http::StatusCode, Json};
use chrono::{DateTime, FixedOffset};
use common::{mfa::check_if_pass_code_matches, ApiError, ErrResp, ErrRespStruct};
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder,
    QuerySelect, Set, TransactionError, TransactionTrait,
};
use once_cell::sync::Lazy;
use tracing::error;

use crate::{
    err::{unexpected_err_resp, Code},
    util::find_user_account_by_user_account_id_with_exclusive_lock,
};

pub(crate) mod mfa_request;
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

async fn disable_mfa(account_id: i64, pool: &DatabaseConnection) -> Result<(), ErrResp> {
    pool.transaction::<_, (), ErrRespStruct>(|txn| {
        Box::pin(async move {
            let user_model =
                find_user_account_by_user_account_id_with_exclusive_lock(txn, account_id).await?;
            let user_model = user_model.ok_or_else(|| {
                error!(
                    "failed to find user_account (user_account_id: {})",
                    account_id
                );
                ErrRespStruct {
                    err_resp: unexpected_err_resp(),
                }
            })?;

            let _ = entity::mfa_info::Entity::delete_by_id(account_id)
                .exec(txn)
                .await
                .map_err(|e| {
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

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use crate::err::Code;

    use super::{ensure_mfa_is_enabled, ensure_mfa_is_not_enabled};

    #[test]
    fn ensure_mfa_is_not_enabled_success() {
        let mfa_enabled = false;
        ensure_mfa_is_not_enabled(mfa_enabled).expect("failed to get Ok");
    }

    #[test]
    fn ensure_mfa_is_not_enabled_error() {
        let mfa_enabled = true;
        let result = ensure_mfa_is_not_enabled(mfa_enabled).expect_err("failed to get Err");

        assert_eq!(StatusCode::BAD_REQUEST, result.0);
        assert_eq!(Code::MfaHasAlreadyBeenEnabled as u32, result.1.code);
    }

    #[test]
    fn ensure_mfa_is_enabled_success() {
        let mfa_enabled = true;
        ensure_mfa_is_enabled(mfa_enabled).expect("failed to get Ok");
    }

    #[test]
    fn ensure_mfa_is_enabled_error() {
        let mfa_enabled = false;
        let result = ensure_mfa_is_enabled(mfa_enabled).expect_err("failed to get Err");

        assert_eq!(StatusCode::BAD_REQUEST, result.0);
        assert_eq!(Code::MfaIsNotEnabled as u32, result.1.code);
    }
}
