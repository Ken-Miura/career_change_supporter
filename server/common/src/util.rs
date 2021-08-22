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

#[cfg(test)]
mod tests {
    use super::*;
    use validator::validate_password;

    #[test]
    fn handle_password_match_case () {
        let password = "0123456789abcdefghijKLMNOPQR@<>.";
        let _ = validate_password(password).expect("failed to get Ok");

        let hashed_pwd = hash_password(password).expect("failed to get Ok");
        let result = is_password_match(password, &hashed_pwd).expect("failed to get Ok");

        assert!(result, "password: {}, hashed password: {}", password, hashed_pwd);
    }

    #[test]
    fn handle_password_non_match_case () {
        let password1 = "0123456789abcdefghijKLMNOPQR@<>.";
        let password2 = "abcdefghi0123456789";
        let _ = validate_password(password1).expect("failed to get Ok");
        let _ = validate_password(password2).expect("failed to get Ok");

        let hashed_pwd = hash_password(password1).expect("failed to get Ok");
        let result = is_password_match(password2, &hashed_pwd).expect("failed to get Ok");

        assert!(!result, "password1: {}, hashed password: {}, password2: {}", password1, hashed_pwd, password2);
    }
}
