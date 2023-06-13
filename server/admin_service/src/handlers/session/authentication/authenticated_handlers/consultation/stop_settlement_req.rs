// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

use super::super::admin::Admin;

pub(crate) async fn post_stop_settlement_req(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<StopSettlementReq>,
) -> RespResult<StopSettlementReqResult> {
    let op = StopSettlementReqOperationImpl { pool };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    post_stop_settlement_req_internal(req.settlement_id, current_date_time, op).await
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct StopSettlementReq {
    settlement_id: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct StopSettlementReqResult {}

async fn post_stop_settlement_req_internal(
    settlement_id: i64,
    current_date_time: DateTime<FixedOffset>,
    op: impl StopSettlementReqOperation,
) -> RespResult<StopSettlementReqResult> {
    validate_settlement_id_is_positive(settlement_id)?;

    let opt = op
        .find_credit_facilities_expired_at_on_the_settlement(settlement_id)
        .await?;
    let exp_date_time = opt.ok_or_else(|| {
        error!("no settlement (settlement_id: {}) found", settlement_id);
        unexpected_err_resp()
    })?;

    todo!()
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

#[async_trait]
trait StopSettlementReqOperation {
    async fn find_credit_facilities_expired_at_on_the_settlement(
        &self,
        settlement_id: i64,
    ) -> Result<Option<DateTime<FixedOffset>>, ErrResp>;
}

struct StopSettlementReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl StopSettlementReqOperation for StopSettlementReqOperationImpl {
    async fn find_credit_facilities_expired_at_on_the_settlement(
        &self,
        settlement_id: i64,
    ) -> Result<Option<DateTime<FixedOffset>>, ErrResp> {
        let model = entity::settlement::Entity::find_by_id(settlement_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find settlement (settlement_id: {}): {}",
                    settlement_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.credit_facilities_expired_at))
    }
}

#[cfg(test)]
mod tests {}
