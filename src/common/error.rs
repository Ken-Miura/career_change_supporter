// Copyright 2021 Ken Miura

pub(crate) mod code;

use serde::Serialize;

pub(crate) const INTERNAL_SERVER_ERROR_MESSAGE: &str =
    "サーバでエラーが発生しました。一定時間後、再度お試しください。";

#[derive(Serialize)]
pub(crate) struct Error {
    pub code: u32,
    pub message: String,
}

pub(crate) trait ToCode {
    fn to_code(&self) -> u32;
}

pub(crate) trait ToMessage {
    fn to_message(&self) -> String;
}

/// エラーの種類ごとに返すステータスコードが異なる場合に利用する
pub(crate) trait ToStatusCode {
    fn to_status_code(&self) -> actix_web::http::StatusCode;
}
