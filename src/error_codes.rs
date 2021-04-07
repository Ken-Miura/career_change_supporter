// Copyright 2021 Ken Miura

use serde::Serialize;

pub(crate) const EMAIL_FORMAT_INVALID_LENGTH: u32 = 1;
pub(crate) const EMAIL_FORMAT_INVALID_EXPRESSION: u32 = 2;
pub(crate) const PASSWORD_FORMAT_INVALID_LENGTH: u32 = 3;
pub(crate) const PASSWORD_FORMAT_INVALID_EXPRESSION: u32 = 4;
pub(crate) const PASSWORD_FORMAT_CONSTRAINTS_VIOLATION: u32 = 5;
//pub(crate) const AUTHENTICATION_FAILED: u32 = 6;
pub(crate) const DB_CONNECTION_UNAVAILABLE: u32 = 7;
pub(crate) const EXECUTION_CANCELED: u32 = 8;
pub(crate) const USER_ALREADY_EXISTS: u32 = 9;
pub(crate) const USER_DUPLICATE: u32 = 10;
pub(crate) const DB_ACCESS_ERROR: u32 = 11;
pub(crate) const EXCEED_REGISTRATION_LIMIT: u32 = 12;

pub(crate) const INTERNAL_SERVER_ERROR_MESSAGE: &str =
    "サーバでエラーが発生しました。一定時間後、再度お試しください。";

#[derive(Serialize)]
pub(crate) struct Error {
    pub code: u32,
    pub message: String,
}
