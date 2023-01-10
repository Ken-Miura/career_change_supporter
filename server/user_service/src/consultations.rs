// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::{async_trait, http::StatusCode, Json};
use chrono::{DateTime, Datelike, Duration, FixedOffset, Timelike, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::{
    consultation,
    sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter},
};
use serde::Serialize;
use tracing::error;

use crate::{
    err::unexpected_err_resp,
    util::{
        consultation::{ConsultationDateTime, LENGTH_OF_MEETING_IN_MINUTE},
        session::User,
    },
};

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
    let length_of_meeting_in_minute = Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64);
    let criteria = *current_date_time - length_of_meeting_in_minute;

    let user_side_consultations = op
        .filter_user_side_consultation(account_id, criteria)
        .await?;

    let consultant_side_consultations = op
        .filter_consultant_side_consultation(account_id, criteria)
        .await?;

    Ok((
        StatusCode::OK,
        Json(ConsultationsResult {
            user_side_consultations,
            consultant_side_consultations,
        }),
    ))
}

#[async_trait]
trait ConsultationsOperation {
    async fn filter_user_side_consultation(
        &self,
        user_account_id: i64,
        criteria_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<UserSideConsultation>, ErrResp>;

    async fn filter_consultant_side_consultation(
        &self,
        consultant_id: i64,
        criteria_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<ConsultantSideConsultation>, ErrResp>;
}

struct ConsultationsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationsOperation for ConsultationsOperationImpl {
    async fn filter_user_side_consultation(
        &self,
        user_account_id: i64,
        criteria_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<UserSideConsultation>, ErrResp> {
        let results = consultation::Entity::find()
        .filter(consultation::Column::MeetingAt.gte(criteria_date_time))
        .filter(consultation::Column::UserAccountId.eq(user_account_id))
        .all(&self.pool)
        .await
        .map_err(|e| {
          error!(
            "failed to filter user side consultation (user_account_id: {}, criteria_date_time: {}): {}",
            user_account_id, criteria_date_time, e
          );
          unexpected_err_resp()
        })?;
        Ok(results
            .into_iter()
            .map(|m| {
                let meeting_at_in_jst = m.meeting_at.with_timezone(&*JAPANESE_TIME_ZONE);
                UserSideConsultation {
                    consultation_id: m.consultation_id,
                    consultant_id: m.consultant_id,
                    meeting_date_time_in_jst: ConsultationDateTime {
                        year: meeting_at_in_jst.year(),
                        month: meeting_at_in_jst.month(),
                        day: meeting_at_in_jst.day(),
                        hour: meeting_at_in_jst.hour(),
                    },
                }
            })
            .collect::<Vec<UserSideConsultation>>())
    }

    async fn filter_consultant_side_consultation(
        &self,
        consultant_id: i64,
        criteria_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<ConsultantSideConsultation>, ErrResp> {
        let results = consultation::Entity::find()
            .filter(consultation::Column::MeetingAt.gte(criteria_date_time))
            .filter(consultation::Column::ConsultantId.eq(consultant_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
              error!(
                  "failed to filter consultant side consultation (consultant_id: {}, criteria_date_time: {}): {}",
                  consultant_id, criteria_date_time, e
              );
              unexpected_err_resp()
          })?;
        Ok(results
            .into_iter()
            .map(|m| {
                let meeting_at_in_jst = m.meeting_at.with_timezone(&*JAPANESE_TIME_ZONE);
                ConsultantSideConsultation {
                    consultation_id: m.consultation_id,
                    user_account_id: m.user_account_id,
                    meeting_date_time_in_jst: ConsultationDateTime {
                        year: meeting_at_in_jst.year(),
                        month: meeting_at_in_jst.month(),
                        day: meeting_at_in_jst.day(),
                        hour: meeting_at_in_jst.hour(),
                    },
                }
            })
            .collect::<Vec<ConsultantSideConsultation>>())
    }
}
