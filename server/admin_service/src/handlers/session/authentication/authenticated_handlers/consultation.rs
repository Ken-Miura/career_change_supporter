// Copyright 2023 Ken Miura

use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};
use tracing::error;

use crate::err::Code;

pub(crate) mod consultant_rating_by_consultation_id;
pub(crate) mod consultation_by_consultation_id;
pub(crate) mod refund_by_consultation_id;
pub(crate) mod settlement_by_consultation_id;
pub(crate) mod stopped_settlement_by_consultation_id;
pub(crate) mod user_rating_by_consultation_id;

fn validate_settlement_id_is_positive(settlement_id: i64) -> Result<(), ErrResp> {
    if !settlement_id.is_positive() {
        error!("settlement_id is not positive: {}", settlement_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::SettlementIdIsNotPositive as u32,
            }),
        ));
    }
    Ok(())
}
