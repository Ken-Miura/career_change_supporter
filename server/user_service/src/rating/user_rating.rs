// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{DatabaseConnection, EntityTrait, TransactionError, TransactionTrait};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::util;
use crate::util::disabled_check::DisabledCheckOperationImpl;
use crate::util::session::User;

use super::{
    ensure_end_of_consultation_date_time_has_passed, ensure_rating_id_is_positive,
    ensure_rating_is_in_valid_range,
};

pub(crate) async fn post_user_rating(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
    Json(req): Json<UserRatingParam>,
) -> RespResult<UserRatingResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = UserRatingOperationImpl { pool };
    handle_user_rating(
        account_id,
        req.user_rating_id,
        req.rating,
        &current_date_time,
        op,
    )
    .await
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct UserRatingParam {
    user_rating_id: i64,
    rating: i16,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct UserRatingResult {}

#[async_trait]
trait UserRatingOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;

    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp>;

    async fn find_user_rating_by_user_rating_id(
        &self,
        user_rating_id: i64,
    ) -> Result<Option<UserRating>, ErrResp>;

    async fn update_user_rating(
        &self,
        user_account_id: i64,
        user_rating_id: i64,
        rating: i16,
    ) -> Result<(), ErrResp>;
}

struct UserRating {
    user_account_id: i64,
    consultant_id: i64,
    consultation_date_time_in_jst: DateTime<FixedOffset>,
    rating: Option<i16>,
}

struct UserRatingOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl UserRatingOperation for UserRatingOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::identity_checker::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp> {
        let op = DisabledCheckOperationImpl::new(&self.pool);
        util::disabled_check::check_if_user_account_is_available(consultant_id, op).await
    }

    async fn find_user_rating_by_user_rating_id(
        &self,
        user_rating_id: i64,
    ) -> Result<Option<UserRating>, ErrResp> {
        let model = entity::user_rating::Entity::find_by_id(user_rating_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find user_rating (user_rating_id: {}): {}",
                    user_rating_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| UserRating {
            user_account_id: m.user_account_id,
            consultant_id: m.consultant_id,
            consultation_date_time_in_jst: m.meeting_at.with_timezone(&(*JAPANESE_TIME_ZONE)),
            rating: m.rating,
        }))
    }

    async fn update_user_rating(
        &self,
        user_account_id: i64,
        user_rating_id: i64,
        rating: i16,
    ) -> Result<(), ErrResp> {
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    // user_ratingを更新する
                    //   ユーザー(ur.user_account_id)の存在チェック＋ロック -> 仮に存在しない場合はそれ以降の操作は何もしないで成功で終わらせる
                    //   user_ratingの取得＋ロック
                    //   user_ratingのratingがNULLであることを確認 -> NULLでないなら既に評価済を示すエラーを返す
                    //   user_ratingのratingに値を入れる
                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(db_err) => {
                    error!("connection error: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    error!("failed to update_user_rating: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

async fn handle_user_rating(
    consultant_id: i64,
    user_rating_id: i64,
    rating: i16,
    current_date_time: &DateTime<FixedOffset>,
    op: impl UserRatingOperation,
) -> RespResult<UserRatingResult> {
    ensure_rating_id_is_positive(user_rating_id)?;
    ensure_rating_is_in_valid_range(rating)?;
    ensure_identity_exists(consultant_id, &op).await?;
    ensure_consultant_is_available(consultant_id, &op).await?;

    let ur = get_user_rating_by_user_rating_id(user_rating_id, &op).await?;
    ensure_consultant_ids_are_same(consultant_id, ur.consultant_id)?;
    ensure_end_of_consultation_date_time_has_passed(
        &ur.consultation_date_time_in_jst,
        current_date_time,
    )?;

    op.update_user_rating(ur.user_account_id, user_rating_id, rating)
        .await?;

    Ok((StatusCode::OK, Json(UserRatingResult {})))
}

async fn ensure_identity_exists(
    account_id: i64,
    op: &impl UserRatingOperation,
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

async fn ensure_consultant_is_available(
    consultant_id: i64,
    op: &impl UserRatingOperation,
) -> Result<(), ErrResp> {
    let available = op.check_if_consultant_is_available(consultant_id).await?;
    if !available {
        error!(
            "consultant is not available (consultant_id: {})",
            consultant_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultantIsNotAvailable as u32,
            }),
        ));
    }
    Ok(())
}

async fn get_user_rating_by_user_rating_id(
    user_rating_id: i64,
    op: &impl UserRatingOperation,
) -> Result<UserRating, ErrResp> {
    let ur = op
        .find_user_rating_by_user_rating_id(user_rating_id)
        .await?;
    match ur {
        Some(u) => Ok(u),
        None => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoUserRatingFound as u32,
            }),
        )),
    }
}

fn ensure_consultant_ids_are_same(
    consultant_id: i64,
    consultant_id_in_user_rating: i64,
) -> Result<(), ErrResp> {
    if consultant_id != consultant_id_in_user_rating {
        error!(
            "consultant_id ({}) and consultant_id_in_user_rating ({}) are not same",
            consultant_id, consultant_id_in_user_rating
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoUserRatingFound as u32,
            }),
        ));
    }
    Ok(())
}
