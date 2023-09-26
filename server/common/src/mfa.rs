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
    let qr_code = totp.get_qr_base64().map_err(|e| {
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
    fn handle_pass_code_not_match_case_two_step_before_case() {
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

        let current_date_time = pass_code_submission_date_time
            + Duration::seconds(ONE_STEP_IN_SECOND as i64)
            + Duration::seconds(ONE_STEP_IN_SECOND as i64);
        let result = check_if_pass_code_matches(
            account_id,
            base32_encoded_secret,
            issuer,
            &current_date_time,
            pass_code.as_str(),
        )
        .expect("failed to get Ok");

        assert!(!result);
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
