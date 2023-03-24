// Copyright 2023 Ken Miura

use std::env;

use common::ErrResp;
use once_cell::sync::Lazy;
use totp_rs::{Algorithm, Secret, TOTP};
use tracing::error;

use crate::err::unexpected_err_resp;

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
