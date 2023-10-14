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
use super::LeftAwaitingWithdrawal;

pub(crate) async fn get_left_awaiting_withdrawal_by_consultation_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<ConsultationIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<LeftAwaitingWithdrawalResult> {
    let query = query.0;
    let op = LeftAwaitingWithdrawalOperationImpl { pool };
    get_left_awaiting_withdrawal_by_consultation_id_internal(query.consultation_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct LeftAwaitingWithdrawalResult {
    left_awaiting_withdrawal: Option<LeftAwaitingWithdrawal>,
}

async fn get_left_awaiting_withdrawal_by_consultation_id_internal(
    consultation_id: i64,
    op: impl LeftAwaitingWithdrawalOperation,
) -> RespResult<LeftAwaitingWithdrawalResult> {
    validate_consultation_id_is_positive(consultation_id)?;

    let left_awaiting_withdrawal = op
        .get_left_awaiting_withdrawal_by_consultation_id(consultation_id)
        .await?;

    Ok((
        StatusCode::OK,
        Json(LeftAwaitingWithdrawalResult {
            left_awaiting_withdrawal,
        }),
    ))
}

#[async_trait]
trait LeftAwaitingWithdrawalOperation {
    async fn get_left_awaiting_withdrawal_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<LeftAwaitingWithdrawal>, ErrResp>;
}

struct LeftAwaitingWithdrawalOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl LeftAwaitingWithdrawalOperation for LeftAwaitingWithdrawalOperationImpl {
    async fn get_left_awaiting_withdrawal_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<LeftAwaitingWithdrawal>, ErrResp> {
        let model = entity::left_awaiting_withdrawal::Entity::find_by_id(consultation_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter left_awaiting_withdrawal (consultation_id: {}): {}",
                    consultation_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| LeftAwaitingWithdrawal {
            consultation_id: m.consultation_id,
            user_account_id: m.user_account_id,
            consultant_id: m.consultant_id,
            meeting_at: m
                .meeting_at
                .with_timezone(&(*JAPANESE_TIME_ZONE))
                .to_rfc3339(),
            fee_per_hour_in_yen: m.fee_per_hour_in_yen,
            sender_name: m.sender_name,
            confirmed_by: m.confirmed_by,
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

    struct LeftAwaitingWithdrawalOperationMock {
        consultation_id: i64,
        left_awaiting_withdrawal: LeftAwaitingWithdrawal,
    }

    #[async_trait]
    impl LeftAwaitingWithdrawalOperation for LeftAwaitingWithdrawalOperationMock {
        async fn get_left_awaiting_withdrawal_by_consultation_id(
            &self,
            consultation_id: i64,
        ) -> Result<Option<LeftAwaitingWithdrawal>, ErrResp> {
            if self.consultation_id != consultation_id {
                return Ok(None);
            }
            Ok(Some(self.left_awaiting_withdrawal.clone()))
        }
    }

    fn create_dummy_left_awaiting_withdrawal(consultation_id: i64) -> LeftAwaitingWithdrawal {
        LeftAwaitingWithdrawal {
            consultation_id,
            user_account_id: 14,
            consultant_id: 68,
            meeting_at: "2023-04-13T14:00:00.0000+09:00".to_string(),
            fee_per_hour_in_yen: 5000,
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                DateTime::parse_from_rfc3339("2023-04-13T14:00:00.0000+09:00")
                    .expect("failed to get Ok"),
            )
            .expect("failed to get Ok"),
            confirmed_by: "admin@test.com".to_string(),
            created_at: "2023-04-28T14:00:00.0000+09:00 ".to_string(),
        }
    }

    #[tokio::test]

    async fn get_left_awaiting_withdrawal_by_consultation_id_internal_success_1_result() {
        let consultation_id = 64431;
        let rc1 = create_dummy_left_awaiting_withdrawal(consultation_id);
        let op_mock = LeftAwaitingWithdrawalOperationMock {
            consultation_id,
            left_awaiting_withdrawal: rc1.clone(),
        };

        let result =
            get_left_awaiting_withdrawal_by_consultation_id_internal(consultation_id, op_mock)
                .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Some(rc1), resp.1 .0.left_awaiting_withdrawal);
    }

    #[tokio::test]

    async fn get_left_awaiting_withdrawal_by_consultation_id_internal_success_no_result() {
        let consultation_id = 64431;
        let rc1 = create_dummy_left_awaiting_withdrawal(consultation_id);
        let op_mock = LeftAwaitingWithdrawalOperationMock {
            consultation_id,
            left_awaiting_withdrawal: rc1.clone(),
        };
        let dummy_id = consultation_id + 501;

        let result =
            get_left_awaiting_withdrawal_by_consultation_id_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(None, resp.1 .0.left_awaiting_withdrawal);
    }

    #[tokio::test]
    async fn get_left_awaiting_withdrawal_by_consultation_id_internal_fail_consultation_id_is_zero()
    {
        let consultation_id = 0;
        let rc1 = create_dummy_left_awaiting_withdrawal(consultation_id);
        let op_mock = LeftAwaitingWithdrawalOperationMock {
            consultation_id,
            left_awaiting_withdrawal: rc1,
        };

        let result =
            get_left_awaiting_withdrawal_by_consultation_id_internal(consultation_id, op_mock)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_left_awaiting_withdrawal_by_consultation_id_internal_fail_consultation_id_is_negative(
    ) {
        let consultation_id = -1;
        let rc1 = create_dummy_left_awaiting_withdrawal(consultation_id);
        let op_mock = LeftAwaitingWithdrawalOperationMock {
            consultation_id,
            left_awaiting_withdrawal: rc1,
        };

        let result =
            get_left_awaiting_withdrawal_by_consultation_id_internal(consultation_id, op_mock)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }
}
