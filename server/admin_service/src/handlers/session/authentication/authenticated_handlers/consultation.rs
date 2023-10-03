// Copyright 2023 Ken Miura

use axum::{http::StatusCode, Json};
use common::{
    payment_platform::{
        construct_access_info, AccessInfo, KEY_TO_PAYMENT_PLATFORM_API_PASSWORD,
        KEY_TO_PAYMENT_PLATFORM_API_URL, KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
    },
    ApiError, ErrResp,
};
use once_cell::sync::Lazy;
use tracing::error;

use crate::err::Code;

pub(crate) mod consultant_rating_by_consultation_id;
pub(crate) mod consultation_by_consultation_id;
pub(crate) mod make_payment_req;
pub(crate) mod receipt_by_consultation_id;
pub(crate) mod refund_by_consultation_id;
pub(crate) mod refund_req;
pub(crate) mod resume_settlement_req;
pub(crate) mod settlement_by_consultation_id;
pub(crate) mod stop_settlement_req;
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

/// PAY.JPにアクセスするための情報を保持する変数
static ACCESS_INFO: Lazy<AccessInfo> = Lazy::new(|| {
    construct_access_info(
        KEY_TO_PAYMENT_PLATFORM_API_URL,
        KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
        KEY_TO_PAYMENT_PLATFORM_API_PASSWORD,
    )
    .expect("failed to get Ok")
});
