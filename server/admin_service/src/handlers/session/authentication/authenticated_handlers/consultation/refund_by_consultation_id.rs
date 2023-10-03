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
use super::super::{validate_consultation_id_is_positive, ConsultationIdQuery};

pub(crate) async fn get_refund_by_consultation_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<ConsultationIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<RefundResult> {
    let query = query.0;
    let op = RefundOperationImpl { pool };
    get_refund_by_consultation_id_internal(query.consultation_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct RefundResult {
    refund: Option<Refund>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct Refund {
    refund_id: i64,
    consultation_id: i64,
    charge_id: String,
    fee_per_hour_in_yen: i32,
    platform_fee_rate_in_percentage: String,
    settled_at: String,  // RFC 3339形式の文字列
    refunded_at: String, // RFC 3339形式の文字列
}

async fn get_refund_by_consultation_id_internal(
    consultation_id: i64,
    op: impl RefundOperation,
) -> RespResult<RefundResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    let refunds = op.get_refunds_by_consultation_id(consultation_id).await?;
    if refunds.len() > 1 {
        error!(
            "{} refunds found (consultation_id: {})",
            refunds.len(),
            consultation_id
        );
        return Err(unexpected_err_resp());
    }
    let refund = refunds.get(0).cloned();
    Ok((StatusCode::OK, Json(RefundResult { refund })))
}

#[async_trait]
trait RefundOperation {
    async fn get_refunds_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Vec<Refund>, ErrResp>;
}

struct RefundOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl RefundOperation for RefundOperationImpl {
    async fn get_refunds_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Vec<Refund>, ErrResp> {
        let models = entity::refund::Entity::find()
            .filter(entity::refund::Column::ConsultationId.eq(consultation_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter refund (consultation_id: {}): {}",
                    consultation_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| Refund {
                refund_id: m.refund_id,
                consultation_id: m.consultation_id,
                charge_id: m.charge_id,
                fee_per_hour_in_yen: m.fee_per_hour_in_yen,
                platform_fee_rate_in_percentage: m.platform_fee_rate_in_percentage,
                settled_at: m
                    .settled_at
                    .with_timezone(&(*JAPANESE_TIME_ZONE))
                    .to_rfc3339(),
                refunded_at: m
                    .refunded_at
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

    struct RefundOperationMock {
        consultation_id: i64,
        refunds: Vec<Refund>,
    }

    #[async_trait]
    impl RefundOperation for RefundOperationMock {
        async fn get_refunds_by_consultation_id(
            &self,
            consultation_id: i64,
        ) -> Result<Vec<Refund>, ErrResp> {
            if self.consultation_id != consultation_id {
                return Ok(vec![]);
            }
            Ok(self.refunds.clone())
        }
    }

    fn create_dummy_refund1(consultation_id: i64) -> Refund {
        Refund {
            refund_id: 10,
            consultation_id,
            charge_id: "ch_6ebea6645ba1bb27307032b23cd5d".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            settled_at: "2023-04-13T14:00:00.0000+09:00 ".to_string(),
            refunded_at: "2023-04-15T14:00:00.0000+09:00 ".to_string(),
        }
    }

    fn create_dummy_refund2(consultation_id: i64) -> Refund {
        Refund {
            refund_id: 12,
            consultation_id,
            charge_id: "ch_7ebea6645ba1bb27307032b23cd5d".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            settled_at: "2023-04-13T14:00:00.0000+09:00 ".to_string(),
            refunded_at: "2023-04-16T14:00:00.0000+09:00 ".to_string(),
        }
    }

    #[tokio::test]

    async fn get_refund_by_consultation_id_internal_success_1_result() {
        let consultation_id = 64431;
        let rf1 = create_dummy_refund1(consultation_id);
        let op_mock = RefundOperationMock {
            consultation_id,
            refunds: vec![rf1.clone()],
        };

        let result = get_refund_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Some(rf1), resp.1 .0.refund);
    }

    #[tokio::test]

    async fn get_refund_by_consultation_id_internal_success_no_result() {
        let consultation_id = 64431;
        let rf1 = create_dummy_refund1(consultation_id);
        let op_mock = RefundOperationMock {
            consultation_id,
            refunds: vec![rf1],
        };
        let dummy_id = consultation_id + 501;

        let result = get_refund_by_consultation_id_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(None, resp.1 .0.refund);
    }

    #[tokio::test]

    async fn get_refund_by_consultation_id_internal_fail_multiple_results() {
        let consultation_id = 64431;
        let rf1 = create_dummy_refund1(consultation_id);
        let rf2 = create_dummy_refund2(consultation_id);
        let op_mock = RefundOperationMock {
            consultation_id,
            refunds: vec![rf1, rf2],
        };

        let result = get_refund_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(unexpected_err_resp().0, resp.0);
        assert_eq!(unexpected_err_resp().1 .0.code, resp.1 .0.code);
    }

    #[tokio::test]
    async fn get_refund_by_consultation_id_internal_fail_consultation_id_is_zero() {
        let consultation_id = 0;
        let rf1 = create_dummy_refund1(consultation_id);
        let op_mock = RefundOperationMock {
            consultation_id,
            refunds: vec![rf1],
        };

        let result = get_refund_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_refund_by_consultation_id_internal_fail_consultation_id_is_negative() {
        let consultation_id = -1;
        let rf1 = create_dummy_refund1(consultation_id);
        let op_mock = RefundOperationMock {
            consultation_id,
            refunds: vec![rf1],
        };

        let result = get_refund_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }
}
