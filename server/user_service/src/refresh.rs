// Copyright 2021 Ken Miura

use std::time::Duration;

use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::{body::Body, http::Request, http::StatusCode};
use common::ErrResp;
use headers::{Cookie, HeaderMapExt};

use crate::util::{
    session::{extract_session_id, LOGIN_SESSION_EXPIRY},
    unexpected_err_resp,
};

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
    let op = RefreshOperationImpl::new(LOGIN_SESSION_EXPIRY);
    get_refresh_internal(option_cookie, store, op).await
}

async fn get_refresh_internal(
    option_cookie: Option<Cookie>,
    store: &impl SessionStore,
    op: impl RefreshOperation,
) -> Result<StatusCode, ErrResp> {
    let session_id_value = match extract_session_id(option_cookie) {
        Some(s) => s,
        None => {
            tracing::debug!("no valid cookie on refresh");
            return Ok(StatusCode::UNAUTHORIZED);
        }
    };
    let option_session = store
        .load_session(session_id_value.clone())
        .await
        .map_err(|e| {
            tracing::error!(
                "failed to load session (session_id={}): {}",
                session_id_value,
                e
            );
            unexpected_err_resp()
        })?;
    let mut session = match option_session {
        Some(s) => s,
        None => {
            tracing::debug!("no valid session on refresh");
            return Ok(StatusCode::UNAUTHORIZED);
        }
    };
    op.set_login_session_expiry(&mut session);
    let _ = store.store_session(session).await.map_err(|e| {
        tracing::error!(
            "failed to store session (session_id={}): {}",
            session_id_value,
            e
        );
        unexpected_err_resp()
    })?;
    Ok(StatusCode::OK)
}

trait RefreshOperation {
    fn set_login_session_expiry(&self, session: &mut Session);
}

struct RefreshOperationImpl {
    expiry: Duration,
}

impl RefreshOperationImpl {
    fn new(expiry: Duration) -> Self {
        Self { expiry }
    }
}

impl RefreshOperation for RefreshOperationImpl {
    fn set_login_session_expiry(&self, session: &mut Session) {
        session.expire_in(self.expiry);
    }
}
