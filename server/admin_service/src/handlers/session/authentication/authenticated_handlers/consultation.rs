// Copyright 2023 Ken Miura

use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};
use serde::Deserialize;
use tracing::error;

use crate::err::Code;

pub(crate) mod consultation_by_consultation_id;
pub(crate) mod user_rating_by_consultation_id;

#[derive(Deserialize)]
pub(crate) struct ConsultationIdQuery {
    consultation_id: i64,
}

fn validate_consultation_id_is_positive(consultation_id: i64) -> Result<(), ErrResp> {
    if !consultation_id.is_positive() {
        error!("consultation_id is not positive: {}", consultation_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultationIdIsNotPositive as u32,
            }),
        ));
    }
    Ok(())
}
