// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::util::validator::{has_control_char, has_non_new_line_control_char};
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::ActiveValue::NotSet;
use entity::sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Set};
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
    todo!()
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
