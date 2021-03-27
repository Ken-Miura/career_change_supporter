// Copyright 2021 Ken Miura

use serde::Serialize;
use std::collections::HashMap;

/// Message for this code takes two arguments (1. length actually input, 2. Max length)
pub(crate) const EMAIL_FORMAT_INVALID_LENGTH: u32 = 1;
/// Message for this code takes one arguments (1. input email address)
pub(crate) const EMAIL_FORMAT_INVALID_EXPRESSION: u32 = 2;
/// Message for this code takes two arguments (1. Min length, 2. Max length)
pub(crate) const PASSWORD_FORMAT_INVALID_LENGTH: u32 = 3;
pub(crate) const PASSWORD_FORMAT_INVALID_EXPRESSION: u32 = 4;
pub(crate) const PASSWORD_FORMAT_CONSTRAINTS_VIOLATION: u32 = 5;

#[derive(Serialize)]
pub(crate) struct Error {
    pub code: u32,
    pub message: String,
}

lazy_static! {
    pub(crate) static ref MESSAGE: HashMap<u32, &'static str> = {
        let mut m = HashMap::new();
        m.insert(EMAIL_FORMAT_INVALID_LENGTH, "メールアドレスの長さが不正です (入力されたメールアドレスの長さ: {})。メールアドレスは{}文字以下である必要があります。");
        m.insert(EMAIL_FORMAT_INVALID_EXPRESSION, "メールアドレスの形式が不正です (入力されたメールアドレス)。\"email.address@example.com\"のような形式で入力してください。");
        // NOTE: Never include passed password information
        m.insert(PASSWORD_FORMAT_INVALID_LENGTH, "パスワードの長さが不正です。パスワードは{}文字以上、{}文字以下である必要があります。");
        // NOTE: Never include passed password information
        m.insert(PASSWORD_FORMAT_INVALID_EXPRESSION, "パスワードに使用できない文字が含まれています。パスワードに使用可能な文字は、半角英数字と記号です。");
        // NOTE: Never include passed password information
        m.insert(PASSWORD_FORMAT_CONSTRAINTS_VIOLATION, "不正な形式のパスワードです。パスワードは小文字、大文字、数字または記号の内、2種類以上を組み合わせる必要があります。");
        m
    };
}
