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

#[cfg(test)]
mod tests {
    use async_session::MemoryStore;
    use headers::{Cookie, HeaderMap, HeaderMapExt, HeaderValue};
    use hyper::StatusCode;

    use crate::{
        refresh::get_refresh_internal,
        util::session::{
            tests::{prepare_cookie_temp, prepare_session, remove_session_from_store},
            LOGIN_SESSION_EXPIRY,
        },
    };

    use super::RefreshOperation;

    struct RefreshOperationMock {
        expiry: std::time::Duration,
    }

    impl RefreshOperation for RefreshOperationMock {
        fn set_login_session_expiry(
            &self,
            _session: &mut async_session::Session,
            expiry: std::time::Duration,
        ) {
            assert_eq!(self.expiry, expiry);
        }
    }

    #[tokio::test]
    async fn refresh_success() {
        let store = MemoryStore::new();
        let user_account_id = 555;
        let session_id_value = prepare_session(user_account_id, &store).await;
        let option_cookie = prepare_cookie_temp(&session_id_value);
        assert_eq!(1, store.count().await);
        let op_mock = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };

        let result =
            get_refresh_internal(option_cookie, &store, op_mock, LOGIN_SESSION_EXPIRY).await;

        assert_eq!(1, store.count().await);
        let code = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, code);
    }

    #[tokio::test]
    async fn refresh_fail_no_cookie() {
        let option_cookie: Option<Cookie> = None;
        let store = MemoryStore::new();
        let op_mock = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };

        let result =
            get_refresh_internal(option_cookie, &store, op_mock, LOGIN_SESSION_EXPIRY).await;

        let code = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::UNAUTHORIZED, code);
    }

    #[tokio::test]
    async fn refresh_fail_incorrect_cookie() {
        let mut headers = HeaderMap::new();
        let header_value = "name=taro"
            .parse::<HeaderValue>()
            .expect("failed to get Ok");
        headers.insert("cookie", header_value);
        let option_cookie = headers.typed_get::<Cookie>();
        let store = MemoryStore::new();
        let op_mock = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };

        let result =
            get_refresh_internal(option_cookie, &store, op_mock, LOGIN_SESSION_EXPIRY).await;

        let code = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::UNAUTHORIZED, code);
    }

    #[tokio::test]
    async fn refresh_fail_session_already_expired() {
        let store = MemoryStore::new();
        let user_account_id = 203;
        let session_id_value = prepare_session(user_account_id, &store).await;
        let option_cookie = prepare_cookie_temp(&session_id_value);
        // リフレッシュ前にセッションを削除
        let _ = remove_session_from_store(&session_id_value, &store).await;
        assert_eq!(0, store.count().await);
        let op_mock = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };

        let result =
            get_refresh_internal(option_cookie, &store, op_mock, LOGIN_SESSION_EXPIRY).await;

        assert_eq!(0, store.count().await);
        let code = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::UNAUTHORIZED, code);
    }
}
