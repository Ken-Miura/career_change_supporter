// // Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use common::payment_platform::charge::{ChargeOperation, ChargeOperationImpl, RefundQuery};
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
use super::ACCESS_INFO;

const REFUNDABLE_DURATION_IN_DAYS: i64 = 180;

pub(crate) async fn post_refund_req(
    Admin { admin_info }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<RefundReq>,
) -> RespResult<RefundReqResult> {
    let op = RefundReqOperationImpl { pool };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    post_refund_req_internal(
        req.receipt_id,
        current_date_time,
        admin_info.email_address,
        op,
    )
    .await
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct RefundReq {
    receipt_id: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct RefundReqResult {}

async fn post_refund_req_internal(
    receipt_id: i64,
    current_date_time: DateTime<FixedOffset>,
    admin_email_address: String,
    op: impl RefundReqOperation,
) -> RespResult<RefundReqResult> {
    validate_receipt_id_is_positive(receipt_id)?;

    let opt = op.find_settled_at_on_the_receipt(receipt_id).await?;
    let settled_at = opt.ok_or_else(|| {
        error!("no receipt (receipt_id: {}) found", receipt_id);
        unexpected_err_resp()
    })?;
    let refund_limit_date_time = settled_at + Duration::days(REFUNDABLE_DURATION_IN_DAYS);
    if current_date_time > refund_limit_date_time {
        error!(
            "current date time ({}) exceeds refund limit date time ({}, settled at {})",
            current_date_time, refund_limit_date_time, settled_at
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ExceedsRefundTimeLimit as u32,
            }),
        ));
    }

    let message = format!(
        "charge_was_refunded_by_administrator ({})",
        admin_email_address
    );
    let query = RefundQuery::new(message.to_string()).map_err(|e| {
        error!(
            "failed to construst RefundQuery (refund_reason: {}): {}",
            message, e
        );
        unexpected_err_resp()
    })?;
    op.refund(receipt_id, query, current_date_time).await?;

    Ok((StatusCode::OK, Json(RefundReqResult {})))
}

#[async_trait]
trait RefundReqOperation {
    async fn find_settled_at_on_the_receipt(
        &self,
        receipt_id: i64,
    ) -> Result<Option<DateTime<FixedOffset>>, ErrResp>;

