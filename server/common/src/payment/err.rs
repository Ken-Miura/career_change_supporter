// Copyright 2021 Ken Miura

use std::error::Error;
use std::fmt::Display;

use serde::Deserialize;

/// PAY.JPのエラー (https://pay.jp/docs/api/?shell#error) を表す構造体
#[derive(Deserialize, Debug)]
pub struct PaymentError {
    pub error: PaymentErrorInner,
}

impl Display for PaymentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {{ {} }}", self.error)
    }
}

impl Error for PaymentError {}

#[derive(Deserialize, Debug)]
pub struct PaymentErrorInner {
    #[serde(rename = "code")]
    pub error_code: Option<String>,
    #[serde(rename = "message")]
    pub error_message: String,
    #[serde(rename = "param")]
    pub param: Option<String>,
    #[serde(rename = "status")]
    pub status_code: i32,
    #[serde(rename = "type")]
    pub error_type: String,
}

impl Display for PaymentErrorInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let err_code = match self.error_code.clone() {
            Some(s) => s,
            None => "null".to_string(),
        };
        let param = match self.param.clone() {
            Some(s) => s,
            None => "null".to_string(),
        };
        write!(
            f,
            "code: {}, message: {}, param: {}, status: {}, type: {}",
            err_code, self.error_message, param, self.status_code, self.error_type
        )
    }
}
