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
    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use chrono::{DateTime, Duration, FixedOffset, TimeZone};
    use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
    use once_cell::sync::Lazy;

    use crate::util::request_consultation::{ConsultationDateTime, LENGTH_OF_MEETING_IN_MINUTE};

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
    struct AwaitingRatingsOperationMock {
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
        user_side_awaiting_ratings: Vec<UserSideAwaitingRating>,
        consultant_side_awaiting_ratings: Vec<ConsultantSideAwaitingRating>,
    }

    #[async_trait]
    impl AwaitingRatingsOperation for AwaitingRatingsOperationMock {
        async fn filter_user_side_awaiting_ratings(
            &self,
            user_account_id: i64,
            start_criteria: DateTime<FixedOffset>,
        ) -> Result<Vec<UserSideAwaitingRating>, ErrResp> {
            assert_eq!(self.account_id, user_account_id);
            let criteria =
                self.current_date_time - Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64);
            assert_eq!(criteria, start_criteria);
            Ok(self.user_side_awaiting_ratings.clone())
        }

        async fn filter_consultant_side_awaiting_ratings(
            &self,
            consultant_id: i64,
            start_criteria: DateTime<FixedOffset>,
        ) -> Result<Vec<ConsultantSideAwaitingRating>, ErrResp> {
            assert_eq!(self.account_id, consultant_id);
            let criteria =
                self.current_date_time - Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64);
            assert_eq!(criteria, start_criteria);
            Ok(self.consultant_side_awaiting_ratings.clone())
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        let account_id = 560;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 2, 25, 21, 32, 21)
            .unwrap();
        vec![
            TestCase {
                name: "empty results".to_string(),
                input: Input {
                    account_id,
                    current_date_time,
                    op: AwaitingRatingsOperationMock {
                        account_id,
                        current_date_time,
                        user_side_awaiting_ratings: vec![],
                        consultant_side_awaiting_ratings: vec![],
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(AwaitingRatingsResult {
                        user_side_awaiting_ratings: vec![],
                        consultant_side_awaiting_ratings: vec![],
                    }),
                )),
            },
            TestCase {
                name: "1 user side awaiting rating, no consultant side awaiting ratings"
                    .to_string(),
                input: Input {
                    account_id,
                    current_date_time,
                    op: AwaitingRatingsOperationMock {
                        account_id,
                        current_date_time,
                        user_side_awaiting_ratings: vec![create_dummy_user_side_awaiting_rating1()],
                        consultant_side_awaiting_ratings: vec![],
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(AwaitingRatingsResult {
                        user_side_awaiting_ratings: vec![create_dummy_user_side_awaiting_rating1()],
                        consultant_side_awaiting_ratings: vec![],
                    }),
                )),
            },
            TestCase {
                name: "2 user side awaiting ratings, no consultant side awaiting ratings"
                    .to_string(),
                input: Input {
                    account_id,
                    current_date_time,
                    op: AwaitingRatingsOperationMock {
                        account_id,
                        current_date_time,
                        user_side_awaiting_ratings: vec![
                            create_dummy_user_side_awaiting_rating1(),
                            create_dummy_user_side_awaiting_rating2(),
                        ],
                        consultant_side_awaiting_ratings: vec![],
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(AwaitingRatingsResult {
                        user_side_awaiting_ratings: vec![
                            create_dummy_user_side_awaiting_rating1(),
                            create_dummy_user_side_awaiting_rating2(),
                        ],
                        consultant_side_awaiting_ratings: vec![],
                    }),
                )),
            },
            TestCase {
                name: "no user side awaiting ratings, 1 consultant side awaiting ratings"
                    .to_string(),
                input: Input {
                    account_id,
                    current_date_time,
                    op: AwaitingRatingsOperationMock {
                        account_id,
                        current_date_time,
                        user_side_awaiting_ratings: vec![],
                        consultant_side_awaiting_ratings: vec![
                            create_dummy_consultant_side_awaiting_rating1(),
                        ],
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(AwaitingRatingsResult {
                        user_side_awaiting_ratings: vec![],
                        consultant_side_awaiting_ratings: vec![
                            create_dummy_consultant_side_awaiting_rating1(),
                        ],
                    }),
                )),
            },
            TestCase {
                name: "no user side awaiting ratings, 2 consultant side awaiting ratings"
                    .to_string(),
                input: Input {
                    account_id,
                    current_date_time,
                    op: AwaitingRatingsOperationMock {
                        account_id,
                        current_date_time,
                        user_side_awaiting_ratings: vec![],
                        consultant_side_awaiting_ratings: vec![
                            create_dummy_consultant_side_awaiting_rating1(),
                            create_dummy_consultant_side_awaiting_rating2(),
                        ],
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(AwaitingRatingsResult {
                        user_side_awaiting_ratings: vec![],
                        consultant_side_awaiting_ratings: vec![
                            create_dummy_consultant_side_awaiting_rating1(),
                            create_dummy_consultant_side_awaiting_rating2(),
                        ],
                    }),
                )),
            },
        ]
    });

    fn create_dummy_user_side_awaiting_rating1() -> UserSideAwaitingRating {
        UserSideAwaitingRating {
            user_rating_id: 5761,
            consultant_id: 10,
            meeting_date_time_in_jst: ConsultationDateTime {
                year: 2023,
                month: 2,
                day: 25,
                hour: 8,
            },
        }
    }

    fn create_dummy_user_side_awaiting_rating2() -> UserSideAwaitingRating {
        UserSideAwaitingRating {
            user_rating_id: 4107,
            consultant_id: 12,
            meeting_date_time_in_jst: ConsultationDateTime {
                year: 2023,
                month: 2,
                day: 26,
                hour: 22,
            },
        }
    }

    fn create_dummy_consultant_side_awaiting_rating1() -> ConsultantSideAwaitingRating {
        ConsultantSideAwaitingRating {
            consultant_rating_id: 70671,
            user_account_id: 234,
            meeting_date_time_in_jst: ConsultationDateTime {
                year: 2023,
                month: 1,
                day: 26,
                hour: 8,
            },
        }
    }

    fn create_dummy_consultant_side_awaiting_rating2() -> ConsultantSideAwaitingRating {
        ConsultantSideAwaitingRating {
            consultant_rating_id: 670,
            user_account_id: 111,
            meeting_date_time_in_jst: ConsultationDateTime {
                year: 2022,
                month: 12,
                day: 23,
                hour: 15,
            },
        }
    }

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
