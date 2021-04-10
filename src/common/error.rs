// Copyright 2021 Ken Miura

pub(crate) mod code;

use serde::Serialize;

#[derive(Serialize)]
pub(crate) struct Error {
    pub code: u32,
    pub message: String,
}

pub(crate) trait Detail {
    fn code(&self) -> u32;
    fn ui_message(&self) -> String;
}
