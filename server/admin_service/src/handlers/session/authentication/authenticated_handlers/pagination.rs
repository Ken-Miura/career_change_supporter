// Copyright 2023 Ken Miura

use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};
use serde::Deserialize;

use crate::err::Code;

#[derive(Deserialize)]
pub(crate) struct Pagination {
    pub(super) page: u64,
    pub(super) per_page: u64,
}

const MAX_PAGE_SIZE: u64 = 50;

pub(super) fn validate_page_size(page_size: u64) -> Result<(), ErrResp> {
    if page_size > MAX_PAGE_SIZE {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::IllegalPageSize as u32,
            }),
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(crate) mod tests {

    use axum::http::StatusCode;

    use crate::err::Code;

    use super::*;

    #[test]
    fn validate_page_size_sucees() {
        validate_page_size(MAX_PAGE_SIZE).expect("failed to get Ok");
    }

    #[test]
    fn validate_page_size_fail() {
        let err_resp = validate_page_size(MAX_PAGE_SIZE + 1).expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(Code::IllegalPageSize as u32, err_resp.1.code);
    }
}
