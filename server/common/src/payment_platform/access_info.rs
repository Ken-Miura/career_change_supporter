// Copyright 2021 Ken Miura

use std::{error::Error, fmt::Display};

/// PAY.JP APIにアクセスするための情報を保持する構造体
#[derive(Debug, Clone)]

pub struct AccessInfo {
    base_url: String,
    username: String,
    password: String,
}

impl AccessInfo {
    /// PAY.JP APIにアクセスするための情報を保持する構造体を返却する
    ///
    /// # Arguments
    /// * `base_url` - パスを含まないPAY.JP APIのURL (FQDNの後の"/"も含まない)。基本的に<https://api.pay.jp>を渡す。テスト用のエンドポイントが用意された際、そちらを利用できるようにパラメータとして用意。
    /// * `username` - PAY.JP APIにアクセスするためのユーザー名
    /// * `password` - PAY.JP APIにアクセスするためのパスワード
    ///
    /// # Errors
    /// base_urlが下記の場合、InvalidParamError::BaseUrlを返す
    /// <ul>
    ///   <li>base_urlが空の場合</li>
    ///   <li>base_urlがhttpsで始まっていない場合</li>
    ///   <li>base_urlが/で終わっている場合</li>
    /// </ul>
    /// usernameが空の場合、InvalidParamError::Usernameを返す<br>
    /// passwordが空の場合、InvalidParamError::Passwordを返す<br>
    pub fn new(
        base_url: String,
        username: String,
        password: String,
    ) -> Result<Self, InvalidParamError> {
        if base_url.is_empty() {
            return Err(InvalidParamError::BaseUrl(
                "Empty url is not allowed".to_string(),
            ));
        }
        if username.is_empty() {
            return Err(InvalidParamError::Username(
                "Empty username is not allowed".to_string(),
            ));
        }
        if password.is_empty() {
            return Err(InvalidParamError::Password(
                "Empty password is not allowed".to_string(),
            ));
        }
        if !base_url.starts_with("https://") {
            return Err(InvalidParamError::BaseUrl(format!(
                "Schemes other than 'https://' are not allowed: {}",
                base_url
            )));
        }
        if base_url.ends_with('/') {
            return Err(InvalidParamError::BaseUrl(
                "Trailing slash is not allowed".to_string(),
            ));
        }
        Ok(Self {
            base_url,
            username,
            password,
        })
    }

    /// PAY.JP APIにアクセスするためのURLを返す。最後に"/"は含まない (ex. <https://api.pay.jp>)
    pub fn base_url(&self) -> String {
        self.base_url.clone()
    }

    /// PAY.JP APIにアクセスするためのユーザー名を返す
    pub fn username(&self) -> String {
        self.username.clone()
    }

    /// PAY.JP APIにアクセスするためのパスワードを返す
    pub fn password(&self) -> String {
        self.password.clone()
    }
}

/// [AccessInfo] 生成時に返却される可能性のあるエラー
#[derive(Debug)]
pub enum InvalidParamError {
    BaseUrl(String),
    Username(String),
    Password(String),
}

impl Display for InvalidParamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidParamError::BaseUrl(s) => {
                write!(f, "InvalidParamError::BaseUrl: {}", s)
            }
            InvalidParamError::Username(s) => write!(f, "InvalidParamError::Username: {}", s),
            InvalidParamError::Password(s) => write!(f, "InvalidParamError::Password: {}", s),
        }
    }
}

impl Error for InvalidParamError {}

#[cfg(test)]
mod tests {
    use crate::payment_platform::access_info::InvalidParamError;

    use super::AccessInfo;

    #[test]
    fn new_success() {
        let base_url = String::from("https://api.pay.jp");
        let username = String::from("test_user");
        let password = String::from("test_password");

        let result = AccessInfo::new(base_url.clone(), username.clone(), password.clone());

        let access_info = result.expect("failed to get Ok");
        assert_eq!(base_url, access_info.base_url());
        assert_eq!(username, access_info.username());
        assert_eq!(password, access_info.password());
    }

    #[test]
    fn new_fail_empty_url() {
        let base_url = String::from("");
        let username = String::from("test_user");
        let password = String::from("test_password");

        let result = AccessInfo::new(base_url.clone(), username.clone(), password.clone());

        let invalid_param_err = result.expect_err("failed to get Err");
        match invalid_param_err {
            InvalidParamError::BaseUrl(_) => { /* pass test */ }
            InvalidParamError::Username(_) => panic!("Username"),
            InvalidParamError::Password(_) => panic!("Password"),
        }
    }

    #[test]
    fn new_fail_not_https() {
        let base_url = String::from("http://api.pay.jp");
        let username = String::from("test_user");
        let password = String::from("test_password");

        let result = AccessInfo::new(base_url.clone(), username.clone(), password.clone());

        let invalid_param_err = result.expect_err("failed to get Err");
        match invalid_param_err {
            InvalidParamError::BaseUrl(_) => { /* pass test */ }
            InvalidParamError::Username(_) => panic!("Username"),
            InvalidParamError::Password(_) => panic!("Password"),
        }
    }

    #[test]
    fn new_fail_trailing_slash_exists() {
        let base_url = String::from("https://api.pay.jp/");
        let username = String::from("test_user");
        let password = String::from("test_password");

        let result = AccessInfo::new(base_url.clone(), username.clone(), password.clone());

        let invalid_param_err = result.expect_err("failed to get Err");
        match invalid_param_err {
            InvalidParamError::BaseUrl(_) => { /* pass test */ }
            InvalidParamError::Username(_) => panic!("Username"),
            InvalidParamError::Password(_) => panic!("Password"),
        }
    }

    #[test]
    fn new_fail_empty_username() {
        let base_url = String::from("https://api.pay.jp");
        let username = String::from("");
        let password = String::from("test_password");

        let result = AccessInfo::new(base_url.clone(), username.clone(), password.clone());

        let invalid_param_err = result.expect_err("failed to get Err");
        match invalid_param_err {
            InvalidParamError::BaseUrl(_) => panic!("BaseUrl"),
            InvalidParamError::Username(_) => { /* pass test */ }
            InvalidParamError::Password(_) => panic!("Password"),
        }
    }

    #[test]
    fn new_fail_empty_password() {
        let base_url = String::from("https://api.pay.jp");
        let username = String::from("test_user");
        let password = String::from("");

        let result = AccessInfo::new(base_url.clone(), username.clone(), password.clone());

        let invalid_param_err = result.expect_err("failed to get Err");
        match invalid_param_err {
            InvalidParamError::BaseUrl(_) => panic!("BaseUrl"),
            InvalidParamError::Username(_) => panic!("Username"),
            InvalidParamError::Password(_) => { /* pass test */ }
        }
    }
}
