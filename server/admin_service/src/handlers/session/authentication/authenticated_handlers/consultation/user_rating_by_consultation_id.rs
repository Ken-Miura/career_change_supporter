// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;

use super::super::admin::Admin;
use super::{validate_consultation_id_is_positive, ConsultationIdQuery};

pub(crate) async fn get_user_rating_by_consultation_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<ConsultationIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<UserRatingResult> {
    let query = query.0;
    let op = UserRatingOperationImpl { pool };
    get_user_rating_by_consultation_id_internal(query.consultation_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct UserRatingResult {
    user_rating: Option<UserRating>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct UserRating {
    user_rating_id: i64,
    consultation_id: i64,
    rating: Option<i16>,
    rated_at: Option<String>, // RFC 3339形式の文字列
}

async fn get_user_rating_by_consultation_id_internal(
    consultation_id: i64,
    op: impl UserRatingOperation,
) -> RespResult<UserRatingResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    let user_rating = op
        .get_user_rating_by_consultation_id(consultation_id)
        .await?;
    Ok((StatusCode::OK, Json(UserRatingResult { user_rating })))
}

#[async_trait]
trait UserRatingOperation {
    async fn get_user_rating_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<UserRating>, ErrResp>;
}

struct UserRatingOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl UserRatingOperation for UserRatingOperationImpl {
    async fn get_user_rating_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<UserRating>, ErrResp> {
        let model = entity::user_rating::Entity::find_by_id(consultation_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find user_rating (consultation_id: {}): {}",
                    consultation_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| UserRating {
            user_rating_id: m.user_rating_id,
            consultation_id: m.consultation_id,
            rating: m.rating,
            rated_at: m
                .rated_at
                .map(|dt| dt.with_timezone(&(*JAPANESE_TIME_ZONE)).to_rfc3339()),
        }))
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::ErrResp;

    use crate::err::Code;

    use super::*;

    struct UserRatingOperationMock {
        consultation_id: i64,
        user_rating: UserRating,
    }

    #[async_trait]
    impl UserRatingOperation for UserRatingOperationMock {
        async fn get_user_rating_by_consultation_id(
            &self,
            consultation_id: i64,
        ) -> Result<Option<UserRating>, ErrResp> {
            if self.consultation_id != consultation_id {
                return Ok(None);
            }
            Ok(Some(self.user_rating.clone()))
        }
    }

    fn create_dummy_user_rating(consultation_id: i64) -> UserRating {
        UserRating {
            user_rating_id: 10,
            consultation_id,
            rating: Some(3),
            rated_at: Some("2023-04-13T14:00:00.0000+09:00 ".to_string()),
        }
    }

    #[tokio::test]

    async fn get_user_rating_by_consultation_id_internal_success_1_result() {
        let consultation_id = 64431;
        let ur = create_dummy_user_rating(consultation_id);
        let op_mock = UserRatingOperationMock {
            consultation_id,
            user_rating: ur.clone(),
        };

        let result = get_user_rating_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Some(ur), resp.1 .0.user_rating);
    }

    #[tokio::test]

    async fn get_user_rating_by_consultation_id_internal_success_no_result() {
        let consultation_id = 64431;
        let ur = create_dummy_user_rating(consultation_id);
        let op_mock = UserRatingOperationMock {
            consultation_id,
            user_rating: ur,
        };
        let dummy_id = consultation_id + 501;

        let result = get_user_rating_by_consultation_id_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(None, resp.1 .0.user_rating);
    }

    #[tokio::test]
    async fn get_user_rating_by_consultation_id_internal_fail_consultation_id_is_zero() {
        let consultation_id = 0;
        let ur = create_dummy_user_rating(consultation_id);
        let op_mock = UserRatingOperationMock {
            consultation_id,
            user_rating: ur,
        };

        let result = get_user_rating_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_user_rating_by_consultation_id_internal_fail_consultation_id_is_negative() {
        let consultation_id = -1;
        let ur = create_dummy_user_rating(consultation_id);
        let op_mock = UserRatingOperationMock {
            consultation_id,
            user_rating: ur,
        };

        let result = get_user_rating_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }
}
