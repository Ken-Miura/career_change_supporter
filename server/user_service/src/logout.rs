// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use axum::extract::{Extension, TypedHeader};
use headers::Cookie;

/// ログアウトを行う
pub(crate) async fn post_logout(
    TypedHeader(_cookies): TypedHeader<Cookie>,
    Extension(_store): Extension<RedisSessionStore>,
) {
}
