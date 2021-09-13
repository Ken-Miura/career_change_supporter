// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::extract::{Extension, TypedHeader};
use axum::http::StatusCode;
use common::ErrResp;
use headers::{Cookie, HeaderMap, HeaderMapExt};

use crate::util::{unexpected_err_resp, COOKIE_NAME};

use axum::{body::Body, http::Request};

/// ログアウトを行う
/// <br>
/// # Errors
///
pub(crate) async fn post_logout(req: Request<Body>) -> LogoutResult {
    let extentions = req.extensions();
    let store = extentions.get::<RedisSessionStore>().ok_or_else(|| {
        tracing::error!("failed to get session store");
        unexpected_err_resp()
    })?;
    let headers = req.headers();
    let option_cookie = headers.typed_try_get::<Cookie>().map_err(|e| {
        tracing::error!("failed to get cookie: {}", e);
        unexpected_err_resp()
    })?;
    post_logout_internal(option_cookie, store).await
}

/// ログアウトリクエストの結果を示す型
pub(crate) type LogoutResult = Result<LogoutResp, ErrResp>;

/// ログアウトに成功した場合に返却される型
pub(crate) type LogoutResp = (StatusCode, HeaderMap);

pub(crate) async fn post_logout_internal(
    option_cookie: Option<Cookie>,
    store: &impl SessionStore,
) -> LogoutResult {
    //let cookie_name_value = cookie.get(COOKIE_NAME);
    //todo!()
    let headerMap = HeaderMap::new();
    Ok((StatusCode::OK, headerMap))
}
