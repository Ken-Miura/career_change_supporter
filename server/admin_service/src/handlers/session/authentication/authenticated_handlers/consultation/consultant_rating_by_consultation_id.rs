// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;

use super::super::admin::Admin;
use super::{validate_consultation_id_is_positive, ConsultationIdQuery};

pub(crate) async fn get_consultant_rating_by_consultation_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<ConsultationIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<ConsultantRatingResult> {
    let query = query.0;
    let op = ConsultantRatingOperationImpl { pool };
    get_consultant_rating_by_consultation_id_internal(query.consultation_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConsultantRatingResult {
    consultant_rating: Option<ConsultantRating>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ConsultantRating {
    consultation_id: i64,
    rating: Option<i16>,
    rated_at: Option<String>, // RFC 3339形式の文字列
}

async fn get_consultant_rating_by_consultation_id_internal(
    consultation_id: i64,
    op: impl ConsultantRatingOperation,
) -> RespResult<ConsultantRatingResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    let consultant_ratings = op
        .get_consultant_ratings_by_consultation_id(consultation_id)
        .await?;
    if consultant_ratings.len() > 1 {
        error!(
            "{} consultant_ratings found (consultation_id: {})",
            consultant_ratings.len(),
            consultation_id
        );
        return Err(unexpected_err_resp());
    }
    let consultant_rating = consultant_ratings.get(0).cloned();
    Ok((
        StatusCode::OK,
        Json(ConsultantRatingResult { consultant_rating }),
    ))
}

#[async_trait]
trait ConsultantRatingOperation {
    async fn get_consultant_ratings_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Vec<ConsultantRating>, ErrResp>;
}

struct ConsultantRatingOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultantRatingOperation for ConsultantRatingOperationImpl {
    async fn get_consultant_ratings_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Vec<ConsultantRating>, ErrResp> {
        let models = entity::consultant_rating::Entity::find()
            .filter(entity::consultant_rating::Column::ConsultationId.eq(consultation_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter consultant_rating (consultation_id: {}): {}",
                    consultation_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| ConsultantRating {
                consultation_id: m.consultation_id,
                rating: m.rating,
                rated_at: m
                    .rated_at
                    .map(|dt| dt.with_timezone(&(*JAPANESE_TIME_ZONE)).to_rfc3339()),
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::ErrResp;

    use crate::err::Code;

    use super::*;

    struct ConsultantRatingOperationMock {
        consultation_id: i64,
        consultant_ratings: Vec<ConsultantRating>,
    }

    #[async_trait]
    impl ConsultantRatingOperation for ConsultantRatingOperationMock {
        async fn get_consultant_ratings_by_consultation_id(
            &self,
            consultation_id: i64,
        ) -> Result<Vec<ConsultantRating>, ErrResp> {
            if self.consultation_id != consultation_id {
                return Ok(vec![]);
            }
            Ok(self.consultant_ratings.clone())
        }
    }

    fn create_dummy_consultant_rating1(consultation_id: i64) -> ConsultantRating {
        ConsultantRating {
            consultation_id,
            rating: Some(3),
            rated_at: Some("2023-04-13T14:00:00.0000+09:00 ".to_string()),
        }
    }

    fn create_dummy_consultant_rating2(consultation_id: i64) -> ConsultantRating {
        ConsultantRating {
            consultation_id,
            rating: Some(3),
            rated_at: Some("2023-04-15T14:00:00.0000+09:00 ".to_string()),
        }
    }

    #[tokio::test]

    async fn get_consultant_rating_by_consultation_id_internal_success_1_result() {
        let consultation_id = 64431;
        let cr1 = create_dummy_consultant_rating1(consultation_id);
        let op_mock = ConsultantRatingOperationMock {
            consultation_id,
            consultant_ratings: vec![cr1.clone()],
        };

        let result =
            get_consultant_rating_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Some(cr1), resp.1 .0.consultant_rating);
    }

    #[tokio::test]

    async fn get_consultant_rating_by_consultation_id_internal_success_no_result() {
        let consultation_id = 64431;
        let cr1 = create_dummy_consultant_rating1(consultation_id);
        let op_mock = ConsultantRatingOperationMock {
            consultation_id,
            consultant_ratings: vec![cr1],
        };
        let dummy_id = consultation_id + 501;

        let result = get_consultant_rating_by_consultation_id_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(None, resp.1 .0.consultant_rating);
    }

    #[tokio::test]

    async fn get_consultant_rating_by_consultation_id_internal_fail_multiple_results() {
        let consultation_id = 64431;
        let cr1 = create_dummy_consultant_rating1(consultation_id);
        let cr2 = create_dummy_consultant_rating2(consultation_id);
        let op_mock = ConsultantRatingOperationMock {
            consultation_id,
            consultant_ratings: vec![cr1, cr2],
        };

        let result =
            get_consultant_rating_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(unexpected_err_resp().0, resp.0);
        assert_eq!(unexpected_err_resp().1 .0.code, resp.1 .0.code);
    }

    #[tokio::test]
    async fn get_consultant_rating_by_consultation_id_internal_fail_consultation_id_is_zero() {
        let consultation_id = 0;
        let cr1 = create_dummy_consultant_rating1(consultation_id);
        let op_mock = ConsultantRatingOperationMock {
            consultation_id,
            consultant_ratings: vec![cr1],
        };

        let result =
            get_consultant_rating_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_consultant_rating_by_consultation_id_internal_fail_consultation_id_is_negative() {
        let consultation_id = -1;
        let cr1 = create_dummy_consultant_rating1(consultation_id);
        let op_mock = ConsultantRatingOperationMock {
            consultation_id,
            consultant_ratings: vec![cr1],
        };

        let result =
            get_consultant_rating_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }
}
