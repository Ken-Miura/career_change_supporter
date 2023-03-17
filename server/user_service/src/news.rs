// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::{async_trait, Json};
use chrono::{DateTime, Datelike, FixedOffset, Utc};
use common::util::Ymd;
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use hyper::StatusCode;
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;

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
    news_id: i64,
    title: String,
    body: String,
    published_date_in_jst: Ymd,
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
        let results = entity::news::Entity::find()
            .filter(entity::news::Column::PublishedAt.gt(criteria))
            .order_by_desc(entity::news::Column::PublishedAt)
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!("failed to filter news (criteria: {}): {}", criteria, e);
                unexpected_err_resp()
            })?;
        Ok(results
            .into_iter()
            .map(|m| {
                let pd = m.published_at.with_timezone(&(*JAPANESE_TIME_ZONE));
                News {
                    news_id: m.news_id,
                    title: m.title,
                    body: m.body,
                    published_date_in_jst: Ymd {
                        year: pd.year(),
                        month: pd.month(),
                        day: pd.day(),
                    },
                }
            })
            .collect())
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
