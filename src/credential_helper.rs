// Copyright 2021 Ken Miura

use once_cell::unsync::Lazy;
use ring::hmac;

// TODO: Consider and change KEY
const PASSWORD_HASH_KEY: [u8; 4] = [0, 1, 2, 3];

static KEY: Lazy<hmac::Key> = Lazy::new(|| hmac::Key::new(hmac::HMAC_SHA512, &PASSWORD_HASH_KEY));

pub(crate) fn hash_password(password: &str) -> &[u8] {
    hmac::sign(&KEY, password.as_bytes()).as_ref()
}

pub(crate) fn verify_password(password: &str, hashed_password: &[u8]) -> Result<(), VerificationError> {
    let _ = hmac::verify(&KEY, password.as_bytes(), hashed_password)?;
    Ok(())
}

enum VerificationError {
    PasswordNotMatch(ring::error::Unspecified),
}

impl From<ring::error::Unspecified> for VerificationError {
    fn from(e: ring::error::Unspecified) -> Self {
        VerificationError::PasswordNotMatch(e)
    }
}
