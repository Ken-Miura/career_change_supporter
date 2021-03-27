// Copyright 2021 Ken Miura

use serde::Serialize;

pub(crate) const EMAIL_FORMAT_INVALID_LENGTH: u32 = 1;
pub(crate) const EMAIL_FORMAT_INVALID_EXPRESSION: u32 = 2;
pub(crate) const PASSWORD_FORMAT_INVALID_LENGTH: u32 = 3;
pub(crate) const PASSWORD_FORMAT_INVALID_EXPRESSION: u32 = 4;
pub(crate) const PASSWORD_FORMAT_CONSTRAINTS_VIOLATION: u32 = 5;
pub(crate) const AUTHENTICATION_FAILED: u32 = 6;

#[derive(Serialize)]
pub(crate) struct Error {
    pub code: u32,
    pub message: String,
}
