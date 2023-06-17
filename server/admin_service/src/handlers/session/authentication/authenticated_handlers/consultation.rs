// Copyright 2023 Ken Miura

use axum::{http::StatusCode, Json};
use common::{
    payment_platform::{
        AccessInfo, KEY_TO_PAYMENT_PLATFORM_API_PASSWORD, KEY_TO_PAYMENT_PLATFORM_API_URL,
        KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
    },
    ApiError, ErrResp, ErrRespStruct,
};
use entity::sea_orm::{DatabaseTransaction, EntityTrait, QuerySelect};
use once_cell::sync::Lazy;
use serde::Deserialize;
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

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

async fn find_settlement_with_exclusive_lock(
    settlement_id: i64,
    txn: &DatabaseTransaction,
) -> Result<entity::settlement::Model, ErrRespStruct> {
    let result = entity::settlement::Entity::find_by_id(settlement_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find settlement (settlement_id: {}): {}",
                settlement_id, e,
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    let model = result.ok_or_else(|| {
        error!("no settlement (settlement_id: {}) found", settlement_id,);
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(model)
}

/// PAY.JPにアクセスするための情報を保持する変数
static ACCESS_INFO: Lazy<AccessInfo> = Lazy::new(|| {
    let url_without_path = std::env::var(KEY_TO_PAYMENT_PLATFORM_API_URL).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_PAYMENT_PLATFORM_API_URL
        )
    });
    let username = std::env::var(KEY_TO_PAYMENT_PLATFORM_API_USERNAME).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_PAYMENT_PLATFORM_API_USERNAME
        )
    });
    let password = std::env::var(KEY_TO_PAYMENT_PLATFORM_API_PASSWORD).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_PAYMENT_PLATFORM_API_PASSWORD
        )
    });
    let access_info = AccessInfo::new(url_without_path, username, password);
    access_info.expect("failed to get Ok")
});
