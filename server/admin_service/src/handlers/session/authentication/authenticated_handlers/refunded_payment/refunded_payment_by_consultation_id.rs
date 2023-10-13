// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;

use super::super::admin::Admin;
use super::super::{validate_consultation_id_is_positive, ConsultationIdQuery};
use super::RefundedPayment;

pub(crate) async fn get_refunded_payment_by_consultation_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<ConsultationIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<RefundedPaymentResult> {
    let query = query.0;
    let op = RefundedPaymentOperationImpl { pool };
    get_refunded_payment_by_consultation_id_internal(query.consultation_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct RefundedPaymentResult {
    refunded_payment: Option<RefundedPayment>,
}

async fn get_refunded_payment_by_consultation_id_internal(
    consultation_id: i64,
    op: impl RefundedPaymentOperation,
) -> RespResult<RefundedPaymentResult> {
    validate_consultation_id_is_positive(consultation_id)?;

    let refunded_payment = op
        .get_refunded_payment_by_consultation_id(consultation_id)
        .await?;

    Ok((
        StatusCode::OK,
        Json(RefundedPaymentResult { refunded_payment }),
    ))
}

#[async_trait]
trait RefundedPaymentOperation {
    async fn get_refunded_payment_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<RefundedPayment>, ErrResp>;
}

struct RefundedPaymentOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl RefundedPaymentOperation for RefundedPaymentOperationImpl {
    async fn get_refunded_payment_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<RefundedPayment>, ErrResp> {
        let model = entity::refunded_payment::Entity::find_by_id(consultation_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter refunded_payment (consultation_id: {}): {}",
                    consultation_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| RefundedPayment {
            consultation_id: m.consultation_id,
            user_account_id: m.user_account_id,
            consultant_id: m.consultant_id,
            meeting_at: m
                .meeting_at
                .with_timezone(&(*JAPANESE_TIME_ZONE))
                .to_rfc3339(),
            fee_per_hour_in_yen: m.fee_per_hour_in_yen,
            transfer_fee_in_yen: m.transfer_fee_in_yen,
            sender_name: m.sender_name,
            reason: m.reason,
            refund_confirmed_by: m.refund_confirmed_by,
            created_at: m
                .created_at
                .with_timezone(&(*JAPANESE_TIME_ZONE))
                .to_rfc3339(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::DateTime;
    use common::ErrResp;

    use crate::err::Code;
    use crate::handlers::session::authentication::authenticated_handlers::generate_sender_name;

    use super::*;

    struct RefundedPaymentOperationMock {
        consultation_id: i64,
        refunded_payment: RefundedPayment,
    }

    #[async_trait]
    impl RefundedPaymentOperation for RefundedPaymentOperationMock {
        async fn get_refunded_payment_by_consultation_id(
            &self,
            consultation_id: i64,
        ) -> Result<Option<RefundedPayment>, ErrResp> {
            if self.consultation_id != consultation_id {
                return Ok(None);
            }
            Ok(Some(self.refunded_payment.clone()))
        }
    }

    fn create_dummy_refunded_payment(consultation_id: i64) -> RefundedPayment {
        RefundedPayment {
            consultation_id,
            user_account_id: 123,
            consultant_id: 456,
            meeting_at: "2023-04-13T14:00:00.0000+09:00".to_string(),
            fee_per_hour_in_yen: 5000,
            transfer_fee_in_yen: 250,
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                DateTime::parse_from_rfc3339("2023-04-13T14:00:00.0000+09:00")
                    .expect("failed to get Ok"),
            )
            .expect("failed to get Ok"),
            reason: "テスト".to_string(),
            refund_confirmed_by: "admin@test.com".to_string(),
            created_at: "2023-04-28T14:00:00.0000+09:00".to_string(),
        }
    }

    #[tokio::test]

    async fn get_refunded_payment_by_consultation_id_internal_success_1_result() {
        let consultation_id = 64431;
        let rp1 = create_dummy_refunded_payment(consultation_id);
        let op_mock = RefundedPaymentOperationMock {
            consultation_id,
            refunded_payment: rp1.clone(),
        };

        let result =
            get_refunded_payment_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Some(rp1), resp.1 .0.refunded_payment);
    }

    #[tokio::test]

    async fn get_refunded_payment_by_consultation_id_internal_success_no_result() {
        let consultation_id = 64431;
        let rp1 = create_dummy_refunded_payment(consultation_id);
        let op_mock = RefundedPaymentOperationMock {
            consultation_id,
            refunded_payment: rp1,
        };
        let dummy_id = consultation_id + 501;

        let result = get_refunded_payment_by_consultation_id_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(None, resp.1 .0.refunded_payment);
    }

    #[tokio::test]
    async fn get_refunded_payment_by_consultation_id_internal_fail_consultation_id_is_zero() {
        let consultation_id = 0;
        let rp1 = create_dummy_refunded_payment(consultation_id);
        let op_mock = RefundedPaymentOperationMock {
            consultation_id,
            refunded_payment: rp1,
        };

        let result =
            get_refunded_payment_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_refunded_payment_by_consultation_id_internal_fail_consultation_id_is_negative() {
        let consultation_id = -1;
        let rp1 = create_dummy_refunded_payment(consultation_id);
        let op_mock = RefundedPaymentOperationMock {
            consultation_id,
            refunded_payment: rp1,
        };

        let result =
            get_refunded_payment_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }
}
