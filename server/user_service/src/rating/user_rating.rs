// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::Code;
use crate::util;
use crate::util::disabled_check::DisabledCheckOperationImpl;
use crate::util::session::User;

use super::{ensure_rating_id_is_positive, ensure_rating_is_in_valid_range};

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
}

struct UserRating {
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
        todo!()
    }
}

async fn handle_user_rating(
    account_id: i64,
    user_rating_id: i64,
    rating: i16,
    current_date_time: &DateTime<FixedOffset>,
    op: impl UserRatingOperation,
) -> RespResult<UserRatingResult> {
    ensure_rating_id_is_positive(user_rating_id)?;
    ensure_rating_is_in_valid_range(rating)?;
    ensure_identity_exists(account_id, &op).await?;
    ensure_consultant_is_available(account_id, &op).await?;

    // user_rating_idでuser_ratingを取得
    // user_ratingのコンサルタントとaccount_idが一致していることを確認する
    // user_ratingにある相談時間とcurrent_date_timeを用いて評価を実施可能かチェックする

    // user_ratingを更新する
    //   ユーザーの存在チェック＋ロック -> 仮に存在しない場合はそれ以降の操作は何もしないで成功で終わらせる
    //   user_ratingの取得＋ロック
    //   user_ratingのratingがNULLであることを確認 -> NULLでないなら既に評価済を示すエラーを返す
    //   user_ratingのratingに値を入れる
    todo!()
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
