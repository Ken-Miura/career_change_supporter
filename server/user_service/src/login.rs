// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use axum::extract::Extension;
use common::{DatabaseConnection, ValidCred};

pub(crate) async fn post_login(
    ValidCred(_cred): ValidCred,
    DatabaseConnection(_conn): DatabaseConnection,
    Extension(_store): Extension<RedisSessionStore>,
) {
}
