// Copyright 2022 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, Datelike, Duration, FixedOffset, Timelike, Utc};
use common::rating::{calculate_average_rating, round_rating_to_one_decimal_places};
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::authentication::authenticated_handlers::authenticated_users::verified_user::VerifiedUser;
use crate::handlers::session::authentication::authenticated_handlers::consultation::{
    consultation_req_exists, ConsultationDateTime, ConsultationRequest,
};
use crate::optional_env_var::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS;

use super::validate_consultation_req_id_is_positive;

pub(crate) async fn get_consultation_request_detail(
    VerifiedUser { user_info }: VerifiedUser,
    query: Query<ConsultationRequestDetailQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<ConsultationRequestDetail> {
    let consultation_req_id = query.consultation_req_id;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = ConsultationRequestDetailOperationImpl { pool };
    handle_consultation_request_detail(
        user_info.account_id,
        consultation_req_id,
        &current_date_time,
        op,
    )
    .await
}

#[derive(Deserialize)]
pub(crate) struct ConsultationRequestDetailQuery {
    consultation_req_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationRequestDetail {
    consultation_req_id: i64,
    user_account_id: i64,
    user_rating: Option<String>, // 適切な型は浮動少数だが、PartialEqの==を正しく動作させるために文字列として処理する
    num_of_rated_of_user: i32,
    fee_per_hour_in_yen: i32,
    first_candidate_in_jst: ConsultationDateTime,
    second_candidate_in_jst: ConsultationDateTime,
    third_candidate_in_jst: ConsultationDateTime,
}

async fn handle_consultation_request_detail(
    user_account_id: i64,
    consultation_req_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    op: impl ConsultationRequestDetailOperation,
) -> RespResult<ConsultationRequestDetail> {
    validate_consultation_req_id_is_positive(consultation_req_id)?;

    let req = op
        .find_consultation_req_by_consultation_req_id(consultation_req_id)
        .await?;
    let req = consultation_req_exists(req, consultation_req_id)?;
    validate_consultation_req_for_reference(&req, user_account_id, current_date_time)?;

    let user_ratings = op
        .filter_user_rating_by_user_account_id(req.user_account_id)
        .await?;
    let (rating, count) = calculate_rating_and_count(user_ratings)?;
    Ok((
        StatusCode::OK,
        Json(ConsultationRequestDetail {
            consultation_req_id: req.consultation_req_id,
            user_account_id: req.user_account_id,
            user_rating: rating,
            num_of_rated_of_user: count,
            fee_per_hour_in_yen: req.fee_per_hour_in_yen,
            first_candidate_in_jst: ConsultationDateTime {
                year: req.first_candidate_date_time_in_jst.year(),
                month: req.first_candidate_date_time_in_jst.month(),
                day: req.first_candidate_date_time_in_jst.day(),
                hour: req.first_candidate_date_time_in_jst.hour(),
            },
            second_candidate_in_jst: ConsultationDateTime {
                year: req.second_candidate_date_time_in_jst.year(),
                month: req.second_candidate_date_time_in_jst.month(),
                day: req.second_candidate_date_time_in_jst.day(),
                hour: req.second_candidate_date_time_in_jst.hour(),
            },
            third_candidate_in_jst: ConsultationDateTime {
                year: req.third_candidate_date_time_in_jst.year(),
                month: req.third_candidate_date_time_in_jst.month(),
                day: req.third_candidate_date_time_in_jst.day(),
                hour: req.third_candidate_date_time_in_jst.hour(),
            },
        }),
    ))
}

#[async_trait]
trait ConsultationRequestDetailOperation {
    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp>;

    async fn filter_user_rating_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<i16>, ErrResp>;
}

struct ConsultationRequestDetailOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationRequestDetailOperation for ConsultationRequestDetailOperationImpl {
    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp> {
        super::super::find_consultation_req_by_consultation_req_id(&self.pool, consultation_req_id)
            .await
    }

    async fn filter_user_rating_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<i16>, ErrResp> {
        let models = entity::user_rating::Entity::find()
            .filter(entity::user_rating::Column::UserAccountId.eq(user_account_id))
            .filter(entity::user_rating::Column::Rating.is_not_null())
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter user_rating (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        models
            .into_iter()
            .map(|m| {
                let r = m.rating.ok_or_else(|| {
                    // NOT NULL 条件で検索しているのでNULLの場合（＝ない場合）はエラー
                    error!(
                        "rating is null (consultation_id: {}, user_account_id: {})",
                        m.consultation_id, m.user_account_id
                    );
                    unexpected_err_resp()
                })?;
                Ok(r)
            })
            .collect::<Result<Vec<i16>, ErrResp>>()
    }
}

fn validate_consultation_req_for_reference(
    consultation_req: &ConsultationRequest,
    consultant_id: i64,
    current_date_time: &DateTime<FixedOffset>,
) -> Result<(), ErrResp> {
    if consultation_req.consultant_id != consultant_id {
        error!(
            "consultant_id ({}) does not match consultation_req.consultant_id ({})",
            consultant_id, consultation_req.consultant_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationReqFound as u32,
            }),
        ));
    }
    let criteria = *current_date_time
        + Duration::seconds(*MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64);
    if consultation_req.latest_candidate_date_time_in_jst <= criteria {
        error!(
            "latest candidate ({}) is not over criteria ({})",
            consultation_req.latest_candidate_date_time_in_jst, criteria
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationReqFound as u32,
            }),
        ));
    }
    Ok(())
}

