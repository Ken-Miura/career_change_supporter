// Copyright 2021 Ken Miura

//! エラーに関連する構造体、関数を集約するモジュール

use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};

/// API呼び出し時の処理の内、admin_service crateのコード発生したエラーに対して付与するエラーコードの列挙<br>
/// admin_service crateでのエラーコードには、30000-39999までの値を利用する。
pub(crate) enum Code {
    UnexpectedErr = 30000,
    EmailOrPwdIncorrect = 30001,
    Unauthorized = 30002,
    IllegalPageSize = 30003,
    NoCreateIdentityReqDetailFound = 30004,
    IllegalDate = 30005,
    InvalidFormatReason = 30006,
    NoUpdateIdentityReqDetailFound = 30007,
    NoUserAccountFound = 30008,
    NoIdentityFound = 30009,
}

pub(crate) fn unexpected_err_resp() -> ErrResp {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            code: Code::UnexpectedErr as u32,
        }),
    )
}
