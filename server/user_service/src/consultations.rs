// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::extract::State;
use chrono::{DateTime, FixedOffset, Utc};
use common::{RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::util::{consultation::ConsultationDateTime, session::User};

pub(crate) async fn get_consultations(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<ConsultationsResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = ConsultationsOperationImpl { pool };
    handle_consultations(account_id, &current_date_time, op).await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationsResult {
    pub(crate) user_side_consultations: Vec<UserSideConsultation>,
    pub(crate) consultant_side_consultations: Vec<ConsultantSideConsultation>,
}

/// 相談申し込み者として行う相談
#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct UserSideConsultation {
    pub(crate) consultation_id: i64,
    pub(crate) consultant_id: i64, // 相談相手のユーザーID
    pub(crate) meeting_date_time_in_jst: ConsultationDateTime,
}

/// 相談相手として行う相談
#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultantSideConsultation {
    pub(crate) consultation_id: i64,
    pub(crate) user_account_id: i64, // 相談申し込み者のユーザーID
    pub(crate) meeting_date_time_in_jst: ConsultationDateTime,
}

async fn handle_consultations(
    account_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    op: impl ConsultationsOperation,
) -> RespResult<ConsultationsResult> {
    todo!()
}

#[async_trait]
trait ConsultationsOperation {}

struct ConsultationsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationsOperation for ConsultationsOperationImpl {}
