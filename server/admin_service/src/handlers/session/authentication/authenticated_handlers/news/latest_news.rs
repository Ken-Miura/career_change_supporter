// Copyright 2023 Ken Miura

use axum::extract::State;
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::handlers::session::authentication::authenticated_handlers::admin::Admin;

pub(crate) async fn get_latest_news(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
) -> RespResult<LatestNewsResult> {
    todo!()
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct LatestNewsResult {
    news: Vec<News>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct News {
    news_id: i64,
    title: String,
    body: String,
    published_at: String, // RFC 3339形式の文字列
}
