// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::unexpected_err_resp;

use super::super::admin::Admin;
use super::validate_consultation_id_is_positive;

pub(crate) async fn post_stop_settlement_req(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<StopSettlementReq>,
) -> RespResult<StopSettlementReqResult> {
    let op = StopSettlementReqOperationImpl { pool };
    post_stop_settlement_req_internal(req.settlement_id, op).await
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct StopSettlementReq {
    settlement_id: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct StopSettlementReqResult {}

async fn post_stop_settlement_req_internal(
    settlement_id: i64,
    op: impl StopSettlementReqOperation,
) -> RespResult<StopSettlementReqResult> {
    todo!()
    // validate_consultation_id_is_positive(consultation_id)?;
    // let stopped_settlements = op
    //     .get_stopped_settlements_by_consultation_id(consultation_id)
    //     .await?;
    // if stopped_settlements.len() > 1 {
    //     error!(
    //         "{} stopped_settlements found (consultation_id: {})",
    //         stopped_settlements.len(),
    //         consultation_id
    //     );
    //     return Err(unexpected_err_resp());
    // }
    // let stopped_settlement = stopped_settlements.get(0).cloned();
    // Ok((
    //     StatusCode::OK,
    //     Json(StoppedSettlementResult { stopped_settlement }),
    // ))
}

#[async_trait]
trait StopSettlementReqOperation {}

struct StopSettlementReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl StopSettlementReqOperation for StopSettlementReqOperationImpl {}

#[cfg(test)]
mod tests {}
