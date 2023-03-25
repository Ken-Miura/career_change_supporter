// Copyright 2023 Ken Miura

use std::{error::Error, fmt::Display};

use once_cell::sync::Lazy;
use regex::Regex;

const PASS_CODE_REGEXP: &str = "^[0-9]{6}$";
static PASS_CODE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(PASS_CODE_REGEXP).expect("failed to compile PASS_CODE regexp"));

/// Validates pass code format.
pub fn validate_pass_code(pass_code: &str) -> Result<(), PassCodeValidationError> {
    if !PASS_CODE_RE.is_match(pass_code) {
        return Err(PassCodeValidationError::InvalidFormat {
            invalid_pass_code: pass_code.to_string(),
        });
    }
    Ok(())
}

/// Error related to [validate_pass_code()]
#[derive(Debug)]
pub enum PassCodeValidationError {
    InvalidFormat { invalid_pass_code: String },
}

impl Display for PassCodeValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PassCodeValidationError::InvalidFormat { invalid_pass_code } => {
                write!(f, "invalid pass code: {}", invalid_pass_code)
            }
        }
    }
}

impl Error for PassCodeValidationError {}

#[cfg(test)]
mod tests {
    use super::{validate_pass_code, PassCodeValidationError};

    #[test]
    fn validate_pass_code_returns_ok_if_given_valid_pass_code() {
        // 6 digit number
        let valid_pass_code = "123456";

        let result = validate_pass_code(valid_pass_code);

        assert!(
            result.is_ok(),
            "valid_pass_code: {}, length: {}",
            valid_pass_code,
            valid_pass_code.len()
        )
    }

    #[test]
    fn validate_pass_code_returns_invalid_format_if_given_5_digit_number() {
        let pass_code = "12345";

        let result = validate_pass_code(pass_code);

        let err = result.expect_err("failed to get Err");
        match err {
            PassCodeValidationError::InvalidFormat { invalid_pass_code } => assert_eq!(
                invalid_pass_code, pass_code,
                "expect: {}, got: {}",
                invalid_pass_code, pass_code
            ),
        }
    }

    #[test]
    fn validate_pass_code_returns_invalid_format_if_given_7_digit_number() {
        let pass_code = "1234567";

        let result = validate_pass_code(pass_code);

        let err = result.expect_err("failed to get Err");
        match err {
            PassCodeValidationError::InvalidFormat { invalid_pass_code } => assert_eq!(
                invalid_pass_code, pass_code,
                "expect: {}, got: {}",
                invalid_pass_code, pass_code
            ),
        }
    }

    #[test]
    fn validate_pass_code_returns_invalid_format_if_pass_code_has_symbol() {
        let pass_code = "!#$%&(";

        let result = validate_pass_code(pass_code);

        let err = result.expect_err("failed to get Err");
        match err {
            PassCodeValidationError::InvalidFormat { invalid_pass_code } => assert_eq!(
                invalid_pass_code, pass_code,
                "expect: {}, got: {}",
                invalid_pass_code, pass_code
            ),
        }
    }

    #[test]
    fn validate_pass_code_returns_invalid_format_if_pass_code_has_alpabet() {
        let pass_code = "abcDEF";

        let result = validate_pass_code(pass_code);

        let err = result.expect_err("failed to get Err");
        match err {
            PassCodeValidationError::InvalidFormat { invalid_pass_code } => assert_eq!(
                invalid_pass_code, pass_code,
                "expect: {}, got: {}",
                invalid_pass_code, pass_code
            ),
        }
    }
}
