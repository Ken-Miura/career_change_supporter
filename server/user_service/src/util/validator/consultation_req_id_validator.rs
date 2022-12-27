// Copyright 2022 Ken Miura

use axum::http::StatusCode;
use axum::Json;
use common::{ApiError, ErrResp};
use tracing::error;

use crate::err::Code;

pub(crate) fn validate_consultation_req_id_is_positive(
    consultation_req_id: i64,
) -> Result<(), ErrResp> {
    if !consultation_req_id.is_positive() {
        error!(
            "consultation_req_id ({}) is not positive",
            consultation_req_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonPositiveConsultationReqId as u32,
            }),
        ));
    }
    Ok(())
}
