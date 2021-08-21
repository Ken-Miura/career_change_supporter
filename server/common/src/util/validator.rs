// Copyright 2021 Ken Miura

use once_cell::sync::Lazy;
use regex::Regex;
use std::error::Error;
use std::fmt::Display;

const EMAIL_ADDRESS_MIN_LENGTH: usize = 1;
const EMAIL_ADDRESS_MAX_LENGTH: usize = 254;
const EMAIL_ADDRESS_REGEXP: &str = r"^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$";
const PASSWORD_MIN_LENGTH: usize = 10;
const PASSWORD_MAX_LENGTH: usize = 32;
// ASCIIコード表の!(0x21)から~(0x7e)までに存在する文字、かつ10以上32以下
const PASSWORD_REGEXP: &str = r"^[!-~]{10,32}$";
const UPPER_CASE_REGEXP: &str = r".*[A-Z].*";
const LOWER_CASE_REGEXP: &str = r".*[a-z].*";
const NUMBER_REGEXP: &str = r".*[0-9].*";
const SYMBOL_REGEXP: &str = r".*[!-/:-@\[-`{-~].*";
const CONSTRAINTS_OF_NUM_OF_COMBINATION: u32 = 2;
const UUID_REGEXP: &str = "^[a-zA-Z0-9]{32}$";

static EMAIL_ADDR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(EMAIL_ADDRESS_REGEXP).expect("failed to compile email address regexp"));
static PWD_RE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(PASSWORD_REGEXP)
        .expect("failed to compile password (characters allowed in password) regexp")
});
static UPPER_CASE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(UPPER_CASE_REGEXP).expect("failed to compile upper case regexp"));
static LOWER_CASE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(LOWER_CASE_REGEXP).expect("failed to compile lower case regexp"));
static NUMBER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(NUMBER_REGEXP).expect("failed to compile number regexp"));
static SYMBOL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(SYMBOL_REGEXP).expect("failed to compile symbol regexp"));
static UUID_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(UUID_REGEXP).expect("failed to compile UUID regexp"));

/// Validate email address.
pub fn validate_email_address(email_address: &str) -> Result<(), EmailAddressValidationError> {
    let mail_addr_length = email_address.len();
    if !(EMAIL_ADDRESS_MIN_LENGTH..=EMAIL_ADDRESS_MAX_LENGTH).contains(&mail_addr_length) {
        return Err(EmailAddressValidationError::InvalidLength {
            length: mail_addr_length,
            min_length: EMAIL_ADDRESS_MIN_LENGTH,
            max_length: EMAIL_ADDRESS_MAX_LENGTH,
        });
    }
    if !EMAIL_ADDR_RE.is_match(email_address) {
        return Err(EmailAddressValidationError::InvalidFormat {
            email_address: email_address.to_string(),
        });
    }
    Ok(())
}

/// Error related to [validate_email_address()]
#[derive(Debug)]
pub enum EmailAddressValidationError {
    InvalidLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    InvalidFormat {
        email_address: String,
    },
}

impl Display for EmailAddressValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailAddressValidationError::InvalidLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid email address length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            EmailAddressValidationError::InvalidFormat { email_address } => {
                write!(f, "invalid email address format: {}", email_address)
            }
        }
    }
}

impl Error for EmailAddressValidationError {}

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
    if UPPER_CASE_RE.is_match(pwd) {
        count += 1;
    }
    if LOWER_CASE_RE.is_match(pwd) {
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
            PasswordValidationError::ConstraintViolation => write!(f, "constraint violation"),
        }
    }
}

