// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};

use crate::err_code;

pub(crate) fn unexpected_err_resp() -> ErrResp {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            code: err_code::UNEXPECTED_ERR,
        }),
    )
}
