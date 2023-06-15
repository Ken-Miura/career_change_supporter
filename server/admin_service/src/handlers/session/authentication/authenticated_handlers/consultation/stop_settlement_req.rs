// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::ActiveValue::NotSet;
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QuerySelect, Set,
    TransactionError, TransactionTrait,
};
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

    op.move_to_stopped_settlement(settlement_id, current_date_time)
        .await?;

    Ok((StatusCode::OK, Json(StopSettlementReqResult {})))
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
    async fn move_to_stopped_settlement(
        &self,
        settlement_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct StopSettlementReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl StopSettlementReqOperation for StopSettlementReqOperationImpl {
    async fn move_to_stopped_settlement(
        &self,
        settlement_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let s = find_settlement_with_exclusive_lock(settlement_id, txn).await?;

                    let ss = entity::stopped_settlement::ActiveModel {
                        stopped_settlement_id: NotSet,
                        consultation_id: Set(s.consultation_id),
                        charge_id: Set(s.charge_id.clone()),
                        fee_per_hour_in_yen: Set(s.fee_per_hour_in_yen),
                        platform_fee_rate_in_percentage: Set(s
                            .platform_fee_rate_in_percentage
                            .clone()),
                        credit_facilities_expired_at: Set(s.credit_facilities_expired_at),
                        stopped_at: Set(current_date_time),
                    };
                    let _ = ss.insert(txn).await.map_err(|e| {
                        error!(
                            "failed to insert stopped_settlement (settlement: {:?}): {}",
                            s, e,
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let _ = entity::settlement::Entity::delete_by_id(settlement_id)
                        .exec(txn)
                        .await
                        .map_err(|e| {
                            error!(
                                "failed to delete settlement (settlement_id: {}): {}",
                                settlement_id, e,
                            );
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(db_err) => {
                    error!("connection error: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    error!("failed to move_to_stopped_settlement: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
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

#[cfg(test)]
mod tests {

    use chrono::TimeZone;

    use super::*;

    struct StopSettlementReqOperationMock {
        settlement_id: i64,
        current_date_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl StopSettlementReqOperation for StopSettlementReqOperationMock {
        async fn move_to_stopped_settlement(
            &self,
            settlement_id: i64,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.settlement_id, settlement_id);
            assert_eq!(self.current_date_time, current_date_time);
            Ok(())
        }
    }

    #[tokio::test]

    async fn post_stop_settlement_req_internal_success() {
        let settlement_id = 64431;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let op_mock = StopSettlementReqOperationMock {
            settlement_id,
            current_date_time,
        };

        let result =
            post_stop_settlement_req_internal(settlement_id, current_date_time, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(StopSettlementReqResult {}, resp.1 .0);
    }

    #[tokio::test]

    async fn post_stop_settlement_req_internal_fail_settlement_id_is_zero() {
        let settlement_id = 0;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let op_mock = StopSettlementReqOperationMock {
            settlement_id,
            current_date_time,
        };

        let result =
            post_stop_settlement_req_internal(settlement_id, current_date_time, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::SettlementIdIsNotPositive as u32, resp.1 .0.code);
    }

    #[tokio::test]

    async fn post_stop_settlement_req_internal_fail_settlement_id_is_negative() {
        let settlement_id = -1;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let op_mock = StopSettlementReqOperationMock {
            settlement_id,
            current_date_time,
        };

        let result =
            post_stop_settlement_req_internal(settlement_id, current_date_time, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::SettlementIdIsNotPositive as u32, resp.1 .0.code);
    }
}
