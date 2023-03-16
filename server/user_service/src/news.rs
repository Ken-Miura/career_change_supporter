// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use hyper::StatusCode;
use serde::Serialize;

const NEWS_RETRIEVAL_CRITERIA_IN_DAYS: i64 = 180;

pub(crate) async fn get_news(State(pool): State<DatabaseConnection>) -> RespResult<NewsResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = NewsOperationImpl { pool };
    handle_news(&current_date_time, op).await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct NewsResult {
    news_array: Vec<News>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct News {
    title: String,
    body: String,
    published_date_in_jst: PublishedDate,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct PublishedDate {
    year: i32,
    month: u32,
    day: u32,
}

#[async_trait]
trait NewsOperation {
    /// criteriaより新しい日付に掲載されているnewsを取得する
    async fn filter_news_by_criteria(
        &self,
        criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<News>, ErrResp>;
}

struct NewsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl NewsOperation for NewsOperationImpl {
    async fn filter_news_by_criteria(
        &self,
        criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<News>, ErrResp> {
        todo!();
    }
}

async fn handle_news(
    current_date_time: &DateTime<FixedOffset>,
    op: impl NewsOperation,
) -> RespResult<NewsResult> {
    let criteria = *current_date_time - chrono::Duration::days(NEWS_RETRIEVAL_CRITERIA_IN_DAYS);
    let news_array = op.filter_news_by_criteria(criteria).await?;
    Ok((StatusCode::OK, Json(NewsResult { news_array })))
}
