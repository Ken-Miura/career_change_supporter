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

    struct LatestNewsOperationMock {
        news_array: Vec<(i64, String, String, DateTime<FixedOffset>)>,
    }

    #[async_trait]
    impl LatestNewsOperation for LatestNewsOperationMock {
        async fn filter_news_by_criteria(
            &self,
            criteria: DateTime<FixedOffset>,
        ) -> Result<Vec<News>, ErrResp> {
            Ok(self
                .news_array
                .clone()
                .into_iter()
                .filter(|n| n.3 > criteria)
                .map(|m| News {
                    news_id: m.0,
                    title: m.1,
                    body: m.2,
                    published_at: m.3.to_rfc3339(),
                })
                .collect::<Vec<News>>())
        }
    }

    #[tokio::test]
    async fn handle_news_empty_result() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let op = LatestNewsOperationMock { news_array: vec![] };

        let result = handle_latest_news(&current_date_time, op)
            .await
            .expect("failed to get Ok");

        assert_eq!(result.0, StatusCode::OK);
        assert_eq!(result.1 .0, LatestNewsResult { news_array: vec![] });
    }

    #[tokio::test]
    async fn handle_news_1_result() {
        let news_id = 1;
        let title = "title".to_string();
        let body = r"line1
      line2
      line3"
            .to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let op = LatestNewsOperationMock {
            news_array: vec![(news_id, title.clone(), body.clone(), current_date_time)],
        };

        let result = handle_latest_news(&current_date_time, op)
            .await
            .expect("failed to get Ok");

        assert_eq!(result.0, StatusCode::OK);
        assert_eq!(
            result.1 .0,
            LatestNewsResult {
                news_array: vec![News {
                    news_id,
                    title,
                    body,
                    published_at: current_date_time.to_rfc3339()
                }]
            }
        );
    }

    #[tokio::test]
    async fn handle_news_2_results() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let news_id1 = 1;
        let title1 = "title1".to_string();
        let body1 = r"line1
      line2
      line3"
            .to_string();
        let pd1 = current_date_time - chrono::Duration::days(1);
        let news_id2 = 2;
        let title2 = "title2".to_string();
        let body2 = r"line
      line
      line"
            .to_string();
        let pd2 = current_date_time - chrono::Duration::days(2);
        let op = LatestNewsOperationMock {
            news_array: vec![
                (news_id1, title1.clone(), body1.clone(), pd1),
                (news_id2, title2.clone(), body2.clone(), pd2),
            ],
        };

        let result = handle_latest_news(&current_date_time, op)
            .await
            .expect("failed to get Ok");

        assert_eq!(result.0, StatusCode::OK);
        assert_eq!(
            result.1 .0,
            LatestNewsResult {
                news_array: vec![
                    News {
                        news_id: news_id1,
                        title: title1,
                        body: body1,
                        published_at: pd1.to_rfc3339()
                    },
                    News {
                        news_id: news_id2,
                        title: title2,
                        body: body2,
                        published_at: pd2.to_rfc3339()
                    }
                ]
            }
        );
    }

    #[tokio::test]
    async fn handle_news_does_not_return_old_news() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 11, 21, 32, 21)
            .unwrap();
        let news_id = 1;
        let title = "title".to_string();
        let body = r"line1
      line2
      line3"
            .to_string();
        let pd = current_date_time - chrono::Duration::days(NEWS_RETRIEVAL_CRITERIA_IN_DAYS);
        let op = LatestNewsOperationMock {
            news_array: vec![(news_id, title.clone(), body.clone(), pd)],
        };

        let result = handle_latest_news(&current_date_time, op)
            .await
            .expect("failed to get Ok");

        assert_eq!(result.0, StatusCode::OK);
        assert_eq!(result.1 .0, LatestNewsResult { news_array: vec![] });
    }
}
