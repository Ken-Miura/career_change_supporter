// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use common::smtp::{SmtpClient, SMTP_HOST, SMTP_PASSWORD, SMTP_PORT, SMTP_USERNAME};
use common::{smtp::SendMail, RespResult, JAPANESE_TIME_ZONE};
use common::{ApiError, ErrResp};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::Code;
use crate::util::session::User;
use crate::util::{
    self, consultation_req_exists, validate_consultation_req_id_is_positive, ConsultationRequest,
    MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE,
};

pub(crate) async fn post_consultation_request_acceptance(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
    Json(param): Json<ConsultationRequestAcceptanceParam>,
) -> RespResult<ConsultationRequestAcceptanceResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = ConsultationRequestAcceptanceOperationImpl { pool };
    let smtp_client = SmtpClient::new(
        SMTP_HOST.to_string(),
        *SMTP_PORT,
        SMTP_USERNAME.to_string(),
        SMTP_PASSWORD.to_string(),
    );
    handle_consultation_request_acceptance(account_id, &param, &current_date_time, op, smtp_client)
        .await
}

#[derive(Deserialize)]
pub(crate) struct ConsultationRequestAcceptanceParam {
    pub(crate) consultation_req_id: i64,
    pub(crate) picked_candidate: u8,
    pub(crate) user_checked: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationRequestAcceptanceResult {}

async fn handle_consultation_request_acceptance(
    user_account_id: i64,
    param: &ConsultationRequestAcceptanceParam,
    current_date_time: &DateTime<FixedOffset>,
    op: impl ConsultationRequestAcceptanceOperation,
    send_mail: impl SendMail,
) -> RespResult<ConsultationRequestAcceptanceResult> {
    let consultation_req_id = param.consultation_req_id;
    validate_consultation_req_id_is_positive(consultation_req_id)?;
    validate_identity_exists(user_account_id, &op).await?;

    let req = op
        .find_consultation_req_by_consultation_req_id(consultation_req_id)
        .await?;
    let req = consultation_req_exists(req, consultation_req_id)?;
    validate_consultation_req_for_acceptance(&req, user_account_id, current_date_time)?;

    todo!()
}

#[async_trait]
trait ConsultationRequestAcceptanceOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;

    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp>;
}

struct ConsultationRequestAcceptanceOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationRequestAcceptanceOperation for ConsultationRequestAcceptanceOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp> {
        util::find_consultation_req_by_consultation_req_id(&self.pool, consultation_req_id).await
    }
}

async fn validate_identity_exists(
    account_id: i64,
    op: &impl ConsultationRequestAcceptanceOperation,
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

fn validate_consultation_req_for_acceptance(
    consultation_req: &ConsultationRequest,
    consultant_id: i64,
    current_date_time: &DateTime<FixedOffset>,
) -> Result<(), ErrResp> {
    if consultation_req.consultant_id != consultant_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationReqFound as u32,
            }),
        ));
    }
    let criteria = *current_date_time
        + Duration::hours(*MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE as i64);
    if consultation_req.latest_candidate_date_time_in_jst <= criteria {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationReqFound as u32,
            }),
        ));
    }
    Ok(())
}
