// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, Datelike, Duration, FixedOffset, Timelike, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::{
    consultant_rating,
    sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect},
    user_rating,
};
use serde::Serialize;
use tracing::error;

use crate::{
    err::unexpected_err_resp,
    util::{
        request_consultation::{ConsultationDateTime, LENGTH_OF_MEETING_IN_MINUTE},
        session::User,
    },
};

const MAX_NUM_OF_USER_SIDE_AWAITING_RATINGS: u64 = 20;
const MAX_NUM_OF_CONSULTANT_SIDE_AWAITING_RATINGS: u64 = 20;

pub(crate) async fn get_awaiting_ratings(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<AwaitingRatingsResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = AwaitingRatingsOperationImpl { pool };
    handle_awaiting_ratings(account_id, &current_date_time, op).await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct AwaitingRatingsResult {
    user_side_awaiting_ratings: Vec<UserSideAwaitingRating>,
    consultant_side_awaiting_ratings: Vec<ConsultantSideAwaitingRating>,
}

/// 相談申し込み者として行う評価
#[derive(Clone, Debug, Serialize, PartialEq)]
struct UserSideAwaitingRating {
    user_rating_id: i64,
    consultant_id: i64, // 相談相手のユーザーID
    meeting_date_time_in_jst: ConsultationDateTime,
}

/// 相談相手として行う評価
#[derive(Clone, Debug, Serialize, PartialEq)]
struct ConsultantSideAwaitingRating {
    consultant_rating_id: i64,
    user_account_id: i64, // 相談申し込み者のユーザーID
    meeting_date_time_in_jst: ConsultationDateTime,
}

// 身分のチェックが出来ていなければ、そもそも相談の申込みができない
// 相談の申込みが出来ていなければ、評価待ちは何もない
// 従って身分のチェックができていないユーザーは空の結果が返るだけなので
// わざわざ身分チェックをする処理を入れない
#[async_trait]
trait AwaitingRatingsOperation {
    async fn filter_user_side_awaiting_ratings(
        &self,
        user_account_id: i64,
        start_criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<UserSideAwaitingRating>, ErrResp>;

    async fn filter_consultant_side_awaiting_ratings(
        &self,
        consultant_id: i64,
        start_criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<ConsultantSideAwaitingRating>, ErrResp>;
}

struct AwaitingRatingsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl AwaitingRatingsOperation for AwaitingRatingsOperationImpl {
    async fn filter_user_side_awaiting_ratings(
        &self,
        user_account_id: i64,
        start_criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<UserSideAwaitingRating>, ErrResp> {
        let results = user_rating::Entity::find()
            .filter(user_rating::Column::MeetingAt.lt(start_criteria))
            .filter(user_rating::Column::UserAccountId.eq(user_account_id))
            .filter(user_rating::Column::Rating.is_null()) // null -> まだ未評価であるもの
            .limit(MAX_NUM_OF_USER_SIDE_AWAITING_RATINGS)
            .order_by_asc(user_rating::Column::MeetingAt)
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter user_rating (user_account_id: {}, start_criteria: {}): {}",
                    user_account_id, start_criteria, e
                );
                unexpected_err_resp()
            })?;
        Ok(results
            .into_iter()
            .map(|m| {
                let meeting_at_in_jst = m.meeting_at.with_timezone(&*JAPANESE_TIME_ZONE);
                UserSideAwaitingRating {
                    user_rating_id: m.user_rating_id,
                    consultant_id: m.consultant_id,
                    meeting_date_time_in_jst: ConsultationDateTime {
                        year: meeting_at_in_jst.year(),
                        month: meeting_at_in_jst.month(),
                        day: meeting_at_in_jst.day(),
                        hour: meeting_at_in_jst.hour(),
                    },
                }
            })
            .collect::<Vec<UserSideAwaitingRating>>())
    }

    async fn filter_consultant_side_awaiting_ratings(
        &self,
        consultant_id: i64,
        start_criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<ConsultantSideAwaitingRating>, ErrResp> {
        let results = consultant_rating::Entity::find()
            .filter(consultant_rating::Column::MeetingAt.lt(start_criteria))
            .filter(consultant_rating::Column::ConsultantId.eq(consultant_id))
            .filter(consultant_rating::Column::Rating.is_null()) // null -> まだ未評価であるもの
            .limit(MAX_NUM_OF_CONSULTANT_SIDE_AWAITING_RATINGS)
            .order_by_asc(consultant_rating::Column::MeetingAt)
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter consultant_rating (consultant_id: {}, start_criteria: {}): {}",
                    consultant_id, start_criteria, e
                );
                unexpected_err_resp()
            })?;
        Ok(results
            .into_iter()
            .map(|m| {
                let meeting_at_in_jst = m.meeting_at.with_timezone(&*JAPANESE_TIME_ZONE);
                ConsultantSideAwaitingRating {
                    consultant_rating_id: m.consultant_rating_id,
                    user_account_id: m.user_account_id,
                    meeting_date_time_in_jst: ConsultationDateTime {
                        year: meeting_at_in_jst.year(),
                        month: meeting_at_in_jst.month(),
                        day: meeting_at_in_jst.day(),
                        hour: meeting_at_in_jst.hour(),
                    },
                }
            })
            .collect::<Vec<ConsultantSideAwaitingRating>>())
    }
}

async fn handle_awaiting_ratings(
    account_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    op: impl AwaitingRatingsOperation,
) -> RespResult<AwaitingRatingsResult> {
    let length_of_meeting_in_minute = Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64);
    let criteria = *current_date_time - length_of_meeting_in_minute;

    let user_side_awaiting_ratings = op
        .filter_user_side_awaiting_ratings(account_id, criteria)
        .await?;
    let consultant_side_awaiting_ratings = op
        .filter_consultant_side_awaiting_ratings(account_id, criteria)
        .await?;

    Ok((
        StatusCode::OK,
        Json(AwaitingRatingsResult {
            user_side_awaiting_ratings,
            consultant_side_awaiting_ratings,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use chrono::{DateTime, FixedOffset};
    use common::{ErrResp, RespResult};
    use once_cell::sync::Lazy;

    use super::{
        handle_awaiting_ratings, AwaitingRatingsOperation, AwaitingRatingsResult,
        ConsultantSideAwaitingRating, UserSideAwaitingRating,
    };

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<AwaitingRatingsResult>,
    }

    #[derive(Debug)]
    struct Input {
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
        op: AwaitingRatingsOperationMock,
    }

    #[derive(Clone, Debug)]
    struct AwaitingRatingsOperationMock {}

    #[async_trait]
    impl AwaitingRatingsOperation for AwaitingRatingsOperationMock {
        async fn filter_user_side_awaiting_ratings(
            &self,
            user_account_id: i64,
            start_criteria: DateTime<FixedOffset>,
        ) -> Result<Vec<UserSideAwaitingRating>, ErrResp> {
            todo!()
        }

        async fn filter_consultant_side_awaiting_ratings(
            &self,
            consultant_id: i64,
            start_criteria: DateTime<FixedOffset>,
        ) -> Result<Vec<ConsultantSideAwaitingRating>, ErrResp> {
            todo!()
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| vec![]);

    #[tokio::test]
    async fn handle_user_side_info_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let account_id = test_case.input.account_id;
            let current_date_time = test_case.input.current_date_time;
            let op = test_case.input.op.clone();

            let result = handle_awaiting_ratings(account_id, &current_date_time, op).await;

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
