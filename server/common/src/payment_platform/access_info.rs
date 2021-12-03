// Copyright 2021 Ken Miura

use std::{error::Error, fmt::Display};

const API_VERSION_PATH: &str = "/v1";

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
    /// * `url_without_path` - パスを含まないPAY.JP APIのURL (FQDNの後の"/"も含まない)。基本的に<https://api.pay.jp>を渡す。テスト用のエンドポイントが用意された際、そちらを利用できるようにパラメータとして用意。
    /// * `username` - PAY.JP APIにアクセスするためのユーザー名
    /// * `password` - PAY.JP APIにアクセスするためのパスワード
    ///
    /// # Errors
    /// url_without_pathが下記の場合、InvalidParamError::UrlWithoutPathを返す
    /// <ul>
    ///   <li>url_without_pathが空の場合</li>
    ///   <li>url_without_pathがhttpsで始まっていない場合</li>
    ///   <li>url_without_pathが/で終わっている場合</li>
    /// </ul>
    /// usernameが空の場合、InvalidParamError::Usernameを返す<br>
    /// passwordが空の場合、InvalidParamError::Passwordを返す<br>
    pub fn new(
        url_without_path: String,
        username: String,
        password: String,
    ) -> Result<Self, InvalidParamError> {
        if url_without_path.is_empty() {
            return Err(InvalidParamError::UrlWithoutPath(
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
        if !url_without_path.starts_with("https://") {
            return Err(InvalidParamError::UrlWithoutPath(format!(
                "Schemes other than 'https://' are not allowed: {}",
                url_without_path
            )));
        }
        if url_without_path.ends_with("/") {
            return Err(InvalidParamError::UrlWithoutPath(
                "Trailing slash is not allowed".to_string(),
            ));
        }
        Ok(Self {
            base_url: format!("{}{}", url_without_path, API_VERSION_PATH),
            username,
            password,
        })
    }

    /// PAY.JP APIにアクセスするためのURLを返す。バージョンを示すパスも含む。最後に"/"は含まない (ex. <https://api.pay.jp/v1>)
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
    UrlWithoutPath(String),
    Username(String),
    Password(String),
}

impl Display for InvalidParamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidParamError::UrlWithoutPath(s) => {
                write!(f, "InvalidParamError::UrlWithoutPath: {}", s)
            }
            InvalidParamError::Username(s) => write!(f, "InvalidParamError::Username: {}", s),
            InvalidParamError::Password(s) => write!(f, "InvalidParamError::Password: {}", s),
        }
    }
}

impl Error for InvalidParamError {}
