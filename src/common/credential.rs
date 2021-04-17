// // Copyright 2021 Ken Miura

use crate::common::error::handled;
use once_cell::sync::Lazy;
use regex::Regex;
use ring::hmac;
use serde::Deserialize;

const EMAIL_ADDRESS_MAX_LENGTH: usize = 254;
const EMAIL_ADDRESS_REGEXP: &str = r"^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$";
const PASSWORD_MIN_LENGTH: usize = 10;
const PASSWORD_MAX_LENGTH: usize = 32;
// // TODO: パスワード文字に記号を利用して問題ないか検証する
const PASSWORD_REGEXP: &str = r"^[!-~]{10,32}$";
const UPPER_CASE_REGEXP: &str = r".*[A-Z].*";
const LOWER_CASE_REGEXP: &str = r".*[a-z].*";
const NUMBER_REGEXP: &str = r".*[0-9].*";
const SYMBOL_REGEXP: &str = r".*[!-/:-@\[-`{-~].*";
const CONSTRAINTS_OF_NUM_OF_COMBINATION: u32 = 2;
// // TODO: Consider and change KEY
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
    pub(crate) fn validate(&self) -> Result<(), handled::Error> {
        let _ = Credential::validate_email_address(&self.email_address)?;
        let _ = Credential::validate_password(&self.password)?;
        Ok(())
    }

    fn validate_email_address(email_address: &str) -> Result<(), handled::Error> {
        let mail_addr_length = email_address.len();
        if mail_addr_length > EMAIL_ADDRESS_MAX_LENGTH {
            let e =
                handled::InvalidEmailAddressLength::new(mail_addr_length, EMAIL_ADDRESS_MAX_LENGTH);
            return Err(handled::Error::InvalidEmailAddressLength(e));
        }
        if !EMAIL_ADDR_RE.is_match(email_address) {
            let e = handled::InvalidEmailAddressFormat::new(email_address.to_string());
            return Err(handled::Error::InvalidEmailAddressFormat(e));
        }
        Ok(())
    }

    /// パスワード要件
    /// 10文字以上32文字以下の文字列
    /// 使える文字列は半角英数字と記号 (ASCIIコードの0x21-0x7e)
    /// 大文字、小文字、数字、記号のいずれか二種類以上を組み合わせる必要がある
    fn validate_password(password: &str) -> Result<(), handled::Error> {
        let pwd_length = password.len();
        if pwd_length < PASSWORD_MIN_LENGTH || pwd_length > PASSWORD_MAX_LENGTH {
            let e = handled::InvalidPasswordLength::new(PASSWORD_MIN_LENGTH, PASSWORD_MAX_LENGTH);
            return Err(handled::Error::InvalidPasswordLength(e));
        }
        if !PWD_RE.is_match(password) {
            let e = handled::InvalidPasswordFormat::new();
            return Err(handled::Error::InvalidPasswordFormat(e));
        }
        let _ = Credential::validate_password_constraints(password)?;
        Ok(())
    }

    fn validate_password_constraints(pwd: &str) -> Result<(), handled::Error> {
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
            let e = handled::PasswordConstraintsViolation::new();
            return Err(handled::Error::PasswordConstraintsViolation(e));
        }
        Ok(())
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
) -> Result<(), handled::Error> {
    let result = hmac::verify(&KEY, password.as_bytes(), hashed_password);
    if let Err(err) = result {
        let e = handled::PasswordNotMatch::new(err);
        return Err(handled::Error::PasswordNotMatch(e));
    }
    Ok(())
}
