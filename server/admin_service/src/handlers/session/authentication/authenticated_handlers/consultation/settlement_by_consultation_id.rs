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

pub(crate) async fn get_settlement_by_consultation_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<ConsultationIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<SettlementResult> {
    let query = query.0;
    let op = SettlementOperationImpl { pool };
    get_settlement_by_consultation_id_internal(query.consultation_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct SettlementResult {
    settlement: Option<Settlement>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct Settlement {
    settlement_id: i64,
    consultation_id: i64,
    charge_id: String,
    fee_per_hour_in_yen: i32,
    platform_fee_rate_in_percentage: String,
    credit_facilities_expired_at: String, // RFC 3339形式の文字列
}

async fn get_settlement_by_consultation_id_internal(
    consultation_id: i64,
    op: impl SettlementOperation,
) -> RespResult<SettlementResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    let settlements = op
        .get_settlements_by_consultation_id(consultation_id)
        .await?;
    if settlements.len() > 1 {
        error!(
            "{} settlements found (consultation_id: {})",
            settlements.len(),
            consultation_id
        );
        return Err(unexpected_err_resp());
    }
    let settlement = settlements.get(0).cloned();
    Ok((StatusCode::OK, Json(SettlementResult { settlement })))
}

#[async_trait]
trait SettlementOperation {
    async fn get_settlements_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Vec<Settlement>, ErrResp>;
}

struct SettlementOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl SettlementOperation for SettlementOperationImpl {
    async fn get_settlements_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Vec<Settlement>, ErrResp> {
        let models = entity::settlement::Entity::find()
            .filter(entity::settlement::Column::ConsultationId.eq(consultation_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find settlement (consultation_id: {}): {}",
                    consultation_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| Settlement {
                settlement_id: m.settlement_id,
                consultation_id: m.consultation_id,
                charge_id: m.charge_id,
                fee_per_hour_in_yen: m.fee_per_hour_in_yen,
                platform_fee_rate_in_percentage: m.platform_fee_rate_in_percentage,
                credit_facilities_expired_at: m
                    .credit_facilities_expired_at
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

    struct SettlementOperationMock {
        consultation_id: i64,
        settlements: Vec<Settlement>,
    }

    #[async_trait]
    impl SettlementOperation for SettlementOperationMock {
        async fn get_settlements_by_consultation_id(
            &self,
            consultation_id: i64,
        ) -> Result<Vec<Settlement>, ErrResp> {
            if self.consultation_id != consultation_id {
                return Ok(vec![]);
            }
            Ok(self.settlements.clone())
        }
    }

    fn create_dummy_settlement1(consultation_id: i64) -> Settlement {
        Settlement {
            settlement_id: 10,
            consultation_id,
            charge_id: "ch_6ebea6645ba1bb27307032b23cd5d".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: "2023-04-13T14:00:00.0000+09:00 ".to_string(),
        }
    }

    fn create_dummy_settlement2(consultation_id: i64) -> Settlement {
        Settlement {
            settlement_id: 12,
            consultation_id,
            charge_id: "ch_7ebea6645ba1bb27307032b23cd5d".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.0".to_string(),
            credit_facilities_expired_at: "2023-04-13T14:00:00.0000+09:00 ".to_string(),
        }
    }

    #[tokio::test]

    async fn get_settlement_by_consultation_id_internal_success_1_result() {
        let consultation_id = 64431;
        let st1 = create_dummy_settlement1(consultation_id);
        let op_mock = SettlementOperationMock {
            consultation_id,
            settlements: vec![st1.clone()],
        };

        let result = get_settlement_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Some(st1), resp.1 .0.settlement);
    }

    #[tokio::test]

    async fn get_settlement_by_consultation_id_internal_success_no_result() {
        let consultation_id = 64431;
        let st1 = create_dummy_settlement1(consultation_id);
        let op_mock = SettlementOperationMock {
            consultation_id,
            settlements: vec![st1],
        };
        let dummy_id = consultation_id + 501;

        let result = get_settlement_by_consultation_id_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(None, resp.1 .0.settlement);
    }

    #[tokio::test]

    async fn get_settlement_by_consultation_id_internal_fail_multiple_results() {
        let consultation_id = 64431;
        let st1 = create_dummy_settlement1(consultation_id);
        let st2 = create_dummy_settlement2(consultation_id);
        let op_mock = SettlementOperationMock {
            consultation_id,
            settlements: vec![st1, st2],
        };

        let result = get_settlement_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(unexpected_err_resp().0, resp.0);
        assert_eq!(unexpected_err_resp().1 .0.code, resp.1 .0.code);
    }

    #[tokio::test]
    async fn get_settlement_by_consultation_id_internal_fail_consultation_id_is_zero() {
        let consultation_id = 0;
        let st1 = create_dummy_settlement1(consultation_id);
        let op_mock = SettlementOperationMock {
            consultation_id,
            settlements: vec![st1],
        };

        let result = get_settlement_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_settlement_by_consultation_id_internal_fail_consultation_id_is_negative() {
        let consultation_id = -1;
        let st1 = create_dummy_settlement1(consultation_id);
        let op_mock = SettlementOperationMock {
            consultation_id,
            settlements: vec![st1],
        };

        let result = get_settlement_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }
}
