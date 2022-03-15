// Copyright 2021 Ken Miura
//! [PAY.JP API](https://pay.jp/docs/api/) を利用するためのモジュール群<br>
//! <br>
//! レスポンスのJSONオブジェクトのプロパティに関して、プロパティの値の型は公式のSDKの[Go](https://github.com/payjp/payjp-go)と[Java](https://github.com/payjp/payjp-java)を参考に実装。
//! プロパティが存在するかどうか、そのプロパティの値がnullableかどうかは、公式のSDKの[Node](https://github.com/payjp/payjp-node)のtypescriptの宣言を参照し実装。<br>

pub mod charge;
pub mod customer;
pub mod tenant;
pub mod tenant_transfer;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt::Display;

/// PAY.JP APIのURLを保持する環境変数名
pub const KEY_TO_PAYMENT_PLATFORM_API_URL: &str = "PAYMENT_PLATFORM_API_URL";
/// PAY.JP APIのユーザー名を保持する環境変数名
pub const KEY_TO_PAYMENT_PLATFORM_API_USERNAME: &str = "PAYMENT_PLATFORM_API_USERNAME";
/// PAY.JP APIのパスワードを保持する環境変数名
pub const KEY_TO_PAYMENT_PLATFORM_API_PASSWORD: &str = "PAYMENT_PLATFORM_API_PASSWORD";

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

impl StdError for InvalidParamError {}

/// [listオブジェクト](https://pay.jp/docs/api/#list%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88) を示す構造体
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct List<T> {
    pub object: String,
    pub has_more: bool,
    pub url: String,
    pub data: Vec<T>,
    pub count: i32,
}

/// [PAY.JP API](https://pay.jp/docs/api/) の操作に関連した失敗を示す列挙型
#[derive(Debug)]
pub enum Error {
    /// リクエストとレスポンスを処理する際に発生するエラー
    RequestProcessingError(Box<dyn StdError + Send + Sync>),
    /// [PAY.JP API](https://pay.jp/docs/api/) の呼び出しの結果として返却されるエラー
    ApiError(ErrorInfo),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::RequestProcessingError(err) => write!(f, "RequestProcessingError: {}", err),
            Error::ApiError(err_info) => write!(f, "ApiError: {}", err_info),
        }
    }
}

impl StdError for Error {}

/// [PAY.JP APIのエラー](https://pay.jp/docs/api/?shell#error) を表す構造体
#[derive(Deserialize, Debug)]
pub struct ErrorInfo {
    pub error: ErrorDetail,
}

impl Display for ErrorInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ error: {} }}", self.error)
    }
}

impl StdError for ErrorInfo {}

/// [ErrorInfo] の一部
#[derive(Deserialize, Debug)]
pub struct ErrorDetail {
    pub message: String,
    pub status: u32,
    pub r#type: String,
    pub code: Option<String>,
    pub param: Option<String>,
    pub charge: Option<String>,
}

impl Display for ErrorDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = match self.code.clone() {
            Some(s) => s,
            None => "null".to_string(),
        };
        let param = match self.param.clone() {
            Some(s) => s,
            None => "null".to_string(),
        };
        let charge = match self.charge.clone() {
            Some(s) => s,
            None => "null".to_string(),
        };
        write!(
            f,
            "{{ message: {}, status: {}, type: {}, code: {}, param: {}, charge: {} }}",
            code, self.message, param, self.status, self.r#type, charge
        )
    }
}

/// [Metadata](https://pay.jp/docs/api/?shell#metadata) を示す型
///
/// 一つのオブジェクトには最大20キーまで保存でき、キーは40文字まで、バリューは500文字までの文字列が設定可能
pub type Metadata = HashMap<String, String>;

#[cfg(test)]
mod tests {
    use super::InvalidParamError;

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

        let result = AccessInfo::new(base_url, username, password);

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

        let result = AccessInfo::new(base_url, username, password);

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

        let result = AccessInfo::new(base_url, username, password);

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

        let result = AccessInfo::new(base_url, username, password);

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

        let result = AccessInfo::new(base_url, username, password);

        let invalid_param_err = result.expect_err("failed to get Err");
        match invalid_param_err {
            InvalidParamError::BaseUrl(_) => panic!("BaseUrl"),
            InvalidParamError::Username(_) => panic!("Username"),
            InvalidParamError::Password(_) => { /* pass test */ }
        }
    }
}
