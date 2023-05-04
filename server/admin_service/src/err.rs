// Copyright 2021 Ken Miura

//! エラーに関連する構造体、関数を集約するモジュール

use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};
use tracing::error;

/// API呼び出し時の処理の内、admin_service crateのコード発生したエラーに対して付与するエラーコードの列挙<br>
/// admin_service crateでのエラーコードには、30000-39999までの値を利用する。
pub(crate) enum Code {
    UnexpectedErr = 30000,
    EmailOrPwdIncorrect = 30001,
    InvalidPassCode = 30002,
    Unauthorized = 30003,
    NoAccountFound = 30004,
    IllegalPageSize = 30005,
    NoCreateIdentityReqDetailFound = 30006,
    IllegalDate = 30007,
    InvalidFormatReason = 30008,
    NoUpdateIdentityReqDetailFound = 30009,
    NoUserAccountFound = 30010,
    NoIdentityFound = 30011,
    NoCreateCareerReqDetailFound = 30012,
    NoUserAccountFoundOrTheAccountIsDisabled = 30013,
}

pub(crate) fn unexpected_err_resp() -> ErrResp {
    error!("unexpected error");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            code: Code::UnexpectedErr as u32,
        }),
    )
}
