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
    calculate_rating_and_count, validate_account_id_is_positive, RatingInfoResult,
    UserAccountIdQuery,
};

pub(crate) async fn get_rating_info_by_user_account_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<UserAccountIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<RatingInfoResult> {
    let query = query.0;
    let op = RatingInfoOperationImpl { pool };
    get_rating_info_by_user_account_id_internal(query.user_account_id, op).await
}

async fn get_rating_info_by_user_account_id_internal(
    user_account_id: i64,
    op: impl RatingInfoOperation,
) -> RespResult<RatingInfoResult> {
    validate_account_id_is_positive(user_account_id)?;
    let ratings = op
        .get_rating_info_by_user_account_id(user_account_id)
        .await?;
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
    async fn get_rating_info_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<i16>, ErrResp>;
}

struct RatingInfoOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl RatingInfoOperation for RatingInfoOperationImpl {
    async fn get_rating_info_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<i16>, ErrResp> {
        let models = entity::consultation::Entity::find()
            .filter(entity::consultation::Column::UserAccountId.eq(user_account_id))
            .find_with_related(entity::user_rating::Entity)
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
                // consultationとuser_ratingは1対1の設計なので取れない場合は想定外エラーとして扱う
                let ur = m.1.get(0).ok_or_else(|| {
                    error!(
                        "failed to find user_rating (consultation_id: {})",
                        m.0.consultation_id
                    );
                    unexpected_err_resp()
                })?;
                let r = ur.rating.ok_or_else(|| {
                    error!(
                        "rating is null (user_rating_id: {}, user_account_id: {})",
                        ur.user_rating_id, m.0.user_account_id
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
    // use axum::async_trait;
    // use axum::http::StatusCode;
    // use common::ErrResp;

    // use crate::err::Code;

    // use super::*;

    // struct RatingInfoOperationMock {
    //     user_account_id: i64,
    //     consultations: Vec<Consultation>,
    // }

    // #[async_trait]
    // impl RatingInfoOperation for RatingInfoOperationMock {
    //     async fn get_rating_info_by_user_account_id(
    //         &self,
    //         user_account_id: i64,
    //     ) -> Result<Vec<i16>, ErrResp> {
    //         if self.user_account_id != user_account_id {
    //             return Ok(vec![]);
    //         }
    //         Ok(self.consultations.clone())
    //     }
    // }

    // fn create_dummy_consultation1(user_account_id: i64) -> Consultation {
    //     Consultation {
    //         consultation_id: 1,
    //         user_account_id,
    //         consultant_id: 510,
    //         meeting_at: "2023-04-13T14:00:00.0000+09:00 ".to_string(),
    //         room_name: "1241cd91a9c3433f98d16f40f51a5090".to_string(),
    //         user_account_entered_at: None,
    //         consultant_entered_at: None,
    //     }
    // }

    // fn create_dummy_consultation2(user_account_id: i64) -> Consultation {
    //     Consultation {
    //         consultation_id: 2,
    //         user_account_id,
    //         consultant_id: 6110,
    //         meeting_at: "2023-04-15T14:00:00.0000+09:00 ".to_string(),
    //         room_name: "3241cd91a9c3433f98d16f40f51a5090".to_string(),
    //         user_account_entered_at: Some("2023-04-15T13:58:32.2424+09:00 ".to_string()),
    //         consultant_entered_at: Some("2023-04-15T13:57:42.3435+09:00 ".to_string()),
    //     }
    // }

    // #[tokio::test]

    // async fn get_rating_info_by_user_account_id_internal_success_1_result() {
    //     let user_account_id = 64431;
    //     let consultation1 = create_dummy_consultation1(user_account_id);
    //     let op_mock = RatingInfoOperationMock {
    //         user_account_id,
    //         consultations: vec![consultation1.clone()],
    //     };

    //     let result = get_rating_info_by_user_account_id_internal(user_account_id, op_mock).await;

    //     let resp = result.expect("failed to get Ok");
    //     assert_eq!(StatusCode::OK, resp.0);
    //     assert_eq!(vec![consultation1], resp.1 .0.consultations);
    // }

    // #[tokio::test]

    // async fn get_rating_info_by_user_account_id_internal_success_2_results() {
    //     let user_account_id = 64431;
    //     let consultation1 = create_dummy_consultation1(user_account_id);
    //     let consultation2 = create_dummy_consultation2(user_account_id);
    //     let op_mock = RatingInfoOperationMock {
    //         user_account_id,
    //         consultations: vec![consultation1.clone(), consultation2.clone()],
    //     };

    //     let result = get_rating_info_by_user_account_id_internal(user_account_id, op_mock).await;

    //     let resp = result.expect("failed to get Ok");
    //     assert_eq!(StatusCode::OK, resp.0);
    //     assert_eq!(vec![consultation1, consultation2], resp.1 .0.consultations);
    // }

    // #[tokio::test]

    // async fn get_rating_info_by_user_account_id_internal_success_no_result() {
    //     let user_account_id = 64431;
    //     let op_mock = RatingInfoOperationMock {
    //         user_account_id,
    //         consultations: vec![],
    //     };

    //     let result = get_rating_info_by_user_account_id_internal(user_account_id, op_mock).await;

    //     let resp = result.expect("failed to get Ok");
    //     assert_eq!(StatusCode::OK, resp.0);
    //     assert_eq!(
    //         Vec::<Consultation>::with_capacity(0),
    //         resp.1 .0.consultations
    //     );
    // }

    // #[tokio::test]
    // async fn get_rating_info_by_user_account_id_internal_fail_user_account_id_is_zero() {
    //     let user_account_id = 0;
    //     let op_mock = RatingInfoOperationMock {
    //         user_account_id,
    //         consultations: vec![],
    //     };

    //     let result = get_rating_info_by_user_account_id_internal(user_account_id, op_mock).await;

    //     let resp = result.expect_err("failed to get Err");
    //     assert_eq!(resp.0, StatusCode::BAD_REQUEST);
    //     assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    // }

    // #[tokio::test]
    // async fn get_rating_info_by_user_account_id_internal_fail_user_account_id_is_negative() {
    //     let user_account_id = -1;
    //     let op_mock = RatingInfoOperationMock {
    //         user_account_id,
    //         consultations: vec![],
    //     };

    //     let result = get_rating_info_by_user_account_id_internal(user_account_id, op_mock).await;

    //     let resp = result.expect_err("failed to get Err");
    //     assert_eq!(resp.0, StatusCode::BAD_REQUEST);
    //     assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    // }
}
