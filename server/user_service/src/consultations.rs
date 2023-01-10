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

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use chrono::{DateTime, FixedOffset};
    use common::{ErrResp, RespResult};
    use once_cell::sync::Lazy;

    use super::{
        handle_consultations, ConsultantSideConsultation, ConsultationsOperation,
        ConsultationsResult, UserSideConsultation,
    };

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<ConsultationsResult>,
    }

    #[derive(Debug)]
    struct Input {
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
        op: ConsultationsOperationMock,
    }

    #[derive(Clone, Debug)]
    struct ConsultationsOperationMock {
        user_account_id: i64,
        criteria_date_time: DateTime<FixedOffset>,
        user_side_consultations: Vec<UserSideConsultation>,
        consultant_side_consultations: Vec<ConsultantSideConsultation>,
    }

    #[async_trait]
    impl ConsultationsOperation for ConsultationsOperationMock {
        async fn filter_user_side_consultation(
            &self,
            user_account_id: i64,
            criteria_date_time: DateTime<FixedOffset>,
        ) -> Result<Vec<UserSideConsultation>, ErrResp> {
            assert_eq!(self.user_account_id, user_account_id);
            assert_eq!(self.criteria_date_time, criteria_date_time);
            Ok(self.user_side_consultations.clone())
        }

        async fn filter_consultant_side_consultation(
            &self,
            consultant_id: i64,
            criteria_date_time: DateTime<FixedOffset>,
        ) -> Result<Vec<ConsultantSideConsultation>, ErrResp> {
            assert_eq!(self.user_account_id, consultant_id);
            assert_eq!(self.criteria_date_time, criteria_date_time);
            Ok(self.consultant_side_consultations.clone())
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| vec![]);

    #[tokio::test]
    async fn test_handle_consultations() {
        for test_case in TEST_CASE_SET.iter() {
            let account_id = test_case.input.account_id;
            let current_date_time = test_case.input.current_date_time;
            let op = test_case.input.op.clone();

            let result = handle_consultations(account_id, &current_date_time, op).await;

            let message = format!("test case \"{}\" failed", test_case.name.clone());
            if test_case.expected.is_ok() {
                let resp = result.expect("failed to get Ok");
                let expected = test_case.expected.as_ref().expect("failed to get Ok");
                assert_eq!(expected.0, resp.0, "{}", message);
                assert_eq!(expected.1 .0, resp.1 .0, "{}", message);
            } else {
                let resp = result.expect_err("failed to get Err");
                let expected = test_case.expected.as_ref().expect_err("failed to get Err");
                assert_eq!(expected.0, resp.0, "{}", message);
                assert_eq!(expected.1 .0, resp.1 .0, "{}", message);
            }
        }
    }
}
