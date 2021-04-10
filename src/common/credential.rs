// Copyright 2021 Ken Miura

use crate::common::error;
use once_cell::sync::Lazy;
use regex::Regex;
use ring::hmac;
use serde::Deserialize;
use std::fmt;

const EMAIL_ADDRESS_MAX_LENGTH: usize = 254;
const EMAIL_ADDRESS_REGEXP: &str = r"^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$";
const PASSWORD_MIN_LENGTH: usize = 10;
const PASSWORD_MAX_LENGTH: usize = 32;
// TODO: パスワード文字に記号を利用して問題ないか検証する
const PASSWORD_REGEXP: &str = r"^[!-~]{10,32}$";
const UPPER_CASE_REGEXP: &str = r".*[A-Z].*";
const LOWER_CASE_REGEXP: &str = r".*[a-z].*";
const NUMBER_REGEXP: &str = r".*[0-9].*";
const SYMBOL_REGEXP: &str = r".*[!-/:-@\[-`{-~].*";
const CONSTRAINTS_OF_NUM_OF_COMBINATION: u32 = 2;
// TODO: Consider and change KEY
const PASSWORD_HASH_KEY: [u8; 4] = [0, 1, 2, 3];

static EMAIL_ADDR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(EMAIL_ADDRESS_REGEXP).expect("never happens panic"));
static PWD_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(PASSWORD_REGEXP).expect("never happens panic"));
static UPPER_CASE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(UPPER_CASE_REGEXP).expect("never happens panic"));
static LOWER_CASE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(LOWER_CASE_REGEXP).expect("never happens panic"));
static NUMBER_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(NUMBER_REGEXP).expect("never happens panic"));
static SYMBOL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(SYMBOL_REGEXP).expect("never happens panic"));
static KEY: Lazy<hmac::Key> = Lazy::new(|| hmac::Key::new(hmac::HMAC_SHA512, &PASSWORD_HASH_KEY));

#[derive(Deserialize)]
pub(crate) struct Credential {
    pub(crate) email_address: String,
    pub(crate) password: String,
}

impl Credential {
    pub(crate) fn validate(&self) -> Result<(), ValidationError> {
        let _ = Credential::validate_email_address(&self.email_address)?;
        let _ = Credential::validate_password(&self.password)?;
        Ok(())
    }

    fn validate_email_address(email_address: &str) -> Result<(), ValidationError> {
        let mail_addr_length = email_address.len();
        if mail_addr_length > EMAIL_ADDRESS_MAX_LENGTH {
            return Err(ValidationError::EmailAddressLength {
                code: error::code::EMAIL_FORMAT_INVALID_LENGTH,
                length: mail_addr_length,
            });
        }
        if !EMAIL_ADDR_RE.is_match(email_address) {
            return Err(ValidationError::EmailAddressFormat {
                code: error::code::EMAIL_FORMAT_INVALID_FORMAT,
                email_address: email_address.to_string(),
            });
        }
        Ok(())
    }

    /// パスワード要件
    /// 10文字以上32文字以下の文字列
    /// 使える文字列は半角英数字と記号 (ASCIIコードの0x21-0x7e)
    /// 大文字、小文字、数字、記号のいずれか二種類以上を組み合わせる必要がある
    fn validate_password(password: &str) -> Result<(), ValidationError> {
        let pwd_length = password.len();
        if pwd_length < PASSWORD_MIN_LENGTH || pwd_length > PASSWORD_MAX_LENGTH {
            return Err(ValidationError::PasswordLength {
                code: error::code::PASSWORD_FORMAT_INVALID_LENGTH,
            });
        }
        if !PWD_RE.is_match(password) {
            return Err(ValidationError::PasswordFormat {
                code: error::code::PASSWORD_FORMAT_INVALID_FORMAT,
            });
        }
        let _ = Credential::validate_password_constraints(password)?;
        Ok(())
    }

    fn validate_password_constraints(pwd: &str) -> Result<(), ValidationError> {
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
            return Err(ValidationError::PasswordConstraintsViolation {
                code: error::code::PASSWORD_FORMAT_CONSTRAINTS_VIOLATION,
            });
        }
        Ok(())
    }
}

