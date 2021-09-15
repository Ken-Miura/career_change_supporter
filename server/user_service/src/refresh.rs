// Copyright 2021 Ken Miura

use async_session::SessionStore;
use axum::{body::Body, http::Request, http::StatusCode};
use headers::Cookie;

pub(crate) async fn get_refresh(_req: Request<Body>) {}

async fn get_refresh_internal(
    _option_cookie: Option<Cookie>,
    _store: &impl SessionStore,
) -> StatusCode {
    StatusCode::OK
}
