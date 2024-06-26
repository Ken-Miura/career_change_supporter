// Copyright 2023 Ken Miura

use std::{error::Error, fmt::Display, string::FromUtf8Error};

use bcrypt::BcryptError;

use crate::BCRYPT_COST;

/// Hash given password string.
pub fn hash_password(password: &str) -> Result<Vec<u8>, PasswordHandlingError> {
    let hashed_pwd_str = bcrypt::hash(password, BCRYPT_COST)?;
    let binary = hashed_pwd_str.as_bytes();
    Ok(Vec::from(binary))
}

/// Check if password given matches hashed one.
pub fn is_password_match(
    password: &str,
    hashed_password: &[u8],
) -> Result<bool, PasswordHandlingError> {
    let hashed_pwd_str = String::from_utf8(Vec::from(hashed_password))?;
    let is_match = bcrypt::verify(password, &hashed_pwd_str)?;
    Ok(is_match)
}

impl From<BcryptError> for PasswordHandlingError {
    fn from(e: BcryptError) -> Self {
        PasswordHandlingError::UnexpectedError(Box::new(e))
    }
}

impl From<FromUtf8Error> for PasswordHandlingError {
    fn from(e: FromUtf8Error) -> Self {
        PasswordHandlingError::UnexpectedError(Box::new(e))
    }
}

/// Error related to [hash_password()] and [is_password_match()]
#[derive(Debug)]
pub enum PasswordHandlingError {
    UnexpectedError(Box<dyn Error + Send + Sync + 'static>),
}

impl Display for PasswordHandlingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PasswordHandlingError::UnexpectedError(e) => {
                write!(f, "failed to handle password: {}", e)
            }
        }
    }
}

impl Error for PasswordHandlingError {}

#[cfg(test)]
mod tests {
    use crate::util::validator::password_validator::validate_password;

    use super::*;

    #[test]
    fn handle_password_match_case() {
        let password = "0123456789abcdefghijKLMNOPQR@<>.";
        validate_password(password).expect("failed to get Ok");

        let hashed_pwd = hash_password(password).expect("failed to get Ok");
        let result = is_password_match(password, &hashed_pwd).expect("failed to get Ok");

        assert!(
            result,
            "password: {}, hashed password: {:?}",
            password, hashed_pwd
        );
    }

    #[test]
    fn handle_password_non_match_case() {
        let password1 = "0123456789abcdefghijKLMNOPQR@<>.";
        let password2 = "abcdefghi0123456789";
        validate_password(password1).expect("failed to get Ok");
        validate_password(password2).expect("failed to get Ok");

        let hashed_pwd = hash_password(password1).expect("failed to get Ok");
        let result = is_password_match(password2, &hashed_pwd).expect("failed to get Ok");

        assert!(
            !result,
            "password1: {}, hashed password: {:?}, password2: {}",
            password1, hashed_pwd, password2
        );
    }
}
