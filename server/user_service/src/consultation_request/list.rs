// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::extract::State;
use axum::http::StatusCode;
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
use crate::util::optional_env_var::MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE;
use crate::util::session::user::User;

const NUM_OF_CONSULTATION_REQUESTS: u64 = 20;

pub(crate) async fn get_consultation_requests(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
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

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use chrono::{DateTime, Duration, FixedOffset, TimeZone};
    use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
    use once_cell::sync::Lazy;

    use crate::{
        consultation_request::list::NUM_OF_CONSULTATION_REQUESTS,
        util::optional_env_var::MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE,
    };

    use super::{
        handle_consultation_requests, ConsultationRequestDescription,
        ConsultationRequestsOperation, ConsultationRequestsResult,
    };

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<ConsultationRequestsResult>,
    }

    #[derive(Debug)]
    struct Input {
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
        op: ConsultationRequestsOperationMock,
    }

    impl Input {
        fn new(
            account_id: i64,
            current_date_time: DateTime<FixedOffset>,
            consultant_requests: Vec<ConsultationRequestDescription>,
        ) -> Self {
            Input {
                account_id,
                current_date_time,
                op: ConsultationRequestsOperationMock {
                    consultant_id: account_id,
                    current_date_time,
                    consultant_requests,
                },
            }
        }
    }

    #[derive(Clone, Debug)]
    struct ConsultationRequestsOperationMock {
        consultant_id: i64,
        current_date_time: DateTime<FixedOffset>,
        consultant_requests: Vec<ConsultationRequestDescription>,
    }

    #[async_trait]
    impl ConsultationRequestsOperation for ConsultationRequestsOperationMock {
        async fn filter_consultation_req(
            &self,
            consultant_id: i64,
            criteria: DateTime<FixedOffset>,
            size: u64,
        ) -> Result<Vec<ConsultationRequestDescription>, ErrResp> {
            assert_eq!(self.consultant_id, consultant_id);
            assert_eq!(
                self.current_date_time
                    + Duration::hours(*MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE as i64),
                criteria
            );
            assert_eq!(NUM_OF_CONSULTATION_REQUESTS, size);
            assert!(self.consultant_requests.len() <= size as usize);
            Ok(self.consultant_requests.clone())
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        let account_id = 1;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
            .unwrap();
        vec![
            TestCase {
                name: "success case (empty result)".to_string(),
                input: Input::new(account_id, current_date_time, vec![]),
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultationRequestsResult {
                        consultation_requests: vec![],
                    }),
                )),
            },
            TestCase {
                name: "success case (1 result)".to_string(),
                input: Input::new(
                    account_id,
                    current_date_time,
                    vec![ConsultationRequestDescription {
                        consultation_req_id: 1,
                        user_account_id: 2,
                    }],
                ),
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultationRequestsResult {
                        consultation_requests: vec![ConsultationRequestDescription {
                            consultation_req_id: 1,
                            user_account_id: 2,
                        }],
                    }),
                )),
            },
            TestCase {
                name: "success case (1 result)".to_string(),
                input: Input::new(
                    account_id,
                    current_date_time,
                    vec![ConsultationRequestDescription {
                        consultation_req_id: 1,
                        user_account_id: 2,
                    }],
                ),
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultationRequestsResult {
                        consultation_requests: vec![ConsultationRequestDescription {
                            consultation_req_id: 1,
                            user_account_id: 2,
                        }],
                    }),
                )),
            },
            TestCase {
                // 実際にいくつのデータまで返すかの責任を追っているのは実装（SQLを発行するORM）だが、仕様を表現するためにテストとして記載
                name: "success case (20 results)".to_string(),
                input: Input::new(
                    account_id,
                    current_date_time,
                    create_dummy_consultation_requests(),
                ),
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultationRequestsResult {
                        consultation_requests: create_dummy_consultation_requests(),
                    }),
                )),
            },
        ]
    });

    fn create_dummy_consultation_requests() -> Vec<ConsultationRequestDescription> {
        let mut data = Vec::with_capacity(20);
        // 値は適当。意味はない。要素数が使用上許容する最大値であることに意味がある
        for i in 0..20 {
            data.push(ConsultationRequestDescription {
                consultation_req_id: i + 20,
                user_account_id: i + 20,
            })
        }
        data
    }

    #[tokio::test]
    async fn test_handle_consultation_requests() {
        for test_case in TEST_CASE_SET.iter() {
            let account_id = test_case.input.account_id;
            let current_date_time = test_case.input.current_date_time;
            let op = test_case.input.op.clone();

            let result = handle_consultation_requests(account_id, &current_date_time, op).await;

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
