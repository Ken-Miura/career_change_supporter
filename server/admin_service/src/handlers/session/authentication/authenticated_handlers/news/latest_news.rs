// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::Json;
use axum::{async_trait, http::StatusCode};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE, NEWS_RETRIEVAL_CRITERIA_IN_DAYS};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;
use tracing::error;

use crate::{
    err::unexpected_err_resp,
    handlers::session::authentication::authenticated_handlers::admin::Admin,
};

pub(crate) async fn get_latest_news(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
) -> RespResult<LatestNewsResult> {
    let op = LatestNewsOperationImpl { pool };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    handle_latest_news(&current_date_time, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct LatestNewsResult {
    news_array: Vec<News>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct News {
    news_id: i64,
    title: String,
    body: String,
    published_at: String, // RFC 3339形式の文字列
}

#[async_trait]
trait LatestNewsOperation {
    /// criteriaより新しい日付に掲載されているnewsを取得する
    async fn filter_news_by_criteria(
        &self,
        criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<News>, ErrResp>;
}

struct LatestNewsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl LatestNewsOperation for LatestNewsOperationImpl {
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
                    published_at: pd.to_rfc3339(),
                }
            })
            .collect())
    }
}

async fn handle_latest_news(
    current_date_time: &DateTime<FixedOffset>,
    op: impl LatestNewsOperation,
) -> RespResult<LatestNewsResult> {
    let criteria = *current_date_time - chrono::Duration::days(NEWS_RETRIEVAL_CRITERIA_IN_DAYS);
    let news_array = op.filter_news_by_criteria(criteria).await?;
    Ok((StatusCode::OK, Json(LatestNewsResult { news_array })))
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;

    use super::*;
}
