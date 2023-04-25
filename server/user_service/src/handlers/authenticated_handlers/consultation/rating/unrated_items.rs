// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, Datelike, Duration, FixedOffset, Timelike, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::consultation;
use entity::prelude::ConsultantRating;
use entity::prelude::UserRating;
use entity::{
    consultant_rating,
    sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect},
    user_rating,
};
use serde::Serialize;
use tracing::error;

use crate::handlers::authenticated_handlers::consultation::ConsultationDateTime;
use crate::{
    err::unexpected_err_resp,
    util::{request_consultation::LENGTH_OF_MEETING_IN_MINUTE, session::user::User},
};

const MAX_NUM_OF_UNRATED_CONSULTANTS: u64 = 20;
const MAX_NUM_OF_UNRATED_USERS: u64 = 20;

pub(crate) async fn get_unrated_items(
    User { user_info }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<UnratedItemsResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = UnratedItemsOperationImpl { pool };
    handle_unrated_items(user_info.account_id, &current_date_time, op).await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct UnratedItemsResult {
    unrated_consultants: Vec<UnratedConsultant>,
    unrated_users: Vec<UnratedUser>,
}

/// 未評価のコンサルタント
#[derive(Clone, Debug, Serialize, PartialEq)]
struct UnratedConsultant {
    consultant_rating_id: i64,
    consultant_id: i64,
    meeting_date_time_in_jst: ConsultationDateTime, // （ユーザーとして）このコンサルタントと相談した日時
}

/// 未評価のユーザー
#[derive(Clone, Debug, Serialize, PartialEq)]
struct UnratedUser {
    user_rating_id: i64,
    user_account_id: i64,
    meeting_date_time_in_jst: ConsultationDateTime, // （コンサルタントとして）このユーザーと相談した日時
}

// 身分のチェックが出来ていなければ、そもそも相談の申込みができない
// 相談の申込みが出来ていなければ、未評価の項目は何もない
// 従って身分のチェックができていないユーザーは空の結果が返るだけなので
// わざわざ身分チェックをする処理を入れない
#[async_trait]
trait UnratedItemsOperation {
    /// コンサルタントに対する未評価のレコードを取得する
    /// 取得するレコードは、最大[MAX_NUM_OF_UNRATED_CONSULTANTS]件で相談日時で昇順にソート済
    /// コンサルタントを評価するのはユーザーなのでuser_account_idでフィルターする
    async fn filter_unrated_consultants_by_user_account_id(
        &self,
        user_account_id: i64,
        start_criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<UnratedConsultant>, ErrResp>;

    /// ユーザーに対する未評価のレコードを取得する
    /// 取得するレコードは、最大[MAX_NUM_OF_UNRATED_USERS]件で相談日時で昇順にソート済
    /// ユーザーを評価するのはコンサルタントなのでconsultant_idでフィルターする
    async fn filter_unrated_users_by_consultant_id(
        &self,
        consultant_id: i64,
        start_criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<UnratedUser>, ErrResp>;
}

struct UnratedItemsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl UnratedItemsOperation for UnratedItemsOperationImpl {
    async fn filter_unrated_consultants_by_user_account_id(
        &self,
        user_account_id: i64,
        start_criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<UnratedConsultant>, ErrResp> {
        let results = consultation::Entity::find()
            .filter(consultation::Column::MeetingAt.lt(start_criteria))
            .filter(consultation::Column::UserAccountId.eq(user_account_id))
            .find_with_related(ConsultantRating)
            .filter(consultant_rating::Column::Rating.is_null()) // null -> まだ未評価であるもの
            .limit(MAX_NUM_OF_UNRATED_CONSULTANTS)
            .order_by_asc(consultation::Column::MeetingAt)
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                "failed to filter consultation and consultant_rating (user_account_id: {}, start_criteria: {}): {}", user_account_id, start_criteria, e);
                unexpected_err_resp()
            })?;
        results
            .into_iter()
            .map(|m| {
                let c = m.0;
                // consultationとconsultant_ratingは1対1の設計なので取れない場合は想定外エラーとして扱う
                let cr = m.1.get(0).ok_or_else(|| {
                    error!(
                        "failed to find consultant_rating (consultant_id: {})",
                        c.consultation_id
                    );
                    unexpected_err_resp()
                })?;
                let meeting_at_in_jst = c.meeting_at.with_timezone(&*JAPANESE_TIME_ZONE);
                Ok(UnratedConsultant {
                    consultant_rating_id: cr.consultant_rating_id,
                    consultant_id: c.consultant_id,
                    meeting_date_time_in_jst: ConsultationDateTime {
                        year: meeting_at_in_jst.year(),
                        month: meeting_at_in_jst.month(),
                        day: meeting_at_in_jst.day(),
                        hour: meeting_at_in_jst.hour(),
                    },
                })
            })
            .collect::<Result<Vec<UnratedConsultant>, ErrResp>>()
    }

    async fn filter_unrated_users_by_consultant_id(
        &self,
        consultant_id: i64,
        start_criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<UnratedUser>, ErrResp> {
        let results = consultation::Entity::find()
            .filter(consultation::Column::MeetingAt.lt(start_criteria))
            .filter(consultation::Column::ConsultantId.eq(consultant_id))
            .find_with_related(UserRating)
            .filter(user_rating::Column::Rating.is_null()) // null -> まだ未評価であるもの
            .limit(MAX_NUM_OF_UNRATED_USERS)
            .order_by_asc(consultation::Column::MeetingAt)
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter consultation and user_rating (consultant_id: {}, start_criteria: {}): {}",
                    consultant_id, start_criteria, e
                );
                unexpected_err_resp()
            })?;
        results
            .into_iter()
            .map(|m| {
                let c = m.0;
                // consultationとuser_ratingは1対1の設計なので取れない場合は想定外エラーとして扱う
                let ur = m.1.get(0).ok_or_else(|| {
                    error!(
                        "failed to find user_rating (consultant_id: {})",
                        c.consultation_id
                    );
                    unexpected_err_resp()
                })?;
                let meeting_at_in_jst = c.meeting_at.with_timezone(&*JAPANESE_TIME_ZONE);
                Ok(UnratedUser {
                    user_rating_id: ur.user_rating_id,
                    user_account_id: c.user_account_id,
                    meeting_date_time_in_jst: ConsultationDateTime {
                        year: meeting_at_in_jst.year(),
                        month: meeting_at_in_jst.month(),
                        day: meeting_at_in_jst.day(),
                        hour: meeting_at_in_jst.hour(),
                    },
                })
            })
            .collect::<Result<Vec<UnratedUser>, ErrResp>>()
    }
}

