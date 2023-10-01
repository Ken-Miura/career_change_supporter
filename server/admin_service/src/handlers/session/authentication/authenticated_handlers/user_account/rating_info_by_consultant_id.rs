// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::{ErrResp, RespResult};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::error;

use crate::err::unexpected_err_resp;

use super::super::admin::Admin;
use super::{
    calculate_rating_and_count, validate_account_id_is_positive, ConsultantIdQuery,
    RatingInfoResult,
};

pub(crate) async fn get_rating_info_by_consultant_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<ConsultantIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<RatingInfoResult> {
    let query = query.0;
    let op = RatingInfoOperationImpl { pool };
    get_rating_info_by_consultant_id_internal(query.consultant_id, op).await
}

async fn get_rating_info_by_consultant_id_internal(
    consultant_id: i64,
    op: impl RatingInfoOperation,
) -> RespResult<RatingInfoResult> {
    validate_account_id_is_positive(consultant_id)?;
    let ratings = op.get_rating_info_by_consultant_id(consultant_id).await?;
    let rating_and_count = calculate_rating_and_count(ratings);
    Ok((
        StatusCode::OK,
        Json(RatingInfoResult {
            average_rating: rating_and_count.0,
            count: rating_and_count.1,
        }),
    ))
}

#[async_trait]
trait RatingInfoOperation {
    async fn get_rating_info_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Vec<i16>, ErrResp>;
}

struct RatingInfoOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl RatingInfoOperation for RatingInfoOperationImpl {
    async fn get_rating_info_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Vec<i16>, ErrResp> {
        let models = entity::consultation::Entity::find()
            .filter(entity::consultation::Column::ConsultantId.eq(consultant_id))
            .find_with_related(entity::consultant_rating::Entity)
            .filter(entity::consultant_rating::Column::Rating.is_not_null())
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter consultant_rating (consultant_id: {}): {}",
                    consultant_id, e
                );
                unexpected_err_resp()
            })?;
        models
            .into_iter()
            .map(|m| {
                // TODO: 設計が変わったので修正、対策
                // consultationとconsultant_ratingは1対1の設計なので取れない場合は想定外エラーとして扱う
                let ur = m.1.get(0).ok_or_else(|| {
                    error!(
                        "failed to find consultant_rating (consultation_id: {})",
                        m.0.consultation_id
                    );
                    unexpected_err_resp()
                })?;
                let r = ur.rating.ok_or_else(|| {
                    error!(
                        "rating is null (consultation_id: {}, consultant_id: {})",
                        ur.consultation_id, m.0.consultant_id
                    );
                    unexpected_err_resp()
                })?;
                Ok(r)
            })
            .collect::<Result<Vec<i16>, ErrResp>>()
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::ErrResp;

    use crate::err::Code;

    use super::*;

    struct RatingInfoOperationMock {
        consultant_id: i64,
        ratings: Vec<i16>,
    }

    #[async_trait]
    impl RatingInfoOperation for RatingInfoOperationMock {
        async fn get_rating_info_by_consultant_id(
            &self,
            consultant_id: i64,
        ) -> Result<Vec<i16>, ErrResp> {
            if self.consultant_id != consultant_id {
                panic!("never reach here");
            }
            Ok(self.ratings.clone())
        }
    }

    #[tokio::test]

    async fn get_rating_info_by_consultant_id_internal_success_no_result() {
        let consultant_id = 64431;
        let ratings = vec![];
        let op_mock = RatingInfoOperationMock {
            consultant_id,
            ratings,
        };

        let result = get_rating_info_by_consultant_id_internal(consultant_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(None, resp.1 .0.average_rating);
        assert_eq!(0, resp.1 .0.count);
    }

    #[tokio::test]

    async fn get_rating_info_by_consultant_id_internal_success_1() {
        let consultant_id = 64431;
        let ratings = vec![3];
        let op_mock = RatingInfoOperationMock {
            consultant_id,
            ratings,
        };

        let result = get_rating_info_by_consultant_id_internal(consultant_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Some("3.0".to_string()), resp.1 .0.average_rating);
        assert_eq!(1, resp.1 .0.count);
    }

    #[tokio::test]

    async fn get_rating_info_by_consultant_id_internal_success_2() {
        let consultant_id = 64431;
        let ratings = vec![3, 4];
        let op_mock = RatingInfoOperationMock {
            consultant_id,
            ratings,
        };

        let result = get_rating_info_by_consultant_id_internal(consultant_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Some("3.5".to_string()), resp.1 .0.average_rating);
        assert_eq!(2, resp.1 .0.count);
    }

    #[tokio::test]

    async fn get_rating_info_by_consultant_id_internal_success_3() {
        let consultant_id = 64431;
        let ratings = vec![3, 4, 1];
        let op_mock = RatingInfoOperationMock {
            consultant_id,
            ratings,
        };

        let result = get_rating_info_by_consultant_id_internal(consultant_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Some("2.7".to_string()), resp.1 .0.average_rating);
        assert_eq!(3, resp.1 .0.count);
    }

    #[tokio::test]
    async fn get_rating_info_by_consultant_id_internal_fail_consultant_id_is_zero() {
        let consultant_id = 0;
        let ratings = vec![3, 4, 1];
        let op_mock = RatingInfoOperationMock {
            consultant_id,
            ratings,
        };

        let result = get_rating_info_by_consultant_id_internal(consultant_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_rating_info_by_consultant_id_internal_fail_consultant_id_is_negative() {
        let consultant_id = -1;
        let ratings = vec![3, 4, 1];
        let op_mock = RatingInfoOperationMock {
            consultant_id,
            ratings,
        };

        let result = get_rating_info_by_consultant_id_internal(consultant_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }
}
