// Copyright 2023 Ken Miura

pub(crate) mod setting_change;
pub(crate) mod temp_secret;

use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, FixedOffset};
use common::{ApiError, ErrResp};
use entity::sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect,
};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

const MAX_NUM_OF_TEMP_MFA_SECRETS: u64 = 8;

#[derive(Clone, Debug, PartialEq)]
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

fn extract_first_temp_mfa_secret(
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

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use crate::err::Code;

    use super::{ensure_mfa_is_not_enabled, extract_first_temp_mfa_secret, TempMfaSecret};

    #[test]
    fn extract_first_temp_mfa_secret_empty_case() {
        let temp_secrets = vec![];

        let result = extract_first_temp_mfa_secret(temp_secrets);

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(err_resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(err_resp.1.code, Code::NoTempMfaSecretFound as u32);
    }

    #[test]
    fn extract_first_temp_mfa_secret_1_temp_mfa_secret() {
        let temp_mfa_secret = TempMfaSecret {
            temp_mfa_secret_id: 1,
            base32_encoded_secret: "7GRCVBFZ73L6NM5VTBKN7SBS4652NTIK".to_string(),
        };
        let temp_secrets = vec![temp_mfa_secret.clone()];

        let result = extract_first_temp_mfa_secret(temp_secrets).expect("failed to get Ok");

        assert_eq!(result, temp_mfa_secret);
    }

    #[test]
    fn extract_first_temp_mfa_secret_2_temp_mfa_secrets() {
        let temp_mfa_secret1 = TempMfaSecret {
            temp_mfa_secret_id: 1,
            base32_encoded_secret: "7GRCVBFZ73L6NM5VTBKN7SBS4652NTIK".to_string(),
        };
        let temp_mfa_secret2 = TempMfaSecret {
            temp_mfa_secret_id: 2,
            base32_encoded_secret: "HU7YU2643SZJMWFW5MUOMWNMHSGLA3S6".to_string(),
        };
        let temp_secrets = vec![temp_mfa_secret2.clone(), temp_mfa_secret1];

        let result = extract_first_temp_mfa_secret(temp_secrets).expect("failed to get Ok");

        assert_eq!(result, temp_mfa_secret2);
    }

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
}