impl Error for PasswordValidationError {}

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
    use core::panic;

    use super::*;

    #[test]
    fn validate_email_address_returns_ok_if_given_valid_email_address_with_various_letters() {
        let valid_email_address =
            "The.quick.brown.fox.jumps.over.the.lazy.dog_0123456789@example.com";

        let result = validate_email_address(valid_email_address);

        assert!(
            result.is_ok(),
            "valid_email_address: {}",
            valid_email_address
        );
    }

    #[test]
    fn validate_email_address_returns_ok_if_given_valid_email_address_of_254_characters() {
        let valid_email_address =
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa@example.com";

        let result = validate_email_address(valid_email_address);

        assert!(
            result.is_ok(),
            "valid_email_address: {}",
            valid_email_address
        );
    }

    #[test]
    fn validate_email_address_returns_invalid_length_if_given_empty_string() {
        let empty_str = "";

        let result = validate_email_address(empty_str);

        let err = result.expect_err("failed to get Err");
        match err {
            EmailAddressValidationError::InvalidLength {
                length,
                min_length: _,
                max_length: _,
            } => assert!(length == 0, "length: {}", length),
            EmailAddressValidationError::InvalidFormat { email_address } => panic!(
                "EmailAddressValidationError::InvalidFormat {{ email_address: \"{}\" }}",
                email_address
            ),
        }
    }

    #[test]
    fn validate_email_address_returns_invalid_length_if_given_255_characters() {
        // valid format email address of 255 characters
        let too_long_email_address = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa@example.com";

        let result = validate_email_address(too_long_email_address);

        let err = result.expect_err("failed to get Err");
        match err {
            EmailAddressValidationError::InvalidLength {
                length,
                min_length: _,
                max_length: _,
            } => assert!(length == 255, "length: {}", length),
            EmailAddressValidationError::InvalidFormat { email_address } => panic!(
                "EmailAddressValidationError::InvalidFormat {{ email_address: \"{}\" }}",
                email_address
            ),
        }
    }

    #[test]
    fn validate_email_address_returns_invalid_format_if_given_one_letter() {
        let one_letter = "a";

        let result = validate_email_address(one_letter);

        let err = result.expect_err("failed to get Err");
        match err {
            EmailAddressValidationError::InvalidLength {
                length,
                min_length: _,
                max_length: _,
            } => panic!(
                "EmailAddressValidationError::InvalidLength: length: {}",
                length
            ),
            EmailAddressValidationError::InvalidFormat { email_address } => {
                assert_eq!(
                    one_letter, &email_address,
                    "email_address: {}",
                    email_address
                )
            }
        }
    }

    #[test]
    fn validate_email_address_returns_invalid_format_if_given_email_address_without_domain() {
        let email_address_without_domain = "test@";

        let result = validate_email_address(email_address_without_domain);

        let err = result.expect_err("failed to get Err");
        match err {
            EmailAddressValidationError::InvalidLength {
                length,
                min_length: _,
                max_length: _,
            } => panic!(
                "EmailAddressValidationError::InvalidLength: length: {}",
                length
            ),
            EmailAddressValidationError::InvalidFormat { email_address } => assert_eq!(
                email_address_without_domain, &email_address,
                "email_address: {}",
                email_address
            ),
        }
    }

    #[test]
    fn validate_email_address_returns_invalid_format_if_given_email_address_without_local_part() {
        let email_address_without_local_part = "@example.com";

        let result = validate_email_address(email_address_without_local_part);

        let err = result.expect_err("failed to get Err");
        match err {
            EmailAddressValidationError::InvalidLength {
                length,
                min_length: _,
                max_length: _,
            } => panic!(
                "EmailAddressValidationError::InvalidLength: length: {}",
                length
            ),
            EmailAddressValidationError::InvalidFormat { email_address } => assert_eq!(
                email_address_without_local_part, &email_address,
                "email_address: {}",
                email_address
            ),
        }
    }

    #[test]
    fn validate_password_returns_ok_if_given_valid_password_1() {
        // 10 letters with lower case and upper case
        let valid_password = "aaaaaaaaaaA";

        let result = validate_password(valid_password);
        
        assert!(result.is_ok(), "valid_password: {}, length: {}", valid_password, valid_password.len());
    }

    #[test]
    fn validate_password_returns_ok_if_given_valid_password_2() {
        // 32 letters with lower case and digit
        let valid_password = "a1234567890123456789012345678901";

        let result = validate_password(valid_password);
        
        assert!(result.is_ok(), "valid_password: {}, length: {}", valid_password, valid_password.len());
    }

    #[test]
    fn validate_password_returns_ok_if_given_valid_password_3() {
        // lower case and symbol
        let valid_password = "a!\"#$%&'()~-^\\=~|@[`{;:]+*},./?_";

        let result = validate_password(valid_password);
        
        assert!(result.is_ok(), "valid_password: {}, length: {}", valid_password, valid_password.len());
    }

    #[test]
    fn validate_password_returns_ok_if_given_valid_password_4() {
        // upper case and symbol
        let valid_password = "Z!\"#$%&'()~-^\\=~|@[`{;:]+*},./?_";

        let result = validate_password(valid_password);
        
        assert!(result.is_ok(), "valid_password: {}, length: {}", valid_password, valid_password.len());
    }

    #[test]
    fn validate_password_returns_ok_if_given_valid_password_5() {
        // upper case and digit
        let valid_password = "Z0123456789";

        let result = validate_password(valid_password);
        
        assert!(result.is_ok(), "valid_password: {}, length: {}", valid_password, valid_password.len());
    }

    #[test]
    fn validate_password_returns_ok_if_given_valid_password_6() {
        // symbol and digit
        let valid_password = "<>123456789";

        let result = validate_password(valid_password);
        
        assert!(result.is_ok(), "valid_password: {}, length: {}", valid_password, valid_password.len());
    }

    #[test]
    fn validate_password_returns_ok_if_given_valid_password_7() {
        // lower case, upper case, symbol and digit
        let valid_password = "bC<>123456789";

        let result = validate_password(valid_password);
        
        assert!(result.is_ok(), "valid_password: {}, length: {}", valid_password, valid_password.len());
    }

    #[test]
    fn validate_password_returns_invalid_length_if_given_9_letters_password() {
        let invalid_password = "a12345678";

        let result = validate_password(invalid_password);
        
        let err = result.expect_err("failed to get Err");
        match err {
            PasswordValidationError::InvalidLength { min_length: _, max_length: _ } => { /* pass test */ },
            PasswordValidationError::InvalidCharacter => panic!("PasswordValidationError::InvalidCharacter"),
            PasswordValidationError::ConstraintViolation => panic!("PasswordValidationError::ConstraintViolation"),
        }
    }

    #[test]
    fn validate_password_returns_invalid_length_if_given_33_letters_password() {
        let invalid_password = "01234567890123456789012345678901A";

        let result = validate_password(invalid_password);
        
        let err = result.expect_err("failed to get Err");
        match err {
            PasswordValidationError::InvalidLength { min_length: _, max_length: _ } => { /* pass test */ },
            PasswordValidationError::InvalidCharacter => panic!("PasswordValidationError::InvalidCharacter"),
            PasswordValidationError::ConstraintViolation => panic!("PasswordValidationError::ConstraintViolation"),
        }
    }

    #[test]
    fn validate_password_returns_invalid_length_if_given_password_with_invalid_character() {
        // 全角アルファベット、全角数字、日本語
        let invalid_password = "a1_ｂ１２不正な文字";

        let result = validate_password(invalid_password);
        
        let err = result.expect_err("failed to get Err");
        match err {
            PasswordValidationError::InvalidLength { min_length: _, max_length: _ } => panic!("PasswordValidationError::InvalidLength"),
            PasswordValidationError::InvalidCharacter => { /* pass test */ },
            PasswordValidationError::ConstraintViolation => panic!("PasswordValidationError::ConstraintViolation"),
        }
    }

    #[test]
    fn validate_password_returns_invalid_length_if_given_password_only_lower_case() {
        let invalid_password = "eeeeeeeeee";

        let result = validate_password(invalid_password);
        
        let err = result.expect_err("failed to get Err");
        match err {
            PasswordValidationError::InvalidLength { min_length: _, max_length: _ } => panic!("PasswordValidationError::InvalidLength"),
            PasswordValidationError::InvalidCharacter => panic!("PasswordValidationError::InvalidCharacter"),
            PasswordValidationError::ConstraintViolation => { /* pass test */ },
        }
    }

    #[test]
    fn validate_password_returns_invalid_length_if_given_password_only_upper_case() {
        let invalid_password = "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD";

        let result = validate_password(invalid_password);
        
        let err = result.expect_err("failed to get Err");
        match err {
            PasswordValidationError::InvalidLength { min_length: _, max_length: _ } => panic!("PasswordValidationError::InvalidLength"),
            PasswordValidationError::InvalidCharacter => panic!("PasswordValidationError::InvalidCharacter"),
            PasswordValidationError::ConstraintViolation => { /* pass test */ },
        }
    }

    #[test]
    fn validate_password_returns_invalid_length_if_given_password_only_symbol() {
        let invalid_password = "!#$%&'()<>";

        let result = validate_password(invalid_password);
        
        let err = result.expect_err("failed to get Err");
        match err {
            PasswordValidationError::InvalidLength { min_length: _, max_length: _ } => panic!("PasswordValidationError::InvalidLength"),
            PasswordValidationError::InvalidCharacter => panic!("PasswordValidationError::InvalidCharacter"),
            PasswordValidationError::ConstraintViolation => { /* pass test */ },
        }
    }

    #[test]
    fn validate_password_returns_invalid_length_if_given_password_only_digit() {
        let invalid_password = "01234567890123456789012345678901";

        let result = validate_password(invalid_password);
        
        let err = result.expect_err("failed to get Err");
        match err {
            PasswordValidationError::InvalidLength { min_length: _, max_length: _ } => panic!("PasswordValidationError::InvalidLength"),
            PasswordValidationError::InvalidCharacter => panic!("PasswordValidationError::InvalidCharacter"),
            PasswordValidationError::ConstraintViolation => { /* pass test */ },
        }
    }

    #[test]
    fn validate_uuid_returns_ok_if_given_valid_uuid() {
        // digit, lowercase or uppercase with 32 letters
        let valid_uuid = "0123456789abcdefghijKLMNOPQRSTUV";

        let result = validate_uuid(valid_uuid);
        
        assert!(result.is_ok(), "valid_uuid: {}, length: {}", valid_uuid, valid_uuid.len())
    }

    #[test]
    fn validate_uuid_returns_invalid_format_if_given_33_letters() {
        let uuid = "0123456789abcdefghijKLMNOPQRSTUVW";

        let result = validate_uuid(uuid);
        
        let err = result.expect_err("failed to get Err");
        match err {
            UuidValidationError::InvalidFormat { invalid_uuid } => assert_eq!(invalid_uuid, uuid, "expect: {}, got: {}", invalid_uuid, uuid),
        }
    }

    #[test]
    fn validate_uuid_returns_invalid_format_if_given_31_letters() {
        let uuid = "0123456789abcdefghijKLMNOPQRSTU";

        let result = validate_uuid(uuid);
        
        let err = result.expect_err("failed to get Err");
        match err {
            UuidValidationError::InvalidFormat { invalid_uuid } => assert_eq!(invalid_uuid, uuid, "expect: {}, got: {}", invalid_uuid, uuid),
        }
    }

    #[test]
    fn validate_uuid_returns_invalid_format_if_given_symbol_1() {
        let uuid = "01234567-89abcdef-ghijKLMN-OPQRSTUV";

        let result = validate_uuid(uuid);
        
        let err = result.expect_err("failed to get Err");
        match err {
            UuidValidationError::InvalidFormat { invalid_uuid } => assert_eq!(invalid_uuid, uuid, "expect: {}, got: {}", invalid_uuid, uuid),
        }
    }

    #[test]
    fn validate_uuid_returns_invalid_format_if_given_symbol_2() {
        let uuid = "0123456789!#$%&'()=~0123456789<>";

        let result = validate_uuid(uuid);
        
        let err = result.expect_err("failed to get Err");
        match err {
            UuidValidationError::InvalidFormat { invalid_uuid } => assert_eq!(invalid_uuid, uuid, "expect: {}, got: {}", invalid_uuid, uuid),
        }
    }
}
