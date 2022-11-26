// Copyright 2022 Ken Miura

use axum::http::StatusCode;
use axum::{async_trait, Json};
use axum::{extract::Query, Extension};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::prelude::ConsultationReq;
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::util::session::User;
use crate::util::{
    self, ConsultationDateTime, MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE,
};

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

struct ConsultationRequest {
    consultation_req_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    fee_per_hour_in_yen: i32,
    first_candidate_date_time_in_jst: DateTime<FixedOffset>,
    second_candidate_date_time_in_jst: DateTime<FixedOffset>,
    third_candidate_date_time_in_jst: DateTime<FixedOffset>,
    charge_id: String,
    latest_candidate_date_time_in_jst: DateTime<FixedOffset>,
}

async fn handle_consultation_request_detail(
    user_account_id: i64,
    consultation_req_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    op: impl ConsultationRequestDetailOperation,
) -> RespResult<ConsultationRequestDetail> {
    validate_consultation_req_id_is_positive(consultation_req_id)?;
    validate_identity_exists(user_account_id, &op).await?;

    let req = op
        .find_consultation_req_by_consultation_req_id(consultation_req_id)
        .await?;
    let req = consultation_req_exists(req, consultation_req_id)?;
    validate_consultation_req(&req, user_account_id, current_date_time)?;
    // TODO: userが存在するかどうか＋無効化されているかどうかチェック
    todo!()
}

#[async_trait]
trait ConsultationRequestDetailOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp>;
}

struct ConsultationRequestDetailOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationRequestDetailOperation for ConsultationRequestDetailOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp> {
        let model = ConsultationReq::find_by_id(consultation_req_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find consultation_req (consultation_req_id: {}): {}",
                    consultation_req_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| ConsultationRequest {
            consultation_req_id: m.consultation_req_id,
            user_account_id: m.user_account_id,
            consultant_id: m.consultant_id,
            fee_per_hour_in_yen: m.fee_per_hour_in_yen,
            first_candidate_date_time_in_jst: m
                .first_candidate_date_time
                .with_timezone(&(*JAPANESE_TIME_ZONE)), // TODO: with_timezoneが必要か確認する
            second_candidate_date_time_in_jst: m
                .second_candidate_date_time
                .with_timezone(&(*JAPANESE_TIME_ZONE)),
            third_candidate_date_time_in_jst: m
                .third_candidate_date_time
                .with_timezone(&(*JAPANESE_TIME_ZONE)),
            charge_id: m.charge_id,
            latest_candidate_date_time_in_jst: m.latest_candidate_date_time,
        }))
    }
}

fn validate_consultation_req_id_is_positive(consultation_req_id: i64) -> Result<(), ErrResp> {
    if !consultation_req_id.is_positive() {
        error!(
            "consultation_req_id ({}) is not positive",
            consultation_req_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonPositiveConsultationReqId as u32,
            }),
        ));
    }
    Ok(())
}

async fn validate_identity_exists(
    account_id: i64,
    op: &impl ConsultationRequestDetailOperation,
) -> Result<(), ErrResp> {
    let identity_exists = op.check_if_identity_exists(account_id).await?;
    if !identity_exists {
        error!("identity is not registered (account_id: {})", account_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoIdentityRegistered as u32,
            }),
        ));
    }
    Ok(())
}

fn consultation_req_exists(
    consultation_request: Option<ConsultationRequest>,
    consultation_req_id: i64,
) -> Result<ConsultationRequest, ErrResp> {
    let req = consultation_request.ok_or_else(|| {
        error!(
            "no consultation_req (consultation_req_id: {}) found",
            consultation_req_id
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonConsultationReqFound as u32,
            }),
        )
    })?;
    Ok(req)
}

fn validate_consultation_req(
    consultation_req: &ConsultationRequest,
    consultant_id: i64,
    current_date_time: &DateTime<FixedOffset>,
) -> Result<(), ErrResp> {
    if consultation_req.consultant_id != consultant_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonConsultationReqFound as u32,
            }),
        ));
    }
    let criteria = *current_date_time
        + Duration::hours(*MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE as i64);
    if consultation_req.latest_candidate_date_time_in_jst <= criteria {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonConsultationReqFound as u32,
            }),
        ));
    }
    Ok(())
}
