// Copyright 2021 Ken Miura

pub mod validator;

use std::error::Error;
use std::fmt::Display;

// TODO: リリース前に値を調整する (パスワードのストレッチングが2^BCRYPT_COST回実行される)
const BCRYPT_COST: u32 = 7;

/// Hash given password string.
pub fn hash_password(password: &str) -> Result<String, PasswordHandlingError> {
    let result = bcrypt::hash(password, BCRYPT_COST);
    match result {
        Ok(hashed_pwd_str) => Ok(hashed_pwd_str),
        Err(e) => Err(PasswordHandlingError::UnexpectedError(Box::new(e))),
    }
}

/// Check if password given matches hashed one.
pub fn is_password_match(
    password: &str,
    hashed_password: &str,
) -> Result<bool, PasswordHandlingError> {
    let result = bcrypt::verify(password, &hashed_password);
    match result {
        Ok(is_match) => Ok(is_match),
        Err(e) => Err(PasswordHandlingError::UnexpectedError(Box::new(e))),
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
            PasswordHandlingError::UnexpectedError(e) => write!(f, "failed to handle password: {}", e),
        }
    }
}

impl Error for PasswordHandlingError {
}
