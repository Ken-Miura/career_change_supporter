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

pub(crate) async fn get_stopped_settlement_by_consultation_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<ConsultationIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<StoppedSettlementResult> {
    let query = query.0;
    let op = StoppedSettlementOperationImpl { pool };
    get_stopped_settlement_by_consultation_id_internal(query.consultation_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct StoppedSettlementResult {
    stopped_settlement: Option<StoppedSettlement>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct StoppedSettlement {
    stopped_settlement_id: i64,
    consultation_id: i64,
    charge_id: String,
    fee_per_hour_in_yen: i32,
    platform_fee_rate_in_percentage: String,
    credit_facilities_expired_at: String, // RFC 3339形式の文字列
    stopped_at: String,                   // RFC 3339形式の文字列
}

async fn get_stopped_settlement_by_consultation_id_internal(
    consultation_id: i64,
    op: impl StoppedSettlementOperation,
) -> RespResult<StoppedSettlementResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    let stopped_settlements = op
        .get_stopped_settlements_by_consultation_id(consultation_id)
        .await?;
    if stopped_settlements.len() > 1 {
        error!(
            "{} stopped_settlements found (consultation_id: {})",
            stopped_settlements.len(),
            consultation_id
        );
        return Err(unexpected_err_resp());
    }
    let stopped_settlement = stopped_settlements.get(0).cloned();
    Ok((
        StatusCode::OK,
        Json(StoppedSettlementResult { stopped_settlement }),
    ))
}

#[async_trait]
trait StoppedSettlementOperation {
    async fn get_stopped_settlements_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Vec<StoppedSettlement>, ErrResp>;
}

struct StoppedSettlementOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl StoppedSettlementOperation for StoppedSettlementOperationImpl {
    async fn get_stopped_settlements_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Vec<StoppedSettlement>, ErrResp> {
        let models = entity::stopped_settlement::Entity::find()
            .filter(entity::stopped_settlement::Column::ConsultationId.eq(consultation_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter stopped_settlement (consultation_id: {}): {}",
                    consultation_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| StoppedSettlement {
                stopped_settlement_id: m.stopped_settlement_id,
                consultation_id: m.consultation_id,
                charge_id: m.charge_id,
                fee_per_hour_in_yen: m.fee_per_hour_in_yen,
                platform_fee_rate_in_percentage: m.platform_fee_rate_in_percentage,
                credit_facilities_expired_at: m
                    .credit_facilities_expired_at
                    .with_timezone(&(*JAPANESE_TIME_ZONE))
                    .to_rfc3339(),
                stopped_at: m
                    .stopped_at
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

    struct StoppedSettlementOperationMock {
        consultation_id: i64,
        stopped_settlements: Vec<StoppedSettlement>,
    }

    #[async_trait]
    impl StoppedSettlementOperation for StoppedSettlementOperationMock {
        async fn get_stopped_settlements_by_consultation_id(
            &self,
            consultation_id: i64,
        ) -> Result<Vec<StoppedSettlement>, ErrResp> {
            if self.consultation_id != consultation_id {
                return Ok(vec![]);
            }
            Ok(self.stopped_settlements.clone())
        }
    }

    fn create_dummy_stopped_settlement1(consultation_id: i64) -> StoppedSettlement {
        StoppedSettlement {
            stopped_settlement_id: 10,
            consultation_id,
            charge_id: "ch_6ebea6645ba1bb27307032b23cd5d".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: "2023-04-13T14:00:00.0000+09:00 ".to_string(),
            stopped_at: "2023-03-01T14:00:00.0000+09:00 ".to_string(),
        }
    }

    fn create_dummy_stopped_settlement2(consultation_id: i64) -> StoppedSettlement {
        StoppedSettlement {
            stopped_settlement_id: 12,
            consultation_id,
            charge_id: "ch_7ebea6645ba1bb27307032b23cd5d".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: "2023-04-13T14:00:00.0000+09:00 ".to_string(),
            stopped_at: "2023-03-01T14:00:00.0000+09:00 ".to_string(),
        }
    }

    #[tokio::test]

    async fn get_stopped_settlement_by_consultation_id_internal_success_1_result() {
        let consultation_id = 64431;
        let st1 = create_dummy_stopped_settlement1(consultation_id);
        let op_mock = StoppedSettlementOperationMock {
            consultation_id,
            stopped_settlements: vec![st1.clone()],
        };

        let result =
            get_stopped_settlement_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Some(st1), resp.1 .0.stopped_settlement);
    }

    #[tokio::test]

    async fn get_stopped_settlement_by_consultation_id_internal_success_no_result() {
        let consultation_id = 64431;
        let st1 = create_dummy_stopped_settlement1(consultation_id);
        let op_mock = StoppedSettlementOperationMock {
            consultation_id,
            stopped_settlements: vec![st1],
        };
        let dummy_id = consultation_id + 501;

        let result = get_stopped_settlement_by_consultation_id_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(None, resp.1 .0.stopped_settlement);
    }

    #[tokio::test]

    async fn get_stopped_settlement_by_consultation_id_internal_fail_multiple_results() {
        let consultation_id = 64431;
        let st1 = create_dummy_stopped_settlement1(consultation_id);
        let st2 = create_dummy_stopped_settlement2(consultation_id);
        let op_mock = StoppedSettlementOperationMock {
            consultation_id,
            stopped_settlements: vec![st1, st2],
        };

        let result =
            get_stopped_settlement_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(unexpected_err_resp().0, resp.0);
        assert_eq!(unexpected_err_resp().1 .0.code, resp.1 .0.code);
    }

    #[tokio::test]
    async fn get_stopped_settlement_by_consultation_id_internal_fail_consultation_id_is_zero() {
        let consultation_id = 0;
        let st1 = create_dummy_stopped_settlement1(consultation_id);
        let op_mock = StoppedSettlementOperationMock {
            consultation_id,
            stopped_settlements: vec![st1],
        };

        let result =
            get_stopped_settlement_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_stopped_settlement_by_consultation_id_internal_fail_consultation_id_is_negative() {
        let consultation_id = -1;
        let st1 = create_dummy_stopped_settlement1(consultation_id);
        let op_mock = StoppedSettlementOperationMock {
            consultation_id,
            stopped_settlements: vec![st1],
        };

        let result =
            get_stopped_settlement_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }
}
