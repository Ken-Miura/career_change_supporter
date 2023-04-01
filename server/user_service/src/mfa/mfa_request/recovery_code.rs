// Copyright 2023 Ken Miura

use async_redis_session::RedisSessionStore;
use axum::{extract::State, Json};
use axum_extra::extract::SignedCookieJar;
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

pub(crate) async fn post_recovery_code(
    jar: SignedCookieJar,
    State(pool): State<DatabaseConnection>,
    State(store): State<RedisSessionStore>,
    Json(req): Json<RecoveryCodeReq>,
) -> RespResult<RecoveryCodeReqResult> {
    todo!()
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RecoveryCodeReq {
    recovery_code: String,
}

#[derive(Serialize)]
pub(crate) struct RecoveryCodeReqResult {}
