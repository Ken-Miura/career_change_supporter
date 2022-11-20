// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::Extension;
use axum::Json;
use chrono::DateTime;
use chrono::Duration;
use chrono::FixedOffset;
use chrono::Utc;
use common::ErrResp;
use common::RespResult;
use common::JAPANESE_TIME_ZONE;
use entity::consultation_req;
use entity::prelude::ConsultationReq;
use entity::sea_orm::ColumnTrait;
use entity::sea_orm::DatabaseConnection;
use entity::sea_orm::EntityTrait;
use entity::sea_orm::QueryFilter;
use entity::sea_orm::QueryOrder;
use entity::sea_orm::QuerySelect;
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;
use crate::util::session::User;
use crate::util::MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE;

const NUM_OF_CONSULTATION_REQUESTS: u64 = 20;

pub(crate) async fn get_consultation_requests(
    User { account_id }: User,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<ConsultationRequestsResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = ConsultationRequestsOperationImpl { pool };
    handle_consultation_requests(account_id, &current_date_time, op).await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationRequestsResult {
    pub(crate) consultation_requests: Vec<ConsultationRequestDescription>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationRequestDescription {
    pub(crate) consultation_req_id: i64,
    pub(crate) user_account_id: i64,
}

async fn handle_consultation_requests(
    account_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    op: impl ConsultationRequestsOperation,
) -> RespResult<ConsultationRequestsResult> {
    let criteria = *current_date_time
        + Duration::hours(*MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE as i64);
    let reqs = op
        .filter_consultation_req(account_id, criteria, NUM_OF_CONSULTATION_REQUESTS)
        .await?;
    Ok((
        StatusCode::OK,
        Json(ConsultationRequestsResult {
            consultation_requests: reqs,
        }),
    ))
}

#[async_trait]
trait ConsultationRequestsOperation {
    /// consultant_idが一致し、latest_candidate_date_timeがcriteriaより未来の時刻である
    /// ConsultationRequestDescriptionをsize個取得する。取得した結果は、latest_candidate_date_timeで昇順に並べ替え済みである。
    async fn filter_consultation_req(
        &self,
        consultant_id: i64,
        criteria: DateTime<FixedOffset>,
        size: u64,
    ) -> Result<Vec<ConsultationRequestDescription>, ErrResp>;
}

struct ConsultationRequestsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationRequestsOperation for ConsultationRequestsOperationImpl {
    async fn filter_consultation_req(
        &self,
        consultant_id: i64,
        criteria: DateTime<FixedOffset>,
        size: u64,
    ) -> Result<Vec<ConsultationRequestDescription>, ErrResp> {
        let models = ConsultationReq::find()
            .filter(consultation_req::Column::LatestCandidateDateTime.gt(criteria))
            .filter(consultation_req::Column::ConsultantId.eq(consultant_id))
            .order_by_asc(consultation_req::Column::LatestCandidateDateTime)
            .limit(size)
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter consultation_req (consultant_id: {}, criteria: {}, size: {}): {}",
                    consultant_id, criteria, size, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| ConsultationRequestDescription {
                consultation_req_id: m.consultation_req_id,
                user_account_id: m.user_account_id,
            })
            .collect::<Vec<ConsultationRequestDescription>>())
    }
}
