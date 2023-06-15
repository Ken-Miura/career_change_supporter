// // Copyright 2023 Ken Miura

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

pub(crate) async fn post_resume_settlement_req(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<ResumeSettlementReq>,
) -> RespResult<ResumeSettlementReqResult> {
    let op = ResumeSettlementReqOperationImpl { pool };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    post_resume_settlement_req_internal(req.stopped_settlement_id, current_date_time, op).await
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct ResumeSettlementReq {
    stopped_settlement_id: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ResumeSettlementReqResult {}

async fn post_resume_settlement_req_internal(
    stopped_settlement_id: i64,
    current_date_time: DateTime<FixedOffset>,
    op: impl ResumeSettlementReqOperation,
) -> RespResult<ResumeSettlementReqResult> {
    validate_stopped_settlement_id_is_positive(stopped_settlement_id)?;

    let opt = op
        .find_credit_facilities_expired_at_on_the_stopped_settlement(stopped_settlement_id)
        .await?;
    let expired_date_time = opt.ok_or_else(|| {
        error!(
            "no stopped_settlement (stopped_settlement_id: {}) found",
            stopped_settlement_id
        );
        unexpected_err_resp()
    })?;
    if current_date_time > expired_date_time {
        error!(
            "credit faclities expiray date ({}) passed current date time ({})",
            expired_date_time, current_date_time
        );
        return Err((
            StatusCode::OK,
            Json(ApiError {
                code: Code::CreditFacilitiesAlreadyExpired as u32,
            }),
        ));
    }

    op.move_to_settlement(stopped_settlement_id).await?;

    Ok((StatusCode::OK, Json(ResumeSettlementReqResult {})))
}

fn validate_stopped_settlement_id_is_positive(stopped_settlement_id: i64) -> Result<(), ErrResp> {
    if !stopped_settlement_id.is_positive() {
        error!(
            "stopped_settlement_id is not positive: {}",
            stopped_settlement_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::StoppedSettlementIdIsNotPositive as u32,
            }),
        ));
    }
    Ok(())
}

#[async_trait]
trait ResumeSettlementReqOperation {
    async fn find_credit_facilities_expired_at_on_the_stopped_settlement(
        &self,
        stopped_settlement_id: i64,
    ) -> Result<Option<DateTime<FixedOffset>>, ErrResp>;

    async fn move_to_settlement(&self, stopped_settlement_id: i64) -> Result<(), ErrResp>;
}

struct ResumeSettlementReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ResumeSettlementReqOperation for ResumeSettlementReqOperationImpl {
    async fn find_credit_facilities_expired_at_on_the_stopped_settlement(
        &self,
        stopped_settlement_id: i64,
    ) -> Result<Option<DateTime<FixedOffset>>, ErrResp> {
        let model = entity::stopped_settlement::Entity::find_by_id(stopped_settlement_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find stopped_settlement (stopped_settlement_id: {}): {}",
                    stopped_settlement_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.credit_facilities_expired_at))
    }

    async fn move_to_settlement(&self, stopped_settlement_id: i64) -> Result<(), ErrResp> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
