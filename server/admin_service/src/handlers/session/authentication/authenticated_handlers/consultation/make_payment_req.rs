// // Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::payment_platform::charge::{ChargeOperation, ChargeOperationImpl};
use common::{ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::ActiveValue::NotSet;
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionError, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

use super::super::admin::Admin;
use super::super::find_settlement_with_exclusive_lock;
use super::{validate_settlement_id_is_positive, ACCESS_INFO};

pub(crate) async fn post_make_payment_req(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<MakePaymentReq>,
) -> RespResult<MakePaymentReqResult> {
    let op = MakePaymentReqOperationImpl { pool };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    post_make_payment_req_internal(req.settlement_id, current_date_time, op).await
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct MakePaymentReq {
    settlement_id: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct MakePaymentReqResult {}

async fn post_make_payment_req_internal(
    settlement_id: i64,
    current_date_time: DateTime<FixedOffset>,
    op: impl MakePaymentReqOperation,
) -> RespResult<MakePaymentReqResult> {
    validate_settlement_id_is_positive(settlement_id)?;

    let opt = op
        .find_credit_facilities_expired_at_on_the_settlement(settlement_id)
        .await?;
    let expired_date_time = opt.ok_or_else(|| {
        error!("no settlement (settlement_id: {}) found", settlement_id);
        unexpected_err_resp()
    })?;
    if current_date_time > expired_date_time {
        error!(
            "current date time ({}) exceeds credit faclities expiray date ({})",
            current_date_time, expired_date_time
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::CreditFacilitiesAlreadyExpired as u32,
            }),
        ));
    }

    op.make_payment(settlement_id, current_date_time).await?;

    Ok((StatusCode::OK, Json(MakePaymentReqResult {})))
}

#[async_trait]
trait MakePaymentReqOperation {
    async fn find_credit_facilities_expired_at_on_the_settlement(
        &self,
        settlement_id: i64,
    ) -> Result<Option<DateTime<FixedOffset>>, ErrResp>;

    async fn make_payment(
        &self,
        settlement_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct MakePaymentReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl MakePaymentReqOperation for MakePaymentReqOperationImpl {
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

    async fn make_payment(
        &self,
        settlement_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let s = find_settlement_with_exclusive_lock(settlement_id, txn).await?;

                    let r = entity::receipt::ActiveModel {
                        receipt_id: NotSet,
                        consultation_id: Set(s.consultation_id),
                        charge_id: Set(s.charge_id.clone()),
                        fee_per_hour_in_yen: Set(s.fee_per_hour_in_yen),
                        platform_fee_rate_in_percentage: Set(s
                            .platform_fee_rate_in_percentage
                            .clone()),
                        settled_at: Set(current_date_time),
                    };
                    let receipt_result = r.insert(txn).await.map_err(|e| {
                        error!("failed to insert receipt (settlement: {:?}): {}", s, e,);
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

                    let charge_id = receipt_result.charge_id;
                    let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
                    let _ = charge_op
                        .capture_the_charge(charge_id.as_str())
                        .await
                        .map_err(|e| {
                            error!(
                                "failed to capture the charge (charge_id: {}): {}",
                                charge_id, e
                            );
                            ErrRespStruct {
                                err_resp: (
                                    StatusCode::BAD_REQUEST,
                                    Json(ApiError {
                                        code: Code::PaymentRelatedErr as u32,
                                    }),
                                ),
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
                    error!("failed to make_payment: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone};

    use super::*;

    struct MakePaymentReqOperationMock {
        settlement_id: i64,
        credit_facilities_expired_at: DateTime<FixedOffset>,
        current_date_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl MakePaymentReqOperation for MakePaymentReqOperationMock {
        async fn find_credit_facilities_expired_at_on_the_settlement(
            &self,
            settlement_id: i64,
        ) -> Result<Option<DateTime<FixedOffset>>, ErrResp> {
            assert_eq!(self.settlement_id, settlement_id);
            Ok(Some(self.credit_facilities_expired_at))
        }

        async fn make_payment(
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

    async fn post_make_payment_req_internal_success1() {
        let settlement_id = 64431;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let credit_facilities_expired_at = current_date_time + Duration::seconds(1);
        let op_mock = MakePaymentReqOperationMock {
            settlement_id,
            credit_facilities_expired_at,
            current_date_time,
        };

        let result =
            post_make_payment_req_internal(settlement_id, current_date_time, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(MakePaymentReqResult {}, resp.1 .0);
    }

    #[tokio::test]

    async fn post_make_payment_req_internal_success2() {
        let settlement_id = 64431;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let credit_facilities_expired_at = current_date_time;
        let op_mock = MakePaymentReqOperationMock {
            settlement_id,
            credit_facilities_expired_at,
            current_date_time,
        };

        let result =
            post_make_payment_req_internal(settlement_id, current_date_time, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(MakePaymentReqResult {}, resp.1 .0);
    }

    #[tokio::test]

    async fn post_make_payment_req_internal_fail_settlement_id_is_zero() {
        let settlement_id = 0;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let credit_facilities_expired_at = current_date_time;
        let op_mock = MakePaymentReqOperationMock {
            settlement_id,
            credit_facilities_expired_at,
            current_date_time,
        };

        let result =
            post_make_payment_req_internal(settlement_id, current_date_time, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::SettlementIdIsNotPositive as u32, resp.1 .0.code);
    }

    #[tokio::test]

    async fn post_make_payment_req_internal_fail_settlement_id_is_negative() {
        let settlement_id = -1;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let credit_facilities_expired_at = current_date_time;
        let op_mock = MakePaymentReqOperationMock {
            settlement_id,
            credit_facilities_expired_at,
            current_date_time,
        };

        let result =
            post_make_payment_req_internal(settlement_id, current_date_time, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::SettlementIdIsNotPositive as u32, resp.1 .0.code);
    }

    #[tokio::test]

    async fn post_make_payment_req_internal_fail_current_date_time_exceeds_credit_facilities_expired_at(
    ) {
        let settlement_id = 53215;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let credit_facilities_expired_at = current_date_time - Duration::seconds(1);
        let op_mock = MakePaymentReqOperationMock {
            settlement_id,
            credit_facilities_expired_at,
            current_date_time,
        };

        let result =
            post_make_payment_req_internal(settlement_id, current_date_time, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::CreditFacilitiesAlreadyExpired as u32, resp.1 .0.code);
    }
}
