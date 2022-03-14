// Copyright 2021 Ken Miura

//! エラーに関連する構造体、関数を集約するモジュール

use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};

/// API呼び出し時の処理の内、user_service crateのコード発生したエラーに対して付与するエラーコードの列挙<br>
/// user_service crateでのエラーコードには、20000-29999までの値を利用する。
pub(crate) enum Code {
    UnexpectedErr = 20000,
    EmailOrPwdIncorrect = 20005,
    Unauthorized = 20006,
    AccountDisabled = 20007,
}

pub(crate) fn unexpected_err_resp() -> ErrResp {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            code: Code::UnexpectedErr as u32,
        }),
    )
}
