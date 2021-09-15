// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::{body::Body, http::Request, http::StatusCode};
use common::ErrResp;
use headers::{Cookie, HeaderMapExt};

use crate::util::unexpected_err_resp;

pub(crate) async fn get_refresh(req: Request<Body>) -> Result<StatusCode, ErrResp> {
    let headers = req.headers();
    let option_cookie = headers.typed_try_get::<Cookie>().map_err(|e| {
        tracing::error!("failed to get cookie: {}", e);
        unexpected_err_resp()
    })?;
    let extentions = req.extensions();
    let store = extentions.get::<RedisSessionStore>().ok_or_else(|| {
        tracing::error!("failed to get session store");
        unexpected_err_resp()
    })?;
    Ok(get_refresh_internal(option_cookie, store).await)
}

async fn get_refresh_internal(
    _option_cookie: Option<Cookie>,
    _store: &impl SessionStore,
) -> StatusCode {
    StatusCode::OK
}
