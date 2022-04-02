// Copyright 2021 Ken Miura

use std::{error::Error, fmt::Display};

use once_cell::sync::Lazy;
use regex::Regex;

const UUID_REGEXP: &str = "^[a-zA-Z0-9]{32}$";
static UUID_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(UUID_REGEXP).expect("failed to compile UUID regexp"));

/// Validates UUID format.
pub fn validate_uuid(uuid: &str) -> Result<(), UuidValidationError> {
    if !UUID_RE.is_match(uuid) {
        return Err(UuidValidationError::InvalidFormat {
            invalid_uuid: uuid.to_string(),
        });
    }
    Ok(())
}

/// Error related to [validate_uuid()]
#[derive(Debug)]
pub enum UuidValidationError {
    InvalidFormat { invalid_uuid: String },
}

impl Display for UuidValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UuidValidationError::InvalidFormat { invalid_uuid } => {
                write!(f, "invalid UUID: {}", invalid_uuid)
            }
        }
    }
}

impl Error for UuidValidationError {}

#[cfg(test)]
mod tests {
    use super::{validate_uuid, UuidValidationError};

    #[test]
    fn validate_uuid_returns_ok_if_given_valid_uuid() {
        // digit, lowercase or uppercase with 32 letters
        let valid_uuid = "0123456789abcdefghijKLMNOPQRSTUV";

        let result = validate_uuid(valid_uuid);

        assert!(
            result.is_ok(),
            "valid_uuid: {}, length: {}",
            valid_uuid,
            valid_uuid.len()
        )
    }

    #[test]
    fn validate_uuid_returns_invalid_format_if_given_33_letters() {
        let uuid = "0123456789abcdefghijKLMNOPQRSTUVW";

        let result = validate_uuid(uuid);

        let err = result.expect_err("failed to get Err");
        match err {
            UuidValidationError::InvalidFormat { invalid_uuid } => assert_eq!(
                invalid_uuid, uuid,
                "expect: {}, got: {}",
                invalid_uuid, uuid
            ),
        }
    }

    #[test]
    fn validate_uuid_returns_invalid_format_if_given_31_letters() {
        let uuid = "0123456789abcdefghijKLMNOPQRSTU";

        let result = validate_uuid(uuid);

        let err = result.expect_err("failed to get Err");
        match err {
            UuidValidationError::InvalidFormat { invalid_uuid } => assert_eq!(
                invalid_uuid, uuid,
                "expect: {}, got: {}",
                invalid_uuid, uuid
            ),
        }
    }

    #[test]
    fn validate_uuid_returns_invalid_format_if_given_symbol_1() {
        let uuid = "01234567-89abcdef-ghijKLMN-OPQRSTUV";

        let result = validate_uuid(uuid);

        let err = result.expect_err("failed to get Err");
        match err {
            UuidValidationError::InvalidFormat { invalid_uuid } => assert_eq!(
                invalid_uuid, uuid,
                "expect: {}, got: {}",
                invalid_uuid, uuid
            ),
        }
    }

    #[test]
    fn validate_uuid_returns_invalid_format_if_given_symbol_2() {
        let uuid = "0123456789!#$%&'()=~0123456789<>";

        let result = validate_uuid(uuid);

        let err = result.expect_err("failed to get Err");
        match err {
            UuidValidationError::InvalidFormat { invalid_uuid } => assert_eq!(
                invalid_uuid, uuid,
                "expect: {}, got: {}",
                invalid_uuid, uuid
            ),
        }
    }
}