pub(crate) enum ValidationError {
    EmailAddressLength { code: u32, length: usize },
    EmailAddressFormat { code: u32, email_address: String },
    // NOTE: パスワード系はセキュリティのために入力情報は保持させない
    PasswordLength { code: u32 },
    PasswordFormat { code: u32 },
    PasswordConstraintsViolation { code: u32 },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::EmailAddressLength { code, length } => {
                write!(
                    f,
                    "invalid email address length (error code: {}): {}",
                    code, length
                )
            }
            ValidationError::EmailAddressFormat {
                code,
                email_address,
            } => {
                write!(
                    f,
                    "invalid email address format (error code: {}): {}",
                    code, email_address
                )
            }
            ValidationError::PasswordLength { code } => {
                write!(f, "invalid password length (error code: {})", code)
            }
            ValidationError::PasswordFormat { code } => {
                write!(f, "invalid password format (error code: {})", code)
            }
            ValidationError::PasswordConstraintsViolation { code } => {
                write!(f, "password constraints vaiolation (error code: {})", code)
            }
        }
    }
}

impl error::Detail for ValidationError {
    fn code(&self) -> u32 {
        match self {
            ValidationError::EmailAddressLength { code, length: _ } => *code,
            ValidationError::EmailAddressFormat {
                code,
                email_address: _,
            } => *code,
            ValidationError::PasswordLength { code } => *code,
            ValidationError::PasswordFormat { code } => *code,
            ValidationError::PasswordConstraintsViolation { code } => *code,
        }
    }
    fn ui_message(&self) -> String {
        match self {
        ValidationError::EmailAddressLength { code: _, length} => { format!("メールアドレスの長さが不正です (入力されたメールアドレスの長さ: {})。メールアドレスは{}文字以下である必要があります。", length, EMAIL_ADDRESS_MAX_LENGTH)},
        ValidationError::EmailAddressFormat { code: _, email_address} => { format!("メールアドレスの形式が不正です (入力されたメールアドレス: {})。\"email.address@example.com\"のような形式で入力してください。", email_address)},
        ValidationError::PasswordLength{ code: _} => {format!("パスワードの長さが不正です。パスワードは{}文字以上、{}文字以下である必要があります。", PASSWORD_MIN_LENGTH, PASSWORD_MAX_LENGTH)},
        ValidationError::PasswordFormat{ code: _}=> {String::from("パスワードに使用できない文字が含まれています。パスワードに使用可能な文字は、半角英数字と記号です。")},
        ValidationError::PasswordConstraintsViolation{ code:_}=> {String::from("不正な形式のパスワードです。パスワードは小文字、大文字、数字または記号の内、2種類以上を組み合わせる必要があります。")},
        }
    }
}

pub(crate) fn hash_password(password: &str) -> Vec<u8> {
    let tag = hmac::sign(&KEY, password.as_bytes());
    let binary = tag.as_ref();
    Vec::from(binary)
}

pub(crate) fn verify_password(
    password: &str,
    hashed_password: &[u8],
) -> Result<(), VerificationError> {
    let _ = hmac::verify(&KEY, password.as_bytes(), hashed_password)?;
    Ok(())
}

pub(crate) enum VerificationError {
    PasswordNotMatch {
        code: u32,
        error: ring::error::Unspecified,
    },
}

impl fmt::Display for VerificationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VerificationError::PasswordNotMatch { code, error } => {
                write!(
                    f,
                    "password doesn't match (error code: code: {}): {}",
                    code, error
                )
            }
        }
    }
}

impl From<ring::error::Unspecified> for VerificationError {
    fn from(e: ring::error::Unspecified) -> Self {
        VerificationError::PasswordNotMatch {
            code: error::code::AUTHENTICATION_FAILED,
            error: e,
        }
    }
}

impl error::Detail for VerificationError {
    fn code(&self) -> u32 {
        match self {
            VerificationError::PasswordNotMatch { code, error: _ } => *code,
        }
    }

    fn ui_message(&self) -> String {
        match self {
            VerificationError::PasswordNotMatch { code: _, error: _ } => {
                String::from("メールアドレス、もしくはパスワードが間違っています。")
            }
        }
    }
}
