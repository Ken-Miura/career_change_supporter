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
use super::ReceiptOfConsultation;

pub(crate) async fn get_receipt_of_consultation_by_consultation_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<ConsultationIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<ReceiptOfConsultationResult> {
    let query = query.0;
    let op = ReceiptOfConsultationOperationImpl { pool };
    get_receipt_of_consultation_by_consultation_id_internal(query.consultation_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReceiptOfConsultationResult {
    receipt_of_consultation: Option<ReceiptOfConsultation>,
}

async fn get_receipt_of_consultation_by_consultation_id_internal(
    consultation_id: i64,
    op: impl ReceiptOfConsultationOperation,
) -> RespResult<ReceiptOfConsultationResult> {
    validate_consultation_id_is_positive(consultation_id)?;

    let receipt_of_consultation = op
        .get_receipt_of_consultation_by_consultation_id(consultation_id)
        .await?;

    Ok((
        StatusCode::OK,
        Json(ReceiptOfConsultationResult {
            receipt_of_consultation,
        }),
    ))
}

#[async_trait]
trait ReceiptOfConsultationOperation {
    async fn get_receipt_of_consultation_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<ReceiptOfConsultation>, ErrResp>;
}

struct ReceiptOfConsultationOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ReceiptOfConsultationOperation for ReceiptOfConsultationOperationImpl {
    async fn get_receipt_of_consultation_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<ReceiptOfConsultation>, ErrResp> {
        let models = entity::receipt_of_consultation::Entity::find_by_id(consultation_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter receipt_of_consultation (consultation_id: {}): {}",
                    consultation_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models.map(|m| ReceiptOfConsultation {
            consultation_id: m.consultation_id,
            user_account_id: m.user_account_id,
            consultant_id: m.consultant_id,
            meeting_at: m
                .meeting_at
                .with_timezone(&(*JAPANESE_TIME_ZONE))
                .to_rfc3339(),
            fee_per_hour_in_yen: m.fee_per_hour_in_yen,
            platform_fee_rate_in_percentage: m.platform_fee_rate_in_percentage,
            transfer_fee_in_yen: m.transfer_fee_in_yen,
            reward: m.reward,
            sender_name: m.sender_name,
            bank_code: m.bank_code,
            branch_code: m.branch_code,
            account_type: m.account_type,
            account_number: m.account_number,
            account_holder_name: m.account_holder_name,
            withdrawal_confirmed_by: m.withdrawal_confirmed_by,
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
    use crate::handlers::session::authentication::authenticated_handlers::{
        calculate_reward, generate_sender_name,
    };

    use super::*;

    struct ReceiptOfConsultationOperationMock {
        consultation_id: i64,
        receipt_of_consultation: ReceiptOfConsultation,
    }

    #[async_trait]
    impl ReceiptOfConsultationOperation for ReceiptOfConsultationOperationMock {
        async fn get_receipt_of_consultation_by_consultation_id(
            &self,
            consultation_id: i64,
        ) -> Result<Option<ReceiptOfConsultation>, ErrResp> {
            if self.consultation_id != consultation_id {
                return Ok(None);
            }
            Ok(Some(self.receipt_of_consultation.clone()))
        }
    }

    fn create_dummy_receipt1(consultation_id: i64) -> ReceiptOfConsultation {
        ReceiptOfConsultation {
            consultation_id,
            user_account_id: 14,
            consultant_id: 68,
            meeting_at: "2023-04-13T14:00:00.0000+09:00 ".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "50.0".to_string(),
            transfer_fee_in_yen: 250,
            reward: calculate_reward(5000, "50.0", 250).expect("failed to get Ok"),
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                DateTime::parse_from_rfc3339("2023-04-13T14:00:00.0000+09:00 ")
                    .expect("failed to get Ok"),
            )
            .expect("failed to get Ok"),
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "タナカ　タロウ".to_string(),
            withdrawal_confirmed_by: "admin@test.com".to_string(),
            created_at: "2023-04-28T14:00:00.0000+09:00 ".to_string(),
        }
    }

    #[tokio::test]

    async fn get_receipt_of_consultation_by_consultation_id_internal_success_1_result() {
        let consultation_id = 64431;
        let rc1 = create_dummy_receipt1(consultation_id);
        let op_mock = ReceiptOfConsultationOperationMock {
            consultation_id,
            receipt_of_consultation: rc1.clone(),
        };

        let result =
            get_receipt_of_consultation_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Some(rc1), resp.1 .0.receipt_of_consultation);
    }

    #[tokio::test]

    async fn get_receipt_of_consultation_by_consultation_id_internal_success_no_result() {
        let consultation_id = 64431;
        let rc1 = create_dummy_receipt1(consultation_id);
        let op_mock = ReceiptOfConsultationOperationMock {
            consultation_id,
            receipt_of_consultation: rc1.clone(),
        };
        let dummy_id = consultation_id + 501;

        let result =
            get_receipt_of_consultation_by_consultation_id_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(None, resp.1 .0.receipt_of_consultation);
    }

    #[tokio::test]
    async fn get_receipt_of_consultation_by_consultation_id_internal_fail_consultation_id_is_zero()
    {
        let consultation_id = 0;
        let rc1 = create_dummy_receipt1(consultation_id);
        let op_mock = ReceiptOfConsultationOperationMock {
            consultation_id,
            receipt_of_consultation: rc1,
        };

        let result =
            get_receipt_of_consultation_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_receipt_of_consultation_by_consultation_id_internal_fail_consultation_id_is_negative(
    ) {
        let consultation_id = -1;
        let rc1 = create_dummy_receipt1(consultation_id);
        let op_mock = ReceiptOfConsultationOperationMock {
            consultation_id,
            receipt_of_consultation: rc1,
        };

        let result =
            get_receipt_of_consultation_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }
}
