// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use common::{ApiError, ErrResp, RespResult};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::authentication::authenticated_handlers::admin::Admin;

pub(crate) async fn post_delete_news_req(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<SetDeleteReq>,
) -> RespResult<SetDeleteReqResult> {
    let op = SetDeleteReqOperationImpl { pool };
    handle_delete_news_req(req.news_id, &op).await
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct SetDeleteReq {
    news_id: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct SetDeleteReqResult {}

#[async_trait]
trait SetDeleteReqOperation {
    async fn delete_news(&self, news_id: i64) -> Result<(), ErrResp>;
}

struct SetDeleteReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl SetDeleteReqOperation for SetDeleteReqOperationImpl {
    async fn delete_news(&self, news_id: i64) -> Result<(), ErrResp> {
        let _ = entity::news::Entity::delete_by_id(news_id)
            .exec(&self.pool)
            .await
            .map_err(|e| {
                error!("failed to delete news (news_id: {}): {}", news_id, e);
                unexpected_err_resp()
            })?;
        Ok(())
    }
}

async fn handle_delete_news_req(
    news_id: i64,
    op: &impl SetDeleteReqOperation,
) -> RespResult<SetDeleteReqResult> {
    if !news_id.is_positive() {
        error!("news_id ({}) is not positive", news_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidNewsId as u32,
            }),
        ));
    }

    op.delete_news(news_id).await?;

    Ok((StatusCode::OK, Json(SetDeleteReqResult {})))
}

#[cfg(test)]
mod tests {

    use super::*;

    struct SetDeleteReqOperationMock {
        news_id: i64,
    }

    #[async_trait]
    impl SetDeleteReqOperation for SetDeleteReqOperationMock {
        async fn delete_news(&self, news_id: i64) -> Result<(), ErrResp> {
            assert_eq!(self.news_id, news_id);
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_delete_news_req_success() {
        let news_id = 1;
        let op = SetDeleteReqOperationMock { news_id };

        let result = handle_delete_news_req(news_id, &op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(resp.0, StatusCode::OK);
        assert_eq!(resp.1 .0, SetDeleteReqResult {});
    }

    #[tokio::test]
    async fn handle_delete_news_req_fail_news_id_zero() {
        let news_id = 0;
        let op = SetDeleteReqOperationMock { news_id };

        let result = handle_delete_news_req(news_id, &op).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::InvalidNewsId as u32);
    }

    #[tokio::test]
    async fn handle_delete_news_req_fail_news_id_negative() {
        let news_id = -1;
        let op = SetDeleteReqOperationMock { news_id };

        let result = handle_delete_news_req(news_id, &op).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::InvalidNewsId as u32);
    }
}
