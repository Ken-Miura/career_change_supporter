// Copyright 2021 Ken Miura

pub mod validator;

use std::convert::From;
use std::env::var;
use std::error::Error;
use std::fmt::Display;
use std::string::FromUtf8Error;

use bcrypt::BcryptError;

// TODO: リリース前に値を調整する (パスワードのストレッチングが2^BCRYPT_COST回実行される)
const BCRYPT_COST: u32 = 7;

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
    use super::*;
    use validator::validate_password;

    #[test]
    fn handle_password_match_case() {
        let password = "0123456789abcdefghijKLMNOPQR@<>.";
        let _ = validate_password(password).expect("failed to get Ok");

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
        let _ = validate_password(password1).expect("failed to get Ok");
        let _ = validate_password(password2).expect("failed to get Ok");

        let hashed_pwd = hash_password(password1).expect("failed to get Ok");
        let result = is_password_match(password2, &hashed_pwd).expect("failed to get Ok");

        assert!(
            !result,
            "password1: {}, hashed password: {:?}, password2: {}",
            password1, hashed_pwd, password2
        );
    }
}

/// 入力された環境変数が定義されているかチェックする<br>
/// <br>
/// 入力された環境変数がすべて定義されている場合、Okを返す。<br>
/// 入力された環境変数の内、どれか一つでも定義されていなければ、Errを返す。
/// Errには定義されていない環境変数を含む。
pub fn check_env_vars(env_vars: Vec<String>) -> Result<(), Vec<String>> {
    let not_found_vars = env_vars
        .iter()
        .map(|env_var| (env_var.clone(), var(env_var)))
        .filter(|env_var_and_result| env_var_and_result.1.is_err())
        .map(|env_var_and_err| env_var_and_err.0)
        .collect::<Vec<String>>();
    if !not_found_vars.is_empty() {
        return Err(not_found_vars);
    }
    Ok(())
}