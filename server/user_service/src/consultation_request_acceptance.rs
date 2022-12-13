// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::smtp::{SmtpClient, SMTP_HOST, SMTP_PASSWORD, SMTP_PORT, SMTP_USERNAME};
use common::{smtp::SendMail, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::util::session::User;
use crate::util::validate_consultation_req_id_is_positive;

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
    todo!()
}

#[async_trait]
trait ConsultationRequestAcceptanceOperation {}

struct ConsultationRequestAcceptanceOperationImpl {
    pool: DatabaseConnection,
}

impl ConsultationRequestAcceptanceOperation for ConsultationRequestAcceptanceOperationImpl {}
