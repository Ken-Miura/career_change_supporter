// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use axum::extract::{Extension, TypedHeader};
use common::ErrResp;
use headers::{Cookie, HeaderMap};
use hyper::StatusCode;

/// ログアウトを行う
pub(crate) async fn post_logout(
    TypedHeader(_cookies): TypedHeader<Cookie>,
    Extension(_store): Extension<RedisSessionStore>,
) -> LogoutResult {
    todo!()
}

/// ログアウトリクエストの結果を示す型
pub(crate) type LogoutResult = Result<LogoutResp, ErrResp>;

/// ログアウトに成功した場合に返却される型
pub(crate) type LogoutResp = (StatusCode, HeaderMap);
