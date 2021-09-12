// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::extract::{Extension, TypedHeader};
use common::ErrResp;
use headers::{Cookie, HeaderMap};
use hyper::StatusCode;

use crate::util::COOKIE_NAME;

/// ログアウトを行う
/// <br>
/// # Errors
/// リクエストにcookieを含んでいない場合、ステータスコード400を返す<br>
pub(crate) async fn post_logout(
    TypedHeader(cookie): TypedHeader<Cookie>,
    Extension(store): Extension<RedisSessionStore>,
) -> LogoutResult {
    post_logout_internal(&cookie, store).await;
    todo!()
}

/// ログアウトリクエストの結果を示す型
pub(crate) type LogoutResult = Result<LogoutResp, ErrResp>;

/// ログアウトに成功した場合に返却される型
pub(crate) type LogoutResp = (StatusCode, HeaderMap);

pub(crate) async fn post_logout_internal(
    cookie: &Cookie,
    store: impl SessionStore,
) -> LogoutResult {
    let cookie_name_value = cookie.get(COOKIE_NAME);
    todo!()
}