async fn handle_unrated_items(
    account_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    op: impl UnratedItemsOperation,
) -> RespResult<UnratedItemsResult> {
    let length_of_meeting_in_minute = Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64);
    let criteria = *current_date_time - length_of_meeting_in_minute;

    let unrated_users = op
        .filter_unrated_users_by_consultant_id(account_id, criteria)
        .await?;
    let unrated_consultants = op
        .filter_unrated_consultants_by_user_account_id(account_id, criteria)
        .await?;

    Ok((
        StatusCode::OK,
        Json(UnratedItemsResult {
            unrated_users,
            unrated_consultants,
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

    use crate::handlers::authenticated_handlers::consultation::ConsultationDateTime;
    use crate::util::request_consultation::LENGTH_OF_MEETING_IN_MINUTE;

    use super::{
        handle_unrated_items, UnratedConsultant, UnratedItemsOperation, UnratedItemsResult,
        UnratedUser,
    };

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<UnratedItemsResult>,
    }

    #[derive(Debug)]
    struct Input {
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
        op: UnratedItemsOperationMock,
    }

    #[derive(Clone, Debug)]
    struct UnratedItemsOperationMock {
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
        unrated_consultants: Vec<UnratedConsultant>,
        unrated_users: Vec<UnratedUser>,
    }

    #[async_trait]
    impl UnratedItemsOperation for UnratedItemsOperationMock {
        async fn filter_unrated_consultants_by_user_account_id(
            &self,
            user_account_id: i64,
            start_criteria: DateTime<FixedOffset>,
        ) -> Result<Vec<UnratedConsultant>, ErrResp> {
            assert_eq!(self.account_id, user_account_id);
            let criteria =
                self.current_date_time - Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64);
            assert_eq!(criteria, start_criteria);
            Ok(self.unrated_consultants.clone())
        }

        async fn filter_unrated_users_by_consultant_id(
            &self,
            consultant_id: i64,
            start_criteria: DateTime<FixedOffset>,
        ) -> Result<Vec<UnratedUser>, ErrResp> {
            assert_eq!(self.account_id, consultant_id);
            let criteria =
                self.current_date_time - Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64);
            assert_eq!(criteria, start_criteria);
            Ok(self.unrated_users.clone())
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
                    op: UnratedItemsOperationMock {
                        account_id,
                        current_date_time,
                        unrated_consultants: vec![],
                        unrated_users: vec![],
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(UnratedItemsResult {
                        unrated_consultants: vec![],
                        unrated_users: vec![],
                    }),
                )),
            },
            TestCase {
                name: "1 unrated consultant, no unrated users".to_string(),
                input: Input {
                    account_id,
                    current_date_time,
                    op: UnratedItemsOperationMock {
                        account_id,
                        current_date_time,
                        unrated_consultants: vec![create_dummy_unrated_consultant1()],
                        unrated_users: vec![],
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(UnratedItemsResult {
                        unrated_consultants: vec![create_dummy_unrated_consultant1()],
                        unrated_users: vec![],
                    }),
                )),
            },
            TestCase {
                name: "2 unrated consultants, no unrated users".to_string(),
                input: Input {
                    account_id,
                    current_date_time,
                    op: UnratedItemsOperationMock {
                        account_id,
                        current_date_time,
                        unrated_consultants: vec![
                            create_dummy_unrated_consultant1(),
                            create_dummy_unrated_consultant2(),
                        ],
                        unrated_users: vec![],
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(UnratedItemsResult {
                        unrated_consultants: vec![
                            create_dummy_unrated_consultant1(),
                            create_dummy_unrated_consultant2(),
                        ],
                        unrated_users: vec![],
                    }),
                )),
            },
            TestCase {
                name: "no unrated consultants, 1 unrated user".to_string(),
                input: Input {
                    account_id,
                    current_date_time,
                    op: UnratedItemsOperationMock {
                        account_id,
                        current_date_time,
                        unrated_consultants: vec![],
                        unrated_users: vec![create_dummy_unrated_user1()],
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(UnratedItemsResult {
                        unrated_consultants: vec![],
                        unrated_users: vec![create_dummy_unrated_user1()],
                    }),
                )),
            },
            TestCase {
                name: "no unrated consultants, 2 unrated users".to_string(),
                input: Input {
                    account_id,
                    current_date_time,
                    op: UnratedItemsOperationMock {
                        account_id,
                        current_date_time,
                        unrated_consultants: vec![],
                        unrated_users: vec![
                            create_dummy_unrated_user1(),
                            create_dummy_unrated_user2(),
                        ],
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(UnratedItemsResult {
                        unrated_consultants: vec![],
                        unrated_users: vec![
                            create_dummy_unrated_user1(),
                            create_dummy_unrated_user2(),
                        ],
                    }),
                )),
            },
            TestCase {
                name: "1 unrated consultant, 1 unrated user".to_string(),
                input: Input {
                    account_id,
                    current_date_time,
                    op: UnratedItemsOperationMock {
                        account_id,
                        current_date_time,
                        unrated_consultants: vec![create_dummy_unrated_consultant1()],
                        unrated_users: vec![create_dummy_unrated_user1()],
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(UnratedItemsResult {
                        unrated_consultants: vec![create_dummy_unrated_consultant1()],
                        unrated_users: vec![create_dummy_unrated_user1()],
                    }),
                )),
            },
            TestCase {
                name: "2 unrated consultants, 2 unrated users".to_string(),
                input: Input {
                    account_id,
                    current_date_time,
                    op: UnratedItemsOperationMock {
                        account_id,
                        current_date_time,
                        unrated_consultants: vec![
                            create_dummy_unrated_consultant1(),
                            create_dummy_unrated_consultant2(),
                        ],
                        unrated_users: vec![
                            create_dummy_unrated_user1(),
                            create_dummy_unrated_user2(),
                        ],
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(UnratedItemsResult {
                        unrated_consultants: vec![
                            create_dummy_unrated_consultant1(),
                            create_dummy_unrated_consultant2(),
                        ],
                        unrated_users: vec![
                            create_dummy_unrated_user1(),
                            create_dummy_unrated_user2(),
                        ],
                    }),
                )),
            },
        ]
    });

    fn create_dummy_unrated_consultant1() -> UnratedConsultant {
        UnratedConsultant {
            consultant_rating_id: 70671,
            consultant_id: 234,
            meeting_date_time_in_jst: ConsultationDateTime {
                year: 2023,
                month: 1,
                day: 26,
                hour: 8,
            },
        }
    }

    fn create_dummy_unrated_consultant2() -> UnratedConsultant {
        UnratedConsultant {
            consultant_rating_id: 670,
            consultant_id: 111,
            meeting_date_time_in_jst: ConsultationDateTime {
                year: 2022,
                month: 12,
                day: 23,
                hour: 15,
            },
        }
    }

    fn create_dummy_unrated_user1() -> UnratedUser {
        UnratedUser {
            user_rating_id: 5761,
            user_account_id: 10,
            meeting_date_time_in_jst: ConsultationDateTime {
                year: 2023,
                month: 2,
                day: 25,
                hour: 8,
            },
        }
    }

    fn create_dummy_unrated_user2() -> UnratedUser {
        UnratedUser {
            user_rating_id: 4107,
            user_account_id: 12,
            meeting_date_time_in_jst: ConsultationDateTime {
                year: 2023,
                month: 2,
                day: 26,
                hour: 22,
            },
        }
    }

    #[tokio::test]
    async fn handle_unrated_items_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let account_id = test_case.input.account_id;
            let current_date_time = test_case.input.current_date_time;
            let op = test_case.input.op.clone();

            let result = handle_unrated_items(account_id, &current_date_time, op).await;

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
