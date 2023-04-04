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
    use chrono::{Duration, TimeZone};

    use crate::{
        mfa::{hash_recovery_code, is_recovery_code_match, ONE_STEP_IN_SECOND},
        util::validator::uuid_validator::validate_uuid,
        JAPANESE_TIME_ZONE,
    };

    use super::{
        check_if_pass_code_matches, create_totp, generate_base32_encoded_secret,
        generate_base64_encoded_qr_code,
    };

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
    fn handle_pass_code_match_case_one_step_before_case() {
        let account_id = 413;
        let base32_encoded_secret = "7GRCVBFZ73L6NM5VTBKN7SBS4652NTIK";
        let issuer = "Issuer";
        let pass_code_submission_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 4, 3, 14, 5, 59)
            .unwrap();
        let totp =
            create_totp(account_id, base32_encoded_secret, issuer).expect("failed to get Ok");
        let pass_code = totp.generate(
            u64::try_from(pass_code_submission_date_time.timestamp()).expect("failed to get Ok"),
        );

        let current_date_time =
            pass_code_submission_date_time + Duration::seconds(ONE_STEP_IN_SECOND as i64);
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

    // QRコードがどのように生成されるのか仕様を把握していないため、実際に生成されたものを期待値として準備しておく。
    // そして、今後のアップデートで過去の出力と異なるものが出力されていないか確認する回帰テストとしてのテストを行う。
    #[test]
    fn test_generate_base64_encoded_qr_code() {
        let account_id = 413;
        let base32_encoded_secret = "HU7YU2643SZJMWFW5MUOMWNMHSGLA3S6";
        let issuer = "Issuer";

        let result = generate_base64_encoded_qr_code(account_id, base32_encoded_secret, issuer)
            .expect("failed to get Ok");

        let expected = "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";
        assert_eq!(expected, result);
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