fn calculate_rating_and_count(user_ratings: Vec<i16>) -> Result<(Option<String>, i32), ErrResp> {
    let count = user_ratings.len();
    let rating_option = calculate_average_rating(user_ratings);
    if let Some(rating) = rating_option {
        let rating_str = round_rating_to_one_decimal_places(rating);
        Ok((Some(rating_str), count as i32))
    } else {
        Ok((None, 0))
    }
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;
    use once_cell::sync::Lazy;

    use super::*;

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<ConsultationRequestDetail>,
    }

    #[derive(Debug)]
    struct Input {
        user_account_id: i64,
        consultation_req_id: i64,
        current_date_time: DateTime<FixedOffset>,
        op: ConsultationRequestDetailOperationMock,
    }

    impl Input {
        fn new(
            account_id_of_consultant: i64,
            account_id_of_user: i64,
            consultation_req_id: i64,
            current_date_time: DateTime<FixedOffset>,
            req: Option<ConsultationRequest>,
            user_ratings: Vec<i16>,
        ) -> Self {
            Input {
                user_account_id: account_id_of_consultant,
                consultation_req_id,
                current_date_time,
                op: ConsultationRequestDetailOperationMock {
                    account_id_of_user,
                    consultation_req_id,
                    req,
                    user_ratings,
                },
            }
        }
    }

    #[derive(Clone, Debug)]
    struct ConsultationRequestDetailOperationMock {
        account_id_of_user: i64,
        consultation_req_id: i64,
        req: Option<ConsultationRequest>,
        user_ratings: Vec<i16>,
    }

    #[async_trait]
    impl ConsultationRequestDetailOperation for ConsultationRequestDetailOperationMock {
        async fn find_consultation_req_by_consultation_req_id(
            &self,
            consultation_req_id: i64,
        ) -> Result<Option<ConsultationRequest>, ErrResp> {
            assert_eq!(self.consultation_req_id, consultation_req_id);
            Ok(self.req.clone())
        }

        async fn filter_user_rating_by_user_account_id(
            &self,
            user_account_id: i64,
        ) -> Result<Vec<i16>, ErrResp> {
            assert_eq!(self.account_id_of_user, user_account_id);
            Ok(self.user_ratings.clone())
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        let account_id_of_consultant = 1;
        let account_id_of_user = 2;
        let consultation_req_id = 1;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 11, 28, 7, 31, 54)
            .unwrap();
        let fee_per_hour_in_yen = 6000;
        vec![
            TestCase {
                name: "success case 1 (no user rating found)".to_string(),
                input: Input::new(
                    account_id_of_consultant,
                    account_id_of_user,
                    consultation_req_id,
                    current_date_time,
                    Some(ConsultationRequest {
                        consultation_req_id,
                        user_account_id: account_id_of_user,
                        consultant_id: account_id_of_consultant,
                        fee_per_hour_in_yen,
                        first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 7, 0, 0)
                            .unwrap(),
                        second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 23, 0, 0)
                            .unwrap(),
                        third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                        latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                    }),
                    vec![],
                ),
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultationRequestDetail {
                        consultation_req_id,
                        user_account_id: account_id_of_user,
                        user_rating: None,
                        num_of_rated_of_user: 0,
                        fee_per_hour_in_yen,
                        first_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 5,
                            hour: 7,
                        },
                        second_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 5,
                            hour: 23,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 11,
                            hour: 7,
                        },
                    }),
                )),
            },
            TestCase {
                name: "success case 2 (user rating found)".to_string(),
                input: Input::new(
                    account_id_of_consultant,
                    account_id_of_user,
                    consultation_req_id,
                    current_date_time,
                    Some(ConsultationRequest {
                        consultation_req_id,
                        user_account_id: account_id_of_user,
                        consultant_id: account_id_of_consultant,
                        fee_per_hour_in_yen,
                        first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 7, 0, 0)
                            .unwrap(),
                        second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 23, 0, 0)
                            .unwrap(),
                        third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                        latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                    }),
                    vec![5],
                ),
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultationRequestDetail {
                        consultation_req_id,
                        user_account_id: account_id_of_user,
                        user_rating: Some("5.0".to_string()),
                        num_of_rated_of_user: 1,
                        fee_per_hour_in_yen,
                        first_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 5,
                            hour: 7,
                        },
                        second_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 5,
                            hour: 23,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 11,
                            hour: 7,
                        },
                    }),
                )),
            },
            TestCase {
                name: "success case 3 (2 user ratings found)".to_string(),
                input: Input::new(
                    account_id_of_consultant,
                    account_id_of_user,
                    consultation_req_id,
                    current_date_time,
                    Some(ConsultationRequest {
                        consultation_req_id,
                        user_account_id: account_id_of_user,
                        consultant_id: account_id_of_consultant,
                        fee_per_hour_in_yen,
                        first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 7, 0, 0)
                            .unwrap(),
                        second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 23, 0, 0)
                            .unwrap(),
                        third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                        latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                    }),
                    vec![5, 2],
                ),
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultationRequestDetail {
                        consultation_req_id,
                        user_account_id: account_id_of_user,
                        user_rating: Some("3.5".to_string()),
                        num_of_rated_of_user: 2,
                        fee_per_hour_in_yen,
                        first_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 5,
                            hour: 7,
                        },
                        second_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 5,
                            hour: 23,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 11,
                            hour: 7,
                        },
                    }),
                )),
            },
            TestCase {
                name: "success case 3 (3 user ratings found)".to_string(),
                input: Input::new(
                    account_id_of_consultant,
                    account_id_of_user,
                    consultation_req_id,
                    current_date_time,
                    Some(ConsultationRequest {
                        consultation_req_id,
                        user_account_id: account_id_of_user,
                        consultant_id: account_id_of_consultant,
                        fee_per_hour_in_yen,
                        first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 7, 0, 0)
                            .unwrap(),
                        second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 23, 0, 0)
                            .unwrap(),
                        third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                        latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                    }),
                    vec![5, 2, 3],
                ),
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultationRequestDetail {
                        consultation_req_id,
                        user_account_id: account_id_of_user,
                        user_rating: Some("3.3".to_string()),
                        num_of_rated_of_user: 3,
                        fee_per_hour_in_yen,
                        first_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 5,
                            hour: 7,
                        },
                        second_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 5,
                            hour: 23,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 11,
                            hour: 7,
                        },
                    }),
                )),
            },
            TestCase {
                name: "success case 4 (3 user ratings found with Nones)".to_string(),
                input: Input::new(
                    account_id_of_consultant,
                    account_id_of_user,
                    consultation_req_id,
                    current_date_time,
                    Some(ConsultationRequest {
                        consultation_req_id,
                        user_account_id: account_id_of_user,
                        consultant_id: account_id_of_consultant,
                        fee_per_hour_in_yen,
                        first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 7, 0, 0)
                            .unwrap(),
                        second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 23, 0, 0)
                            .unwrap(),
                        third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                        latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                    }),
                    vec![5, 2, 3],
                ),
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultationRequestDetail {
                        consultation_req_id,
                        user_account_id: account_id_of_user,
                        user_rating: Some("3.3".to_string()),
                        num_of_rated_of_user: 3,
                        fee_per_hour_in_yen,
                        first_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 5,
                            hour: 7,
                        },
                        second_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 5,
                            hour: 23,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 11,
                            hour: 7,
                        },
                    }),
                )),
            },
            TestCase {
                name: "fail NonPositiveConsultationReqId (id: 0)".to_string(),
                input: Input::new(
                    account_id_of_consultant,
                    account_id_of_user,
                    0,
                    current_date_time,
                    Some(ConsultationRequest {
                        consultation_req_id: -1,
                        user_account_id: account_id_of_user,
                        consultant_id: account_id_of_consultant,
                        fee_per_hour_in_yen,
                        first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 7, 0, 0)
                            .unwrap(),
                        second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 23, 0, 0)
                            .unwrap(),
                        third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                        latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                    }),
                    vec![5, 2, 3],
                ),
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NonPositiveConsultationReqId as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NonPositiveConsultationReqId (id: -1)".to_string(),
                input: Input::new(
                    account_id_of_consultant,
                    account_id_of_user,
                    -1,
                    current_date_time,
                    Some(ConsultationRequest {
                        consultation_req_id: -1,
                        user_account_id: account_id_of_user,
                        consultant_id: account_id_of_consultant,
                        fee_per_hour_in_yen,
                        first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 7, 0, 0)
                            .unwrap(),
                        second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 23, 0, 0)
                            .unwrap(),
                        third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                        latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                    }),
                    vec![5, 2, 3],
                ),
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NonPositiveConsultationReqId as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoConsultationReqFound (no consultation request found)".to_string(),
                input: Input::new(
                    account_id_of_consultant,
                    account_id_of_user,
                    consultation_req_id,
                    current_date_time,
                    None,
                    vec![5, 2, 3],
                ),
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoConsultationReqFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoConsultationReqFound (consultant id does not match)".to_string(),
                input: Input::new(
                    account_id_of_consultant,
                    account_id_of_user,
                    consultation_req_id,
                    current_date_time,
                    Some(ConsultationRequest {
                        consultation_req_id,
                        user_account_id: account_id_of_user,
                        consultant_id: account_id_of_consultant + 1,
                        fee_per_hour_in_yen,
                        first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 7, 0, 0)
                            .unwrap(),
                        second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 5, 23, 0, 0)
                            .unwrap(),
                        third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                        latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2022, 12, 11, 7, 0, 0)
                            .unwrap(),
                    }),
                    vec![5, 2, 3],
                ),
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoConsultationReqFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoConsultationReqFound (current date time is within 6 hours of latest candidate date time)".to_string(),
                input: Input::new(
                    account_id_of_consultant,
                    account_id_of_user,
                    consultation_req_id,
                    JAPANESE_TIME_ZONE.with_ymd_and_hms(2022, 12, 11, 1, 0, 0).unwrap(),
                    Some(ConsultationRequest {
                        consultation_req_id,
                        user_account_id: account_id_of_user,
                        consultant_id: account_id_of_consultant,
                        fee_per_hour_in_yen,
                        first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2022, 12, 5, 7, 0, 0).unwrap(),
                        second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2022, 12, 5, 23, 0, 0).unwrap(),
                        third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2022, 12, 11, 7, 0, 0).unwrap(),
                        latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2022, 12, 11, 7, 0, 0).unwrap(),
                    }),
                    vec![5, 2, 3],
                ),
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoConsultationReqFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "success case 5 (current date time is more than 7 days before latest candidate date time)".to_string(),
                input: Input::new(
                    account_id_of_consultant,
                    account_id_of_user,
                    consultation_req_id,
                    JAPANESE_TIME_ZONE.with_ymd_and_hms(2022, 12, 4, 6, 59, 59).unwrap(),
                    Some(ConsultationRequest {
                        consultation_req_id,
                        user_account_id: account_id_of_user,
                        consultant_id: account_id_of_consultant,
                        fee_per_hour_in_yen,
                        first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2022, 12, 5, 7, 0, 0).unwrap(),
                        second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2022, 12, 5, 23, 0, 0).unwrap(),
                        third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2022, 12, 11, 7, 0, 0).unwrap(),
                        latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2022, 12, 11, 7, 0, 0).unwrap(),
                    }),
                    vec![5, 2, 3],
                ),
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultationRequestDetail {
                        consultation_req_id,
                        user_account_id: account_id_of_user,
                        user_rating: Some("3.3".to_string()),
                        num_of_rated_of_user: 3,
                        fee_per_hour_in_yen,
                        first_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 5,
                            hour: 7,
                        },
                        second_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 5,
                            hour: 23,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 12,
                            day: 11,
                            hour: 7,
                        },
                    }),
                )),
            },
        ]
    });

    #[tokio::test]
    async fn test_handle_consultation_request_detail() {
        for test_case in TEST_CASE_SET.iter() {
            let user_account_id = test_case.input.user_account_id;
            let consultation_req_id = test_case.input.consultation_req_id;
            let current_date_time = test_case.input.current_date_time;
            let op = test_case.input.op.clone();

            let result = handle_consultation_request_detail(
                user_account_id,
                consultation_req_id,
                &current_date_time,
                op,
            )
            .await;

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
