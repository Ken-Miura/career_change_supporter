// Copyright 2021 Ken Miura

use std::{error::Error, fmt::Display};

use once_cell::sync::Lazy;
use regex::Regex;

const PASSWORD_MIN_LENGTH: usize = 10;
const PASSWORD_MAX_LENGTH: usize = 32;
// ASCIIコード表の!(0x21)から~(0x7e)までに存在する文字、かつ10以上32以下
const PASSWORD_REGEXP: &str = r"^[!-~]{10,32}$";
const UPPERCASE_REGEXP: &str = r".*[A-Z].*";
const LOWERCASE_REGEXP: &str = r".*[a-z].*";
const NUMBER_REGEXP: &str = r".*[0-9].*";
const SYMBOL_REGEXP: &str = r".*[!-/:-@\[-`{-~].*";
const CONSTRAINTS_OF_NUM_OF_COMBINATION: u32 = 2;

static PWD_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(PASSWORD_REGEXP)
        .expect("failed to compile password (characters allowed in password) regexp")
});
static UPPERCASE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(UPPERCASE_REGEXP).expect("failed to compile uppercase regexp"));
static LOWERCASE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(LOWERCASE_REGEXP).expect("failed to compile lowercase regexp"));
static NUMBER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(NUMBER_REGEXP).expect("failed to compile number regexp"));
static SYMBOL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(SYMBOL_REGEXP).expect("failed to compile symbol regexp"));

/// Validates password.<br>
/// password requirements<br>
/// - 長さが10文字以上32文字以下
/// - 使える文字は半角英数字と記号 (ASCIIコードの0x21-0x7e)
/// - 大文字、小文字、数字、記号のいずれか二種類以上を組み合わせる必要がある
pub fn validate_password(password: &str) -> Result<(), PasswordValidationError> {
    let pwd_length = password.len();
    if !(PASSWORD_MIN_LENGTH..=PASSWORD_MAX_LENGTH).contains(&pwd_length) {
        return Err(PasswordValidationError::InvalidLength {
            min_length: PASSWORD_MIN_LENGTH,
            max_length: PASSWORD_MAX_LENGTH,
        });
    }
    if !PWD_RE.is_match(password) {
        return Err(PasswordValidationError::InvalidCharacter);
    }
    let _ = validate_password_constraints(password)?;
    Ok(())
}

fn validate_password_constraints(pwd: &str) -> Result<(), PasswordValidationError> {
    let mut count = 0;
    if UPPERCASE_RE.is_match(pwd) {
        count += 1;
    }
    if LOWERCASE_RE.is_match(pwd) {
        count += 1;
    }
    if NUMBER_RE.is_match(pwd) {
        count += 1;
    }
    if SYMBOL_RE.is_match(pwd) {
        count += 1;
    }
    if count < CONSTRAINTS_OF_NUM_OF_COMBINATION {
        return Err(PasswordValidationError::ConstraintViolation);
    }
    Ok(())
}

/// Error related to [validate_password()]
#[derive(Debug)]
pub enum PasswordValidationError {
    InvalidLength {
        min_length: usize,
        max_length: usize,
    },
    InvalidCharacter,
    ConstraintViolation,
}

impl Display for PasswordValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordValidationError::InvalidLength {
                min_length,
                max_length,
            } => write!(
                f,
                "password length must be {} or more, and {} or less",
                min_length, max_length
            ),
            PasswordValidationError::InvalidCharacter => write!(f, "invalid character included"),
            PasswordValidationError::ConstraintViolation => write!(f, "constraint violation (password must contain at least two types of lowercase, uppercase, digit, and symbol)"),
        }
    }
}

impl Error for PasswordValidationError {}

#[cfg(test)]
mod tests {
    use super::{validate_password, PasswordValidationError};

    #[test]
    fn validate_password_returns_ok_if_given_valid_password_1() {
        // 10 letters with lowercase and uppercase
        let valid_password = "aaaaaaaaaaA";

        let result = validate_password(valid_password);

        assert!(
            result.is_ok(),
            "valid_password: {}, length: {}",
            valid_password,
            valid_password.len()
        );
    }

    #[test]
    fn validate_password_returns_ok_if_given_valid_password_2() {
        // 32 letters with lowercase and digit
        let valid_password = "a1234567890123456789012345678901";

        let result = validate_password(valid_password);

        assert!(
            result.is_ok(),
            "valid_password: {}, length: {}",
            valid_password,
            valid_password.len()
        );
    }

    #[test]
    fn validate_password_returns_ok_if_given_valid_password_3() {
        // lowercase and symbol
        let valid_password = "a!\"#$%&'()~-^\\=~|@[`{;:]+*},./?_";

        let result = validate_password(valid_password);

        assert!(
            result.is_ok(),
            "valid_password: {}, length: {}",
            valid_password,
            valid_password.len()
        );
    }

    #[test]
    fn validate_password_returns_ok_if_given_valid_password_4() {
        // uppercase and symbol
        let valid_password = "Z!\"#$%&'()~-^\\=~|@[`{;:]+*},./?_";

        let result = validate_password(valid_password);

        assert!(
            result.is_ok(),
            "valid_password: {}, length: {}",
            valid_password,
            valid_password.len()
        );
    }

