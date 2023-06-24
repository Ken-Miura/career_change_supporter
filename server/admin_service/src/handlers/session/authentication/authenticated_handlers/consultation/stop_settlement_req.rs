// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use super::super::admin::Admin;
use super::validate_settlement_id_is_positive;

pub(crate) async fn post_stop_settlement_req(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<StopSettlementReq>,
) -> RespResult<StopSettlementReqResult> {
    let op = StopSettlementReqOperationImpl { pool };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    post_stop_settlement_req_internal(req.settlement_id, current_date_time, op).await
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct StopSettlementReq {
    settlement_id: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct StopSettlementReqResult {}

async fn post_stop_settlement_req_internal(
    settlement_id: i64,
    current_date_time: DateTime<FixedOffset>,
    op: impl StopSettlementReqOperation,
) -> RespResult<StopSettlementReqResult> {
    validate_settlement_id_is_positive(settlement_id)?;

    op.move_to_stopped_settlement(settlement_id, current_date_time)
        .await?;

    Ok((StatusCode::OK, Json(StopSettlementReqResult {})))
}

#[async_trait]
trait StopSettlementReqOperation {
    async fn move_to_stopped_settlement(
        &self,
        settlement_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct StopSettlementReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl StopSettlementReqOperation for StopSettlementReqOperationImpl {
    async fn move_to_stopped_settlement(
        &self,
        settlement_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        super::super::move_to_stopped_settlement(&self.pool, settlement_id, current_date_time).await
    }
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;

    use crate::err::Code;

    use super::*;

    struct StopSettlementReqOperationMock {
        settlement_id: i64,
        current_date_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl StopSettlementReqOperation for StopSettlementReqOperationMock {
        async fn move_to_stopped_settlement(
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

    async fn post_stop_settlement_req_internal_success() {
        let settlement_id = 64431;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let op_mock = StopSettlementReqOperationMock {
            settlement_id,
            current_date_time,
        };

        let result =
            post_stop_settlement_req_internal(settlement_id, current_date_time, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(StopSettlementReqResult {}, resp.1 .0);
    }

    #[tokio::test]

    async fn post_stop_settlement_req_internal_fail_settlement_id_is_zero() {
        let settlement_id = 0;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let op_mock = StopSettlementReqOperationMock {
            settlement_id,
            current_date_time,
        };

        let result =
            post_stop_settlement_req_internal(settlement_id, current_date_time, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::SettlementIdIsNotPositive as u32, resp.1 .0.code);
    }

    #[tokio::test]

    async fn post_stop_settlement_req_internal_fail_settlement_id_is_negative() {
        let settlement_id = -1;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let op_mock = StopSettlementReqOperationMock {
            settlement_id,
            current_date_time,
        };

        let result =
            post_stop_settlement_req_internal(settlement_id, current_date_time, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::SettlementIdIsNotPositive as u32, resp.1 .0.code);
    }
}