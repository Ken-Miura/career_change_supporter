// Copyright 2022 Ken Miura

use axum::http::StatusCode;
use axum::{async_trait, Json};
use axum::{extract::Query, Extension};
use chrono::{DateTime, Datelike, Duration, FixedOffset, Timelike, Utc};
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::prelude::ConsultationReq;
use entity::prelude::UserRating;
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use entity::user_rating;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::util::session::User;
use crate::util::{
    self, round_to_one_decimal_places, ConsultationDateTime,
    MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE,
};

pub(crate) async fn get_consultation_request_detail(
    User { account_id }: User,
    query: Query<ConsultationRequestDetailQuery>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<ConsultationRequestDetail> {
    let consultation_req_id = query.consultation_req_id;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = ConsultationRequestDetailOperationImpl { pool };
    handle_consultation_request_detail(account_id, consultation_req_id, &current_date_time, op)
        .await
}

#[derive(Deserialize)]
pub(crate) struct ConsultationRequestDetailQuery {
    pub(crate) consultation_req_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationRequestDetail {
    pub(crate) consultation_req_id: i64,
    pub(crate) user_account_id: i64,
    pub(crate) user_rating: String, // 適切な型は浮動少数だが、PartialEqの==を正しく動作させるために文字列として処理する
    pub(crate) num_of_rated_of_user: i32,
    pub(crate) fee_per_hour_in_yen: i32,
    pub(crate) first_candidate_in_jst: ConsultationDateTime,
    pub(crate) second_candidate_in_jst: ConsultationDateTime,
    pub(crate) third_candidate_in_jst: ConsultationDateTime,
}

struct ConsultationRequest {
    consultation_req_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    fee_per_hour_in_yen: i32,
    first_candidate_date_time_in_jst: DateTime<FixedOffset>,
    second_candidate_date_time_in_jst: DateTime<FixedOffset>,
    third_candidate_date_time_in_jst: DateTime<FixedOffset>,
    latest_candidate_date_time_in_jst: DateTime<FixedOffset>,
}

async fn handle_consultation_request_detail(
    user_account_id: i64,
    consultation_req_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    op: impl ConsultationRequestDetailOperation,
) -> RespResult<ConsultationRequestDetail> {
    validate_consultation_req_id_is_positive(consultation_req_id)?;
    validate_identity_exists(user_account_id, &op).await?;

    let req = op
        .find_consultation_req_by_consultation_req_id(consultation_req_id)
        .await?;
    let req = consultation_req_exists(req, consultation_req_id)?;
    validate_consultation_req(&req, user_account_id, current_date_time)?;

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
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp>;
    async fn filter_user_rating_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<Option<i16>>, ErrResp>;
}

struct ConsultationRequestDetailOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationRequestDetailOperation for ConsultationRequestDetailOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp> {
        let model = ConsultationReq::find_by_id(consultation_req_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find consultation_req (consultation_req_id: {}): {}",
                    consultation_req_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| ConsultationRequest {
            consultation_req_id: m.consultation_req_id,
            user_account_id: m.user_account_id,
            consultant_id: m.consultant_id,
            fee_per_hour_in_yen: m.fee_per_hour_in_yen,
            first_candidate_date_time_in_jst: m
                .first_candidate_date_time
                .with_timezone(&(*JAPANESE_TIME_ZONE)), // TODO: with_timezoneが必要か確認する
            second_candidate_date_time_in_jst: m
                .second_candidate_date_time
                .with_timezone(&(*JAPANESE_TIME_ZONE)),
            third_candidate_date_time_in_jst: m
                .third_candidate_date_time
                .with_timezone(&(*JAPANESE_TIME_ZONE)),
            latest_candidate_date_time_in_jst: m.latest_candidate_date_time,
        }))
    }

    async fn filter_user_rating_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<Option<i16>>, ErrResp> {
        let models = UserRating::find()
            .filter(user_rating::Column::UserAccountId.eq(user_account_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter user_rating (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models.into_iter().map(|m| m.rating).collect())
    }
}

fn validate_consultation_req_id_is_positive(consultation_req_id: i64) -> Result<(), ErrResp> {
    if !consultation_req_id.is_positive() {
        error!(
            "consultation_req_id ({}) is not positive",
            consultation_req_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonPositiveConsultationReqId as u32,
            }),
        ));
    }
    Ok(())
}

async fn validate_identity_exists(
    account_id: i64,
    op: &impl ConsultationRequestDetailOperation,
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

fn consultation_req_exists(
    consultation_request: Option<ConsultationRequest>,
    consultation_req_id: i64,
) -> Result<ConsultationRequest, ErrResp> {
    let req = consultation_request.ok_or_else(|| {
        error!(
            "no consultation_req (consultation_req_id: {}) found",
            consultation_req_id
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonConsultationReqFound as u32,
            }),
        )
    })?;
    Ok(req)
}

fn validate_consultation_req(
    consultation_req: &ConsultationRequest,
    consultant_id: i64,
    current_date_time: &DateTime<FixedOffset>,
) -> Result<(), ErrResp> {
    if consultation_req.consultant_id != consultant_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonConsultationReqFound as u32,
            }),
        ));
    }
    let criteria = *current_date_time
        + Duration::hours(*MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE as i64);
    if consultation_req.latest_candidate_date_time_in_jst <= criteria {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonConsultationReqFound as u32,
            }),
        ));
    }
    Ok(())
}

fn calculate_rating_and_count(user_ratings: Vec<Option<i16>>) -> Result<(String, i32), ErrResp> {
    let filled_user_ratings = user_ratings
        .into_iter()
        .filter(|u| u.is_some())
        .collect::<Vec<Option<i16>>>();
    let count = filled_user_ratings.len();
    let mut sum = 0_i32;
    for u in filled_user_ratings {
        match u {
            Some(u) => sum += u as i32,
            None => {
                error!("failed to calculate sum of user rating");
                return Err(unexpected_err_resp());
            }
        }
    }
    let rating = sum as f64 / count as f64;
    let rating_str = round_to_one_decimal_places(rating);
    Ok((rating_str, count as i32))
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use chrono::{DateTime, FixedOffset};
    use common::{ErrResp, RespResult};
    use once_cell::sync::Lazy;

    use super::{
        handle_consultation_request_detail, ConsultationRequest, ConsultationRequestDetail,
        ConsultationRequestDetailOperation,
    };

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

    #[derive(Clone, Debug)]
    struct ConsultationRequestDetailOperationMock {}

    #[async_trait]
    impl ConsultationRequestDetailOperation for ConsultationRequestDetailOperationMock {
        async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
            todo!()
        }

        async fn find_consultation_req_by_consultation_req_id(
            &self,
            consultation_req_id: i64,
        ) -> Result<Option<ConsultationRequest>, ErrResp> {
            todo!()
        }

        async fn filter_user_rating_by_user_account_id(
            &self,
            user_account_id: i64,
        ) -> Result<Vec<Option<i16>>, ErrResp> {
            todo!()
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| vec![]);

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
