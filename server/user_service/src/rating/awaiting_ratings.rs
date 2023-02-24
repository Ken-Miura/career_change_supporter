// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::extract::State;
use chrono::{DateTime, FixedOffset, Utc};
use common::{RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::util::{request_consultation::ConsultationDateTime, session::User};

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
    consultation_id: i64,
    consultant_id: i64, // 相談相手のユーザーID
    meeting_date_time_in_jst: ConsultationDateTime,
}

/// 相談相手として行う評価
#[derive(Clone, Debug, Serialize, PartialEq)]
struct ConsultantSideAwaitingRating {
    consultation_id: i64,
    user_account_id: i64, // 相談申し込み者のユーザーID
    meeting_date_time_in_jst: ConsultationDateTime,
}

#[async_trait]
trait AwaitingRatingsOperation {}

struct AwaitingRatingsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl AwaitingRatingsOperation for AwaitingRatingsOperationImpl {}

async fn handle_awaiting_ratings(
    account_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    op: impl AwaitingRatingsOperation,
) -> RespResult<AwaitingRatingsResult> {
    todo!()
}
