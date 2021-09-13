// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::http::StatusCode;
use common::ErrResp;
use headers::{Cookie, HeaderMap, HeaderMapExt, HeaderValue};
use hyper::header::SET_COOKIE;

use crate::util::{unexpected_err_resp, COOKIE_NAME, ROOT_PATH};

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
    let cookie = match option_cookie {
        Some(c) => c,
        None => {
            tracing::debug!("no cookie on logout");
            return Ok((StatusCode::OK, HeaderMap::new()));
        }
    };
    let cookie_name_value = match cookie.get(COOKIE_NAME) {
        Some(value) => value,
        None => {
            tracing::debug!("no {} in cookie on logout", COOKIE_NAME);
            return Ok((StatusCode::OK, HeaderMap::new()));
        }
    };
    let option_session = store
        .load_session(cookie_name_value.to_string())
        .await
        .map_err(|e| {
            tracing::error!("failed to load session: {}", e);
            unexpected_err_resp()
        })?;
    let session = match option_session {
        Some(s) => s,
        None => {
            tracing::debug!("no session in session store on logout");
            return Ok((StatusCode::OK, HeaderMap::new()));
        }
    };
    let _ = store.destroy_session(session).await.map_err(|e| {
        tracing::error!("failed to destroy session (={}): {}", cookie_name_value, e);
        unexpected_err_resp()
    })?;
    let mut headers = HeaderMap::new();
    let expired_cookie = create_expired_cookie_format(cookie_name_value)
        .parse::<HeaderValue>()
        .map_err(|e| {
            tracing::error!("failed to parse cookie ({}): {}", cookie_name_value, e);
            unexpected_err_resp()
        })?;
    headers.insert(SET_COOKIE, expired_cookie);
    Ok((StatusCode::OK, headers))
}

fn create_expired_cookie_format(cookie_name_value: &str) -> String {
    format!(
        // TODO: SSLのセットアップが完了し次第、Secureを追加する
        //"{}={}; SameSite=Strict; Path={}/; Max-Age=-1; Secure; HttpOnly",
        "{}={}; SameSite=Strict; Path={}/; Max-Age=-1; HttpOnly",
        COOKIE_NAME,
        cookie_name_value,
        ROOT_PATH
    )
}
