// Copyright 2021 Ken Miura

use std::time::Duration;

use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::{body::Body, http::Request, http::StatusCode};
use headers::{Cookie, HeaderMapExt};

use crate::util::session::{extract_session_id, LOGIN_SESSION_EXPIRY};

/// ログインセッションを延長する<br>
/// セッションが有効な間に呼び出すと、セッションの有効期限を[LOGIN_SESSION_EXPIRY]だけ延長し、ステータスコード200を返す。<br>
/// <br>
/// # Errors
/// 下記の場合、ステータスコード401を返す。<br>
/// <ul>
///   <li>ヘッダにCookieがない場合</li>
///   <li>CookieにセッションIDが含まれていない場合</li>
///   <li>既にセッションの有効期限が切れている場合</li>
/// </ul>
pub(crate) async fn get_refresh(req: Request<Body>) -> Result<StatusCode, StatusCode> {
    let headers = req.headers();
    let option_cookie = headers.typed_try_get::<Cookie>().map_err(|e| {
        tracing::error!("failed to get cookie: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let extentions = req.extensions();
    let store = extentions.get::<RedisSessionStore>().ok_or_else(|| {
        tracing::error!("failed to get session store");
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    let op = RefreshOperationImpl {};
    get_refresh_internal(option_cookie, store, op, LOGIN_SESSION_EXPIRY).await
}

async fn get_refresh_internal(
    option_cookie: Option<Cookie>,
    store: &impl SessionStore,
    op: impl RefreshOperation,
    expiry: Duration,
) -> Result<StatusCode, StatusCode> {
    let session_id_value = match extract_session_id(option_cookie) {
        Some(s) => s,
        None => {
            tracing::debug!("no valid cookie on refresh");
            return Err(StatusCode::UNAUTHORIZED);
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
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let mut session = match option_session {
        Some(s) => s,
        None => {
            tracing::debug!("no valid session on refresh");
            return Err(StatusCode::UNAUTHORIZED);
        }
    };
    op.set_login_session_expiry(&mut session, expiry);
    // 新たなexpiryを設定したsessionをstoreに保存することでセッション期限を延長する
    let _ = store.store_session(session).await.map_err(|e| {
        tracing::error!(
            "failed to store session (session_id={}): {}",
            session_id_value,
            e
        );
        StatusCode::INTERNAL_SERVER_ERROR
    })?;
    Ok(StatusCode::OK)
}

trait RefreshOperation {
    fn set_login_session_expiry(&self, session: &mut Session, expiry: Duration);
}

struct RefreshOperationImpl {}

impl RefreshOperation for RefreshOperationImpl {
    fn set_login_session_expiry(&self, session: &mut Session, expiry: Duration) {
        session.expire_in(expiry);
    }
}
