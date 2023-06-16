// // Copyright 2023 Ken Miura

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
            StatusCode::BAD_REQUEST,
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
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let ss = find_stopped_settlement_with_exclusive_lock(stopped_settlement_id, txn).await?;

                    let s = entity::settlement::ActiveModel {
                        settlement_id: NotSet,
                        consultation_id: Set(ss.consultation_id),
                        charge_id: Set(ss.charge_id.clone()),
                        fee_per_hour_in_yen: Set(ss.fee_per_hour_in_yen),
                        platform_fee_rate_in_percentage: Set(ss
                            .platform_fee_rate_in_percentage
                            .clone()),
                        credit_facilities_expired_at: Set(ss.credit_facilities_expired_at),
                    };
                    let _ = s.insert(txn).await.map_err(|e| {
                        error!(
                            "failed to insert settlement (stopped_settlement: {:?}): {}",
                            ss, e,
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let _ = entity::stopped_settlement::Entity::delete_by_id(stopped_settlement_id)
                        .exec(txn)
                        .await
                        .map_err(|e| {
                            error!(
                                "failed to delete stopped_settlement (stopped_settlement_id: {}): {}",
                                stopped_settlement_id, e,
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

async fn find_stopped_settlement_with_exclusive_lock(
    stopped_settlement_id: i64,
    txn: &DatabaseTransaction,
) -> Result<entity::stopped_settlement::Model, ErrRespStruct> {
    let result = entity::stopped_settlement::Entity::find_by_id(stopped_settlement_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find stopped_settlement (stopped_settlement_id: {}): {}",
                stopped_settlement_id, e,
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    let model = result.ok_or_else(|| {
        error!(
            "no stopped_settlement (stopped_settlement_id: {}) found",
            stopped_settlement_id,
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(model)
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone};

    use super::*;

    struct ResumeSettlementReqOperationMock {
        stopped_settlement_id: i64,
        credit_facilities_expired_at: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl ResumeSettlementReqOperation for ResumeSettlementReqOperationMock {
        async fn find_credit_facilities_expired_at_on_the_stopped_settlement(
            &self,
            stopped_settlement_id: i64,
        ) -> Result<Option<DateTime<FixedOffset>>, ErrResp> {
            assert_eq!(self.stopped_settlement_id, stopped_settlement_id);
            Ok(Some(self.credit_facilities_expired_at))
        }

        async fn move_to_settlement(&self, stopped_settlement_id: i64) -> Result<(), ErrResp> {
            assert_eq!(self.stopped_settlement_id, stopped_settlement_id);
            Ok(())
        }
    }

    #[tokio::test]

    async fn post_stop_settlement_req_internal_success1() {
        let stopped_settlement_id = 64431;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let credit_facilities_expired_at = current_date_time + Duration::seconds(1);
        let op_mock = ResumeSettlementReqOperationMock {
            stopped_settlement_id,
            credit_facilities_expired_at,
        };

        let result =
            post_resume_settlement_req_internal(stopped_settlement_id, current_date_time, op_mock)
                .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(ResumeSettlementReqResult {}, resp.1 .0);
    }

    #[tokio::test]

    async fn post_stop_settlement_req_internal_success2() {
        let stopped_settlement_id = 64431;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let credit_facilities_expired_at = current_date_time;
        let op_mock = ResumeSettlementReqOperationMock {
            stopped_settlement_id,
            credit_facilities_expired_at,
        };

        let result =
            post_resume_settlement_req_internal(stopped_settlement_id, current_date_time, op_mock)
                .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(ResumeSettlementReqResult {}, resp.1 .0);
    }

    #[tokio::test]

    async fn post_stop_settlement_req_internal_fail_stopped_settlement_id_is_zero() {
        let stopped_settlement_id = 0;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let credit_facilities_expired_at = current_date_time;
        let op_mock = ResumeSettlementReqOperationMock {
            stopped_settlement_id,
            credit_facilities_expired_at,
        };

        let result =
            post_resume_settlement_req_internal(stopped_settlement_id, current_date_time, op_mock)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(
            Code::StoppedSettlementIdIsNotPositive as u32,
            resp.1 .0.code
        );
    }

    #[tokio::test]

    async fn post_stop_settlement_req_internal_fail_stopped_settlement_id_is_negative() {
        let stopped_settlement_id = -1;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let credit_facilities_expired_at = current_date_time;
        let op_mock = ResumeSettlementReqOperationMock {
            stopped_settlement_id,
            credit_facilities_expired_at,
        };

        let result =
            post_resume_settlement_req_internal(stopped_settlement_id, current_date_time, op_mock)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(
            Code::StoppedSettlementIdIsNotPositive as u32,
            resp.1 .0.code
        );
    }

    #[tokio::test]

    async fn post_stop_settlement_req_internal_fail_current_date_time_exceeds_credit_facilities_expired_at(
    ) {
        let stopped_settlement_id = 53215;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let credit_facilities_expired_at = current_date_time - Duration::seconds(1);
        let op_mock = ResumeSettlementReqOperationMock {
            stopped_settlement_id,
            credit_facilities_expired_at,
        };

        let result =
            post_resume_settlement_req_internal(stopped_settlement_id, current_date_time, op_mock)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::CreditFacilitiesAlreadyExpired as u32, resp.1 .0.code);
    }
}
