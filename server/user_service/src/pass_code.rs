// Copyright 2023 Ken Miura

use async_redis_session::RedisSessionStore;
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::SignedCookieJar;
use common::ErrResp;
use entity::sea_orm::DatabaseConnection;
use serde::Deserialize;

pub(crate) async fn post_pass_code(
    jar: SignedCookieJar,
    State(pool): State<DatabaseConnection>,
    State(store): State<RedisSessionStore>,
    Json(req): Json<PassCodeReq>,
) -> Result<(StatusCode, SignedCookieJar), ErrResp> {
    todo!()
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PassCodeReq {
    pass_code: String,
}
