// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::util;
use crate::util::session::User;

pub(crate) async fn post_consultation_request_rejection(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
    Json(param): Json<ConsultationRequestRejectionParam>,
) -> RespResult<ConsultationRequestRejectionResult> {
    let consultation_req_id = param.consultation_req_id;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = ConsultationRequestRejectionImpl { pool };
    handle_consultation_request_rejection(account_id, consultation_req_id, &current_date_time, op)
        .await
}

#[derive(Deserialize)]
pub(crate) struct ConsultationRequestRejectionParam {
    pub(crate) consultation_req_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationRequestRejectionResult {}

async fn handle_consultation_request_rejection(
    account_id: i64,
    consultation_req_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    op: impl ConsultationRequestRejection,
) -> RespResult<ConsultationRequestRejectionResult> {
    todo!()
}

#[async_trait]
trait ConsultationRequestRejection {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
    //   async fn find_consultation_req_by_consultation_req_id(
    //     &self,
    //     consultation_req_id: i64,
    // ) -> Result<Option<ConsultationRequest>, ErrResp>;
}

struct ConsultationRequestRejectionImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationRequestRejection for ConsultationRequestRejectionImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::check_if_identity_exists(&self.pool, account_id).await
    }
}

#[derive(Clone, Debug)]
struct ConsultationRequest {
    consultation_req_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    fee_per_hour_in_yen: i32,
    first_candidate_date_time_in_jst: DateTime<FixedOffset>,
    second_candidate_date_time_in_jst: DateTime<FixedOffset>,
    third_candidate_date_time_in_jst: DateTime<FixedOffset>,
    latest_candidate_date_time_in_jst: DateTime<FixedOffset>,
}
