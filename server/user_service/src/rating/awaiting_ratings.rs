// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::extract::State;
use chrono::{DateTime, Datelike, FixedOffset, Timelike, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::{
    sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder, QuerySelect},
    user_rating,
};
use serde::Serialize;
use tracing::error;

use crate::{
    err::unexpected_err_resp,
    util::{request_consultation::ConsultationDateTime, session::User},
};

const MAX_NUM_OF_USER_SIDE_AWAITING_RATINGS: u64 = 20;

pub(crate) async fn get_awaiting_ratings(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<AwaitingRatingsResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = AwaitingRatingsOperationImpl { pool };
    handle_awaiting_ratings(account_id, &current_date_time, op).await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct AwaitingRatingsResult {
    user_side_awaiting_ratings: Vec<UserSideAwaitingRating>,
    consultant_side_awaiting_ratings: Vec<ConsultantSideAwaitingRating>,
}

/// 相談申し込み者として行う評価
#[derive(Clone, Debug, Serialize, PartialEq)]
struct UserSideAwaitingRating {
    user_rating_id: i64,
    consultant_id: i64, // 相談相手のユーザーID
    meeting_date_time_in_jst: ConsultationDateTime,
}

/// 相談相手として行う評価
#[derive(Clone, Debug, Serialize, PartialEq)]
struct ConsultantSideAwaitingRating {
    consultant_rating_id: i64,
    user_account_id: i64, // 相談申し込み者のユーザーID
    meeting_date_time_in_jst: ConsultationDateTime,
}

// 身分のチェックが出来ていなければ、そもそも相談の申込みができない
// 相談の申込みが出来ていなければ、評価待ちは何もない
// 従って身分のチェックができていないユーザーは空の結果が返るだけなので
// わざわざ身分チェックをする処理を入れない
#[async_trait]
trait AwaitingRatingsOperation {
    async fn filter_user_side_awaiting_ratings(
        &self,
        user_account_id: i64,
        start_criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<UserSideAwaitingRating>, ErrResp>;

    async fn filter_consultant_side_awaiting_ratings(
        &self,
        consultant_id: i64,
        start_criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<ConsultantSideAwaitingRating>, ErrResp>;
}

struct AwaitingRatingsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl AwaitingRatingsOperation for AwaitingRatingsOperationImpl {
    async fn filter_user_side_awaiting_ratings(
        &self,
        user_account_id: i64,
        start_criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<UserSideAwaitingRating>, ErrResp> {
        let results = user_rating::Entity::find()
            .filter(user_rating::Column::MeetingAt.lt(start_criteria))
            .filter(user_rating::Column::UserAccountId.eq(user_account_id))
            .filter(user_rating::Column::Rating.is_null()) // null -> まだ未評価であるもの
            .limit(MAX_NUM_OF_USER_SIDE_AWAITING_RATINGS)
            .order_by_asc(user_rating::Column::MeetingAt)
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter user_rating (user_account_id: {}, start_criteria: {}): {}",
                    user_account_id, start_criteria, e
                );
                unexpected_err_resp()
            })?;
        Ok(results
            .into_iter()
            .map(|m| {
                let meeting_at_in_jst = m.meeting_at.with_timezone(&*JAPANESE_TIME_ZONE);
                UserSideAwaitingRating {
                    user_rating_id: m.user_rating_id,
                    consultant_id: m.consultant_id,
                    meeting_date_time_in_jst: ConsultationDateTime {
                        year: meeting_at_in_jst.year(),
                        month: meeting_at_in_jst.month(),
                        day: meeting_at_in_jst.day(),
                        hour: meeting_at_in_jst.hour(),
                    },
                }
            })
            .collect::<Vec<UserSideAwaitingRating>>())
    }

    async fn filter_consultant_side_awaiting_ratings(
        &self,
        consultant_id: i64,
        start_criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<ConsultantSideAwaitingRating>, ErrResp> {
        todo!()
    }
}

async fn handle_awaiting_ratings(
    account_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    op: impl AwaitingRatingsOperation,
) -> RespResult<AwaitingRatingsResult> {
    todo!()
}
