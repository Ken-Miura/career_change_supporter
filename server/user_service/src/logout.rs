// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::extract::{Extension, TypedHeader};
use axum::http::StatusCode;
use common::ErrResp;
use headers::{Cookie, HeaderMap, HeaderMapExt};

use crate::util::COOKIE_NAME;

use axum::{body::Body, http::Request};

/// ログアウトを行う
/// <br>
/// # Errors
/// リクエストにcookieを含んでいない場合、ステータスコード400を返す<br>
pub(crate) async fn post_logout(
    //TypedHeader(cookie): TypedHeader<Cookie>,
    //Extension(store): Extension<RedisSessionStore>,
    req: Request<Body>, //) -> LogoutResult {
) {
    let extentions = req.extensions();
    let store = extentions
        .get::<RedisSessionStore>()
        .expect("failed to get value");
    let headers = req.headers();
    let result = headers.typed_try_get::<Cookie>();
    match result {
        Ok(option) => match option {
            Some(cookie) => println!("cookie: {:?}", cookie),
            None => println!("None"),
        },
        Err(e) => println!("err: {}", e),
    }
    //post_logout_internal(&cookie, store).await;
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
