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
        return Err(EmailAddressValidationError::InvalidCharacter {
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
    InvalidCharacter {
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
            EmailAddressValidationError::InvalidCharacter { email_address } => {
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
        return Err(PasswordValidationError::ConstraintViolation);
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
