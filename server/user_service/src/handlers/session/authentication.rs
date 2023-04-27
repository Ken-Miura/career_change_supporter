// Copyright 2023 Ken Miura

pub(crate) mod authenticated_handlers;
pub(crate) mod login;
pub(crate) mod logout;
pub(crate) mod mfa;

use std::env;

use axum::{http::StatusCode, Json};
use chrono::{DateTime, FixedOffset};
use common::{mfa::check_if_pass_code_matches, ApiError, ErrResp, ErrRespStruct};
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionError, TransactionTrait,
};
use once_cell::sync::Lazy;
use tracing::error;

use crate::{
    err::{unexpected_err_resp, Code},
    util::find_user_account_by_user_account_id_with_exclusive_lock,
};

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

fn verify_pass_code(
    account_id: i64,
    base32_encoded_secret: &str,
    issuer: &str,
    current_date_time: &DateTime<FixedOffset>,
    pass_code: &str,
) -> Result<(), ErrResp> {
    let matched = check_if_pass_code_matches(
        account_id,
        base32_encoded_secret,
        issuer,
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
    use crate::err::Code;

    use super::{ensure_mfa_is_enabled, verify_pass_code};

    use axum::http::StatusCode;
    use chrono::TimeZone;
    use common::JAPANESE_TIME_ZONE;

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

    #[test]
    fn verify_pass_code_success() {
        let account_id = 413;
        let base32_encoded_secret = "NKQHIV55R4LJV3MD6YSC4Z4UCMT3NDYD";
        let issuer = "Issuer";
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 4, 5, 0, 1, 7)
            .unwrap();
        // 上記のbase32_encoded_secretとcurrent_date_timeでGoogle Authenticatorが実際に算出した値
        let pass_code = "540940";

        verify_pass_code(
            account_id,
            base32_encoded_secret,
            issuer,
            &current_date_time,
            pass_code,
        )
        .expect("failed to get Ok");
    }

    #[test]
    fn verify_pass_code_success_one_step_after() {
        let account_id = 413;
        let base32_encoded_secret = "NKQHIV55R4LJV3MD6YSC4Z4UCMT3NDYD";
        let issuer = "Issuer";
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(
                2023,
                4,
                5,
                0,
                1,
                7 + 30, /* 30s = 本サービスにおける1ステップ */
            )
            .unwrap();
        // 上記のbase32_encoded_secretとcurrent_date_timeでGoogle Authenticatorが実際に算出した値
        let pass_code = "540940";

        verify_pass_code(
            account_id,
            base32_encoded_secret,
            issuer,
            &current_date_time,
            pass_code,
        )
        .expect("failed to get Ok");
    }

    #[test]
    fn verify_pass_code_fail_two_step_after() {
        let account_id = 413;
        let base32_encoded_secret = "NKQHIV55R4LJV3MD6YSC4Z4UCMT3NDYD";
        let issuer = "Issuer";
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(
                2023, 4, 5, 0,
                2, /* 30s = 本サービスにおける1ステップ。2ステップ = 1m */
                7,
            )
            .unwrap();
        // 上記のbase32_encoded_secretとcurrent_date_timeでGoogle Authenticatorが実際に算出した値
        let pass_code = "540940";

        let result = verify_pass_code(
            account_id,
            base32_encoded_secret,
            issuer,
            &current_date_time,
            pass_code,
        )
        .expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1.code, Code::PassCodeDoesNotMatch as u32);
    }

    #[test]
    fn verify_pass_code_fail_incorrect_pass_code() {
        let account_id = 413;
        let base32_encoded_secret = "NKQHIV55R4LJV3MD6YSC4Z4UCMT3NDYD";
        let issuer = "Issuer";
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 4, 5, 0, 1, 7)
            .unwrap();
        let pass_code = "123456";

        let result = verify_pass_code(
            account_id,
            base32_encoded_secret,
            issuer,
            &current_date_time,
            pass_code,
        )
        .expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1.code, Code::PassCodeDoesNotMatch as u32);
    }
}