    #[test]
    fn validate_password_returns_ok_if_given_valid_password_5() {
        // uppercase and digit
        let valid_password = "Z0123456789";

        let result = validate_password(valid_password);

        assert!(
            result.is_ok(),
            "valid_password: {}, length: {}",
            valid_password,
            valid_password.len()
        );
    }

    #[test]
    fn validate_password_returns_ok_if_given_valid_password_6() {
        // symbol and digit
        let valid_password = "<>123456789";

        let result = validate_password(valid_password);

        assert!(
            result.is_ok(),
            "valid_password: {}, length: {}",
            valid_password,
            valid_password.len()
        );
    }

    #[test]
    fn validate_password_returns_ok_if_given_valid_password_7() {
        // lowercase, uppercase, symbol and digit
        let valid_password = "bC<>123456789";

        let result = validate_password(valid_password);

        assert!(
            result.is_ok(),
            "valid_password: {}, length: {}",
            valid_password,
            valid_password.len()
        );
    }

    #[test]
    fn validate_password_returns_invalid_length_if_given_9_letters_password() {
        let invalid_password = "a12345678";

        let result = validate_password(invalid_password);

        let err = result.expect_err("failed to get Err");
        match err {
            PasswordValidationError::InvalidLength {
                min_length: _,
                max_length: _,
            } => { /* pass test */ }
            PasswordValidationError::InvalidCharacter => {
                panic!("PasswordValidationError::InvalidCharacter")
            }
            PasswordValidationError::ConstraintViolation => {
                panic!("PasswordValidationError::ConstraintViolation")
            }
        }
    }

    #[test]
    fn validate_password_returns_invalid_length_if_given_33_letters_password() {
        let invalid_password = "01234567890123456789012345678901A";

        let result = validate_password(invalid_password);

        let err = result.expect_err("failed to get Err");
        match err {
            PasswordValidationError::InvalidLength {
                min_length: _,
                max_length: _,
            } => { /* pass test */ }
            PasswordValidationError::InvalidCharacter => {
                panic!("PasswordValidationError::InvalidCharacter")
            }
            PasswordValidationError::ConstraintViolation => {
                panic!("PasswordValidationError::ConstraintViolation")
            }
        }
    }

    #[test]
    fn validate_password_returns_invalid_length_if_given_password_with_invalid_character() {
        // 全角アルファベット、全角数字、日本語
        let invalid_password = "a1_ｂ１２不正な文字";

        let result = validate_password(invalid_password);

        let err = result.expect_err("failed to get Err");
        match err {
            PasswordValidationError::InvalidLength {
                min_length: _,
                max_length: _,
            } => panic!("PasswordValidationError::InvalidLength"),
            PasswordValidationError::InvalidCharacter => { /* pass test */ }
            PasswordValidationError::ConstraintViolation => {
                panic!("PasswordValidationError::ConstraintViolation")
            }
        }
    }

    #[test]
    fn validate_password_returns_invalid_length_if_given_password_only_lowercase() {
        let invalid_password = "eeeeeeeeee";

        let result = validate_password(invalid_password);

        let err = result.expect_err("failed to get Err");
        match err {
            PasswordValidationError::InvalidLength {
                min_length: _,
                max_length: _,
            } => panic!("PasswordValidationError::InvalidLength"),
            PasswordValidationError::InvalidCharacter => {
                panic!("PasswordValidationError::InvalidCharacter")
            }
            PasswordValidationError::ConstraintViolation => { /* pass test */ }
        }
    }

    #[test]
    fn validate_password_returns_invalid_length_if_given_password_only_uppercase() {
        let invalid_password = "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD";

        let result = validate_password(invalid_password);

        let err = result.expect_err("failed to get Err");
        match err {
            PasswordValidationError::InvalidLength {
                min_length: _,
                max_length: _,
            } => panic!("PasswordValidationError::InvalidLength"),
            PasswordValidationError::InvalidCharacter => {
                panic!("PasswordValidationError::InvalidCharacter")
            }
            PasswordValidationError::ConstraintViolation => { /* pass test */ }
        }
    }

    #[test]
    fn validate_password_returns_invalid_length_if_given_password_only_symbol() {
        let invalid_password = "!#$%&'()<>";

        let result = validate_password(invalid_password);

        let err = result.expect_err("failed to get Err");
        match err {
            PasswordValidationError::InvalidLength {
                min_length: _,
                max_length: _,
            } => panic!("PasswordValidationError::InvalidLength"),
            PasswordValidationError::InvalidCharacter => {
                panic!("PasswordValidationError::InvalidCharacter")
            }
            PasswordValidationError::ConstraintViolation => { /* pass test */ }
        }
    }

    #[test]
    fn validate_password_returns_invalid_length_if_given_password_only_digit() {
        let invalid_password = "01234567890123456789012345678901";

        let result = validate_password(invalid_password);

        let err = result.expect_err("failed to get Err");
        match err {
            PasswordValidationError::InvalidLength {
                min_length: _,
                max_length: _,
            } => panic!("PasswordValidationError::InvalidLength"),
            PasswordValidationError::InvalidCharacter => {
                panic!("PasswordValidationError::InvalidCharacter")
            }
            PasswordValidationError::ConstraintViolation => { /* pass test */ }
        }
    }
}
