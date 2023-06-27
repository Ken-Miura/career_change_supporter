// Copyright 2023 Ken Miura

use axum::{extract::State, Json};
use chrono::Utc;
use common::{RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::handlers::session::authentication::authenticated_handlers::admin::Admin;

pub(crate) async fn post_set_news_req(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<SetNewsReq>,
) -> RespResult<SetNewsReqResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    todo!()
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct SetNewsReq {
    title: String,
    body: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct SetNewsReqResult {}
