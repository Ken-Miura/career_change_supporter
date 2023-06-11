// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;

use super::super::admin::Admin;
use super::{validate_consultation_id_is_positive, ConsultationIdQuery};

pub(crate) async fn get_receipt_by_consultation_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<ConsultationIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<ReceiptResult> {
    let query = query.0;
    let op = ReceiptOperationImpl { pool };
    get_receipt_by_consultation_id_internal(query.consultation_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReceiptResult {
    receipt: Option<Receipt>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct Receipt {
    receipt_id: i64,
    consultation_id: i64,
    charge_id: String,
    fee_per_hour_in_yen: i32,
    platform_fee_rate_in_percentage: String,
    settled_at: String, // RFC 3339形式の文字列
}

async fn get_receipt_by_consultation_id_internal(
    consultation_id: i64,
    op: impl ReceiptOperation,
) -> RespResult<ReceiptResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    let receipts = op.get_receipts_by_consultation_id(consultation_id).await?;
    if receipts.len() > 1 {
        error!(
            "{} receipts found (consultation_id: {})",
            receipts.len(),
            consultation_id
        );
        return Err(unexpected_err_resp());
    }
    let receipt = receipts.get(0).cloned();
    Ok((StatusCode::OK, Json(ReceiptResult { receipt })))
}

#[async_trait]
trait ReceiptOperation {
    async fn get_receipts_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Vec<Receipt>, ErrResp>;
}

struct ReceiptOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ReceiptOperation for ReceiptOperationImpl {
    async fn get_receipts_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Vec<Receipt>, ErrResp> {
        let models = entity::receipt::Entity::find()
            .filter(entity::receipt::Column::ConsultationId.eq(consultation_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter receipt (consultation_id: {}): {}",
                    consultation_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| Receipt {
                receipt_id: m.receipt_id,
                consultation_id: m.consultation_id,
                charge_id: m.charge_id,
                fee_per_hour_in_yen: m.fee_per_hour_in_yen,
                platform_fee_rate_in_percentage: m.platform_fee_rate_in_percentage,
                settled_at: m
                    .settled_at
                    .with_timezone(&(*JAPANESE_TIME_ZONE))
                    .to_rfc3339(),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::ErrResp;

    use crate::err::Code;

    use super::*;

    struct ReceiptOperationMock {
        consultation_id: i64,
        receipts: Vec<Receipt>,
    }

    #[async_trait]
    impl ReceiptOperation for ReceiptOperationMock {
        async fn get_receipts_by_consultation_id(
            &self,
            consultation_id: i64,
        ) -> Result<Vec<Receipt>, ErrResp> {
            if self.consultation_id != consultation_id {
                return Ok(vec![]);
            }
            Ok(self.receipts.clone())
        }
    }

    fn create_dummy_receipt1(consultation_id: i64) -> Receipt {
        Receipt {
            receipt_id: 10,
            consultation_id,
            charge_id: "ch_6ebea6645ba1bb27307032b23cd5d".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            settled_at: "2023-04-13T14:00:00.0000+09:00 ".to_string(),
        }
    }

    fn create_dummy_receipt2(consultation_id: i64) -> Receipt {
        Receipt {
            receipt_id: 12,
            consultation_id,
            charge_id: "ch_7ebea6645ba1bb27307032b23cd5d".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            settled_at: "2023-04-13T14:00:00.0000+09:00 ".to_string(),
        }
    }

    #[tokio::test]

    async fn get_receipt_by_consultation_id_internal_success_1_result() {
        let consultation_id = 64431;
        let rc1 = create_dummy_receipt1(consultation_id);
        let op_mock = ReceiptOperationMock {
            consultation_id,
            receipts: vec![rc1.clone()],
        };

        let result = get_receipt_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Some(rc1), resp.1 .0.receipt);
    }

    #[tokio::test]

    async fn get_receipt_by_consultation_id_internal_success_no_result() {
        let consultation_id = 64431;
        let rc1 = create_dummy_receipt1(consultation_id);
        let op_mock = ReceiptOperationMock {
            consultation_id,
            receipts: vec![rc1],
        };
        let dummy_id = consultation_id + 501;

        let result = get_receipt_by_consultation_id_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(None, resp.1 .0.receipt);
    }

    #[tokio::test]

    async fn get_receipt_by_consultation_id_internal_fail_multiple_results() {
        let consultation_id = 64431;
        let rc1 = create_dummy_receipt1(consultation_id);
        let rc2 = create_dummy_receipt2(consultation_id);
        let op_mock = ReceiptOperationMock {
            consultation_id,
            receipts: vec![rc1, rc2],
        };

        let result = get_receipt_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(unexpected_err_resp().0, resp.0);
        assert_eq!(unexpected_err_resp().1 .0.code, resp.1 .0.code);
    }

    #[tokio::test]
    async fn get_receipt_by_consultation_id_internal_fail_consultation_id_is_zero() {
        let consultation_id = 0;
        let rc1 = create_dummy_receipt1(consultation_id);
        let op_mock = ReceiptOperationMock {
            consultation_id,
            receipts: vec![rc1],
        };

        let result = get_receipt_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_receipt_by_consultation_id_internal_fail_consultation_id_is_negative() {
        let consultation_id = -1;
        let rc1 = create_dummy_receipt1(consultation_id);
        let op_mock = ReceiptOperationMock {
            consultation_id,
            receipts: vec![rc1],
        };

        let result = get_receipt_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }
}