    async fn refund(
        &self,
        receipt_id: i64,
        query: RefundQuery,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct RefundReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl RefundReqOperation for RefundReqOperationImpl {
    async fn find_settled_at_on_the_receipt(
        &self,
        receipt_id: i64,
    ) -> Result<Option<DateTime<FixedOffset>>, ErrResp> {
        let model = entity::receipt::Entity::find_by_id(receipt_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!("failed to find receipt (receipt_id: {}): {}", receipt_id, e);
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.settled_at))
    }

    async fn refund(
        &self,
        receipt_id: i64,
        query: RefundQuery,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let rc = find_receipt_with_exclusive_lock(receipt_id, txn).await?;

                    let rf = entity::refund::ActiveModel {
                        refund_id: NotSet,
                        consultation_id: Set(rc.consultation_id),
                        charge_id: Set(rc.charge_id.clone()),
                        fee_per_hour_in_yen: Set(rc.fee_per_hour_in_yen),
                        platform_fee_rate_in_percentage: Set(rc
                            .platform_fee_rate_in_percentage
                            .clone()),
                        settled_at: Set(rc.settled_at),
                        refunded_at: Set(current_date_time),
                    };
                    let refund_result = rf.insert(txn).await.map_err(|e| {
                        error!("failed to insert refund (receipt: {:?}): {}", rc, e,);
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let _ = entity::receipt::Entity::delete_by_id(receipt_id)
                        .exec(txn)
                        .await
                        .map_err(|e| {
                            error!(
                                "failed to delete receipt (receipt_id: {}): {}",
                                receipt_id, e,
                            );
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;

                    let charge_id = refund_result.charge_id;
                    let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
                    let _ = charge_op
                        .refund(charge_id.as_str(), query)
                        .await
                        .map_err(|e| {
                            error!(
                                "failed to refund the charge (charge_id: {}): {}",
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
                    error!("failed to refund: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

fn validate_receipt_id_is_positive(receipt_id: i64) -> Result<(), ErrResp> {
    if !receipt_id.is_positive() {
        error!("receipt_id is not positive: {}", receipt_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ReceiptIdIsNotPositive as u32,
            }),
        ));
    }
    Ok(())
}

async fn find_receipt_with_exclusive_lock(
    receipt_id: i64,
    txn: &DatabaseTransaction,
) -> Result<entity::receipt::Model, ErrRespStruct> {
    let result = entity::receipt::Entity::find_by_id(receipt_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!("failed to find receipt (receipt_id: {}): {}", receipt_id, e,);
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    let model = result.ok_or_else(|| {
        error!("no receipt (receipt_id: {}) found", receipt_id,);
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

    struct RefundReqOperationMock {
        receipt_id: i64,
        settled_at: DateTime<FixedOffset>,
        query: RefundQuery,
        current_date_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl RefundReqOperation for RefundReqOperationMock {
        async fn find_settled_at_on_the_receipt(
            &self,
            receipt_id: i64,
        ) -> Result<Option<DateTime<FixedOffset>>, ErrResp> {
            assert_eq!(self.receipt_id, receipt_id);
            Ok(Some(self.settled_at))
        }

        async fn refund(
            &self,
            receipt_id: i64,
            query: RefundQuery,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.receipt_id, receipt_id);
            assert_eq!(self.query, query);
            assert_eq!(self.current_date_time, current_date_time);
            Ok(())
        }
    }

    #[tokio::test]

    async fn post_refund_req_internal_success1() {
        let receipt_id = 64431;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let admin_email_address = "admin@test.com";
        let message = format!(
            "charge_was_refunded_by_administrator ({})",
            admin_email_address
        );
        let query = RefundQuery::new(message.to_string()).expect("failed to get Ok");
        let settled_at =
            current_date_time - Duration::days(REFUNDABLE_DURATION_IN_DAYS) + Duration::seconds(1);
        let op_mock = RefundReqOperationMock {
            receipt_id,
            settled_at,
            query,
            current_date_time,
        };

        let result = post_refund_req_internal(
            receipt_id,
            current_date_time,
            admin_email_address.to_string(),
            op_mock,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(RefundReqResult {}, resp.1 .0);
    }

    #[tokio::test]

    async fn post_refund_req_internal_success2() {
        let receipt_id = 64431;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let admin_email_address = "admin@test.com";
        let message = format!(
            "charge_was_refunded_by_administrator ({})",
            admin_email_address
        );
        let query = RefundQuery::new(message.to_string()).expect("failed to get Ok");
        let settled_at = current_date_time - Duration::days(REFUNDABLE_DURATION_IN_DAYS);
        let op_mock = RefundReqOperationMock {
            receipt_id,
            settled_at,
            query,
            current_date_time,
        };

        let result = post_refund_req_internal(
            receipt_id,
            current_date_time,
            admin_email_address.to_string(),
            op_mock,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(RefundReqResult {}, resp.1 .0);
    }

    #[tokio::test]

    async fn post_refund_req_internal_fail_receipt_id_is_zero() {
        let receipt_id = 0;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let admin_email_address = "admin@test.com";
        let message = format!(
            "charge_was_refunded_by_administrator ({})",
            admin_email_address
        );
        let query = RefundQuery::new(message.to_string()).expect("failed to get Ok");
        let settled_at = current_date_time - Duration::days(REFUNDABLE_DURATION_IN_DAYS);
        let op_mock = RefundReqOperationMock {
            receipt_id,
            settled_at,
            query,
            current_date_time,
        };

        let result = post_refund_req_internal(
            receipt_id,
            current_date_time,
            admin_email_address.to_string(),
            op_mock,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::ReceiptIdIsNotPositive as u32, resp.1 .0.code);
    }

    #[tokio::test]

    async fn post_refund_req_internal_fail_receipt_id_is_negative() {
        let receipt_id = -1;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let admin_email_address = "admin@test.com";
        let message = format!("管理者 ({}) の判断による返金処理", admin_email_address);
        let query = RefundQuery::new(message.to_string()).expect("failed to get Ok");
        let settled_at = current_date_time - Duration::days(REFUNDABLE_DURATION_IN_DAYS);
        let op_mock = RefundReqOperationMock {
            receipt_id,
            settled_at,
            query,
            current_date_time,
        };

        let result = post_refund_req_internal(
            receipt_id,
            current_date_time,
            admin_email_address.to_string(),
            op_mock,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::ReceiptIdIsNotPositive as u32, resp.1 .0.code);
    }

    #[tokio::test]

    async fn post_refund_req_internal_fail_current_date_time_exceeds_refund_time_limit() {
        let receipt_id = 35122;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let admin_email_address = "admin@test.com";
        let message = format!(
            "charge_was_refunded_by_administrator ({})",
            admin_email_address
        );
        let query = RefundQuery::new(message.to_string()).expect("failed to get Ok");
        let settled_at =
            current_date_time - Duration::days(REFUNDABLE_DURATION_IN_DAYS) - Duration::seconds(1);
        let op_mock = RefundReqOperationMock {
            receipt_id,
            settled_at,
            query,
            current_date_time,
        };

        let result = post_refund_req_internal(
            receipt_id,
            current_date_time,
            admin_email_address.to_string(),
            op_mock,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::ExceedsRefundTimeLimit as u32, resp.1 .0.code);
    }
}
