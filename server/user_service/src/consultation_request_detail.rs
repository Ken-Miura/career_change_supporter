// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::{extract::Query, Extension};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::util::session::User;
use crate::util::{self, ConsultationDateTime};

pub(crate) async fn get_consultation_request_detail(
    User { account_id }: User,
    query: Query<ConsultationRequestDetailQuery>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<ConsultationRequestDetail> {
    let consultation_req_id = query.consultation_req_id;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = ConsultationRequestDetailOperationImpl { pool };
    handle_consultation_request_detail(account_id, consultation_req_id, &current_date_time, op)
        .await
}

#[derive(Deserialize)]
pub(crate) struct ConsultationRequestDetailQuery {
    pub(crate) consultation_req_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationRequestDetail {
    pub(crate) consultation_req_id: i64,
    pub(crate) user_account_id: i64,
    pub(crate) user_rating: String, // 適切な型は浮動少数だが、PartialEqの==を正しく動作させるために文字列として処理する
    pub(crate) num_of_rated_of_user: i32,
    pub(crate) fee_per_hour_in_yen: i32,
    pub(crate) first_candidate_in_jst: ConsultationDateTime,
    pub(crate) second_candidate_in_jst: ConsultationDateTime,
    pub(crate) third_candidate_in_jst: ConsultationDateTime,
}

async fn handle_consultation_request_detail(
    user_account_id: i64,
    consultation_req_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    op: impl ConsultationRequestDetailOperation,
) -> RespResult<ConsultationRequestDetail> {
    todo!()
}

#[async_trait]
trait ConsultationRequestDetailOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
}

struct ConsultationRequestDetailOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationRequestDetailOperation for ConsultationRequestDetailOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::check_if_identity_exists(&self.pool, account_id).await
    }
}
