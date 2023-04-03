// Copyright 2023 Ken Miura

use std::{error::Error, fmt::Display, string::FromUtf8Error};

use axum::http::StatusCode;
use axum::Json;
use bcrypt::BcryptError;
use chrono::{DateTime, FixedOffset};
use totp_rs::{Algorithm, Secret, TOTP};
use tracing::error;

use crate::{err::Code, ApiError, ErrResp, BCRYPT_COST};

const PASS_CODE_DIGITS: usize = 6;
const SKEW: u8 = 1;
const ONE_STEP_IN_SECOND: u64 = 30;

/// Base32でエンコードされた秘密鍵を生成する
pub fn generate_base32_encoded_secret() -> Result<String, ErrResp> {
    let secret = Secret::generate_secret().to_encoded();
    match secret {
        Secret::Raw(raw_secret) => {
            error!("Secret::Raw is unexpected (value: {:?})", raw_secret);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: Code::UnexpectedErr as u32,
                }),
            ))
        }
        Secret::Encoded(base32_encoded_secret) => Ok(base32_encoded_secret),
    }
}

fn create_totp(
    account_id: i64,
    base32_encoded_secret: &str,
    issuer: &str,
) -> Result<TOTP, ErrResp> {
    // 1. Google Authenticatorの実装に合わせた値
    // 2. rfc-6238の推奨値
    // の優先順位順にパラメータを決定した
    let totp = TOTP::new(
        Algorithm::SHA1,
        PASS_CODE_DIGITS,
        SKEW,
        ONE_STEP_IN_SECOND,
        Secret::Encoded(base32_encoded_secret.to_string())
            .to_bytes()
            .unwrap(),
        Some(issuer.to_string()),
        account_id.to_string(),
    )
    .map_err(|e| {
        error!("failed to create TOTP: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    Ok(totp)
}

/// base32_encoded_secretを表すpng画像のQRコードをBase64エンコードされた文字列の形式で生成する
///
/// base32_encoded_secretは[generate_base32_encoded_secret]で生成したものを利用する
pub fn generate_base64_encoded_qr_code(
    account_id: i64,
    base32_encoded_secret: &str,
    issuer: &str,
) -> Result<String, ErrResp> {
    let totp = create_totp(account_id, base32_encoded_secret, issuer)?;
    let qr_code = totp.get_qr().map_err(|e| {
        error!("failed to create QR code (base64 encoded png img): {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    Ok(qr_code)
}

/// pass_codeがbase32_encoded_secretとcurrent_date_timeから生成されるそれと一致するか確認する。
///
/// 一致する場合、trueを返す。一致しない場合、falseを返す。
pub fn check_if_pass_code_matches(
    account_id: i64,
    base32_encoded_secret: &str,
    issuer: &str,
    current_date_time: &DateTime<FixedOffset>,
    pass_code: &str,
) -> Result<bool, ErrResp> {
    let totp = create_totp(account_id, base32_encoded_secret, issuer)?;
    let ts = create_timestamp(current_date_time)?;
    let matched = totp.check(pass_code, ts);
    Ok(matched)
}

fn create_timestamp(current_date_time: &DateTime<FixedOffset>) -> Result<u64, ErrResp> {
    // chronoのタイムスタンプはi64のため、他のタイムスタンプでよく使われるu64に変換する必要がある
    // https://github.com/chronotope/chrono/issues/326
    // 上記によると、chronoのタイムスタンプがi64であるのはUTC 1970年1月1日午前0時より前の時間を表すため。
    // 従って、現代に生きる我々にとってi64の値が負の値になることはなく、u64へのキャストが失敗することはない。
    let chrono_ts = current_date_time.timestamp();
    let ts = u64::try_from(current_date_time.timestamp()).map_err(|e| {
        error!("failed to convert {} to type u64: {}", chrono_ts, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    Ok(ts)
}

/// Hash given recovery code string.
pub fn hash_recovery_code(recovery_code: &str) -> Result<Vec<u8>, RecoveryCodeHandlingError> {
    let hashed_recovery_code = bcrypt::hash(recovery_code, BCRYPT_COST)?;
    let binary = hashed_recovery_code.as_bytes();
    Ok(Vec::from(binary))
}

/// Check if recovery code given matches hashed one.
pub fn is_recovery_code_match(
    recovery_code: &str,
    hashed_recovery_code: &[u8],
) -> Result<bool, RecoveryCodeHandlingError> {
    let hashed_recovery_code_str = String::from_utf8(Vec::from(hashed_recovery_code))?;
    let is_match = bcrypt::verify(recovery_code, &hashed_recovery_code_str)?;
    Ok(is_match)
}

impl From<BcryptError> for RecoveryCodeHandlingError {
    fn from(e: BcryptError) -> Self {
        RecoveryCodeHandlingError::UnexpectedError(Box::new(e))
    }
}

impl From<FromUtf8Error> for RecoveryCodeHandlingError {
    fn from(e: FromUtf8Error) -> Self {
        RecoveryCodeHandlingError::UnexpectedError(Box::new(e))
    }
}

/// Error related to [hash_recovery_code()] and [is_recovery_code_match()]
#[derive(Debug)]
pub enum RecoveryCodeHandlingError {
    UnexpectedError(Box<dyn Error + Send + Sync + 'static>),
}

impl Display for RecoveryCodeHandlingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryCodeHandlingError::UnexpectedError(e) => {
                write!(f, "failed to handle recovery code: {}", e)
            }
        }
    }
}

impl Error for RecoveryCodeHandlingError {}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use crate::{
        mfa::{hash_recovery_code, is_recovery_code_match},
        util::validator::uuid_validator::validate_uuid,
        JAPANESE_TIME_ZONE,
    };

    use super::{check_if_pass_code_matches, create_totp, generate_base32_encoded_secret};

    #[test]
    fn generate_base32_encoded_secret_finish_successfully() {
        // 出力される文字列は、シードを受け付けるパラメータがなく、完全ランダムなため入出力を指定したテストの記述は出来ない
        // ただ、関数の実行にあたって、Errが返されたり、panicが発生したりせず無事に完了することは確かめておく
        let _ = generate_base32_encoded_secret().expect("failed to get Ok");
    }

    #[test]
    fn handle_pass_code_match_case() {
        let account_id = 413;
        let base32_encoded_secret = "7GRCVBFZ73L6NM5VTBKN7SBS4652NTIK";
        let issuer = "Issuer";
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 4, 3, 14, 5, 59)
            .unwrap();
        let totp =
            create_totp(account_id, base32_encoded_secret, issuer).expect("failed to get Ok");
        let pass_code =
            totp.generate(u64::try_from(current_date_time.timestamp()).expect("failed to get Ok"));

        let result = check_if_pass_code_matches(
            account_id,
            base32_encoded_secret,
            issuer,
            &current_date_time,
            pass_code.as_str(),
        )
        .expect("failed to get Ok");

        assert!(result);
    }

    #[test]
    fn handle_pass_code_not_match_case() {
        let account_id = 413;
        let base32_encoded_secret1 = "7GRCVBFZ73L6NM5VTBKN7SBS4652NTIK";
        let base32_encoded_secret2 = "HU7YU2643SZJMWFW5MUOMWNMHSGLA3S6";
        let issuer = "Issuer";
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 4, 3, 14, 5, 59)
            .unwrap();
        let totp =
            create_totp(account_id, base32_encoded_secret1, issuer).expect("failed to get Ok");
        let pass_code =
            totp.generate(u64::try_from(current_date_time.timestamp()).expect("failed to get Ok"));

        let result = check_if_pass_code_matches(
            account_id,
            base32_encoded_secret2,
            issuer,
            &current_date_time,
            pass_code.as_str(),
        )
        .expect("failed to get Ok");

        assert!(!result);
    }

    #[test]
    fn handle_recovery_code_match_case() {
        let recovery_code = "b0ccdbcfc70446e89ff62a3a42bbb153";
        validate_uuid(recovery_code).expect("failed to get Ok");

        let hashed_recovery_code = hash_recovery_code(recovery_code).expect("failed to get Ok");
        let result =
            is_recovery_code_match(recovery_code, &hashed_recovery_code).expect("failed to get Ok");

        assert!(
            result,
            "recovery_code: {}, hashed_recovery_code: {:?}",
            recovery_code, hashed_recovery_code
        );
    }

    #[test]
    fn handle_recovery_code_non_match_case() {
        let recovery_code1 = "b0ccdbcfc70446e89ff62a3a42bbb153";
        let recovery_code2 = "c0a7698276404eb7af1924d57b1844b0";
        validate_uuid(recovery_code1).expect("failed to get Ok");
        validate_uuid(recovery_code1).expect("failed to get Ok");

        let hashed_recovery_code = hash_recovery_code(recovery_code1).expect("failed to get Ok");
        let result = is_recovery_code_match(recovery_code2, &hashed_recovery_code)
            .expect("failed to get Ok");

        assert!(
            !result,
            "recovery_code1: {}, hashed_recovery_code: {:?}, recovery_code2: {}",
            recovery_code1, hashed_recovery_code, recovery_code2
        );
    }
}
