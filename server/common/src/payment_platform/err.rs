// Copyright 2021 Ken Miura

use std::error::Error as StdError;
use std::fmt::Display;

use serde::Deserialize;

/// API呼び出しに関連した失敗が起こった場合に返却されるenum
#[derive(Debug)]
pub enum Error {
    /// リクエストとレスポンスを処理する際に発生するエラー
    RequestProcessingError(Box<dyn StdError + Send + Sync>),
    /// API呼び出しの結果として返却されたエラー
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

/// PAY.JPのエラー <https://pay.jp/docs/api/?shell#error> を表す構造体
#[derive(Deserialize, Debug)]
pub struct ErrorInfo {
    pub error: ErrorDetail,
}

impl Display for ErrorInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {{ {} }}", self.error)
    }
}

impl StdError for ErrorInfo {}

/// [ErrorInfo] 内に一つ保持される構造体
#[derive(Deserialize, Debug)]
pub struct ErrorDetail {
    #[serde(rename = "message")]
    pub error_message: String,
    #[serde(rename = "status")]
    pub status_code: u32,
    #[serde(rename = "type")]
    pub error_type: String,
    #[serde(rename = "code")]
    pub error_code: Option<String>,
    #[serde(rename = "param")]
    pub param: Option<String>,
    #[serde(rename = "charge")]
    pub charge_id: Option<String>,
}

impl Display for ErrorDetail {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_code = match self.error_code.clone() {
            Some(s) => s,
            None => "null".to_string(),
        };
        let param = match self.param.clone() {
            Some(s) => s,
            None => "null".to_string(),
        };
        let charge_id = match self.charge_id.clone() {
            Some(s) => s,
            None => "null".to_string(),
        };
        write!(
            f,
            "message: {}, status: {}, type: {}, code: {}, param: {}, charge: {}",
            err_code, self.error_message, param, self.status_code, self.error_type, charge_id
        )
    }
}
