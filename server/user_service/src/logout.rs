// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::http::StatusCode;
use common::ErrResp;
use headers::{Cookie, HeaderMap, HeaderMapExt, HeaderValue};
use hyper::header::SET_COOKIE;

use crate::util::{
    session::{create_expired_cookie_format, extract_session_id, KEY_TO_USER_ACCOUNT_ID},
    unexpected_err_resp,
};

use axum::{body::Body, http::Request};

/// ログアウトを行う
/// <br>
/// リクエストにCookieが含まれていない場合、ステータスコード200を返す<br>
/// Cookieのnameが[COOKIE_NAME]でない場合、ステータスコード200を返す<br>
/// [COOKIE_NAME]の値と一致するセッションがない場合（既にセッションが期限切れの場合も含む）、ステータスコード200を返す<br>
/// [COOKIE_NAME]の値と一致するセッションがある場合、
/// セッションを削除（ログアウト）し、ステータスコード200と期限切れのCookie（ブラウザ上のCookieをブラウザに削除してもらうため）を返す<br>
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

async fn post_logout_internal(
    option_cookie: Option<Cookie>,
    store: &impl SessionStore,
) -> LogoutResult {
    let session_id_value = match extract_session_id(option_cookie) {
        Some(s) => s,
        None => {
            tracing::debug!("no valid cookie on logout");
            return Ok((StatusCode::OK, HeaderMap::new()));
        }
    };
    let option_session = store
        .load_session(session_id_value.to_string())
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
    match session.get::<i32>(KEY_TO_USER_ACCOUNT_ID) {
        Some(id) => tracing::info!("User (id: {}) logged out", id),
        None => tracing::info!("Someone logged out"),
    };
    let _ = store.destroy_session(session).await.map_err(|e| {
        tracing::error!(
            "failed to destroy session (session_id: {}): {}",
            session_id_value,
            e
        );
        unexpected_err_resp()
    })?;
    let mut headers = HeaderMap::new();
    let expired_cookie = create_expired_cookie_format(&session_id_value)
        .parse::<HeaderValue>()
        .map_err(|e| {
            tracing::error!("failed to parse cookie ({}): {}", session_id_value, e);
            unexpected_err_resp()
        })?;
    headers.insert(SET_COOKIE, expired_cookie);
    Ok((StatusCode::OK, headers))
}

#[cfg(test)]
mod tests {
    use async_session::{MemoryStore, Session, SessionStore};
    use headers::{Cookie, HeaderMap, HeaderMapExt, HeaderValue};

    use crate::util::{
        session::KEY_TO_USER_ACCOUNT_ID,
        session::{
            create_cookie_format,
            tests::{extract_cookie_max_age_value, extract_session_id_value},
        },
    };

    use super::*;

    #[tokio::test]
    async fn logout_success_session_alive() {
        let store = MemoryStore::new();
        let mut session = Session::new();
        let user_account_id = 203;
        // 実行環境（PCの性能）に依存させないように、テストコード内ではexpiryは設定しない
        let _ = session
            .insert(KEY_TO_USER_ACCOUNT_ID, user_account_id)
            .expect("failed to get Ok");
        let session_id_value = store
            .store_session(session)
            .await
            .expect("failed to get Ok")
            .expect("failed to get value");
        let mut headers = HeaderMap::new();
        let header_value = create_cookie_format(&session_id_value)
            .parse::<HeaderValue>()
            .expect("failed to get Ok");
        headers.insert("cookie", header_value);
        let option_cookie = headers.typed_get::<Cookie>();
        assert_eq!(1, store.count().await);

        let result = post_logout_internal(option_cookie, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
        assert_eq!(StatusCode::OK, result.0);
        let header_value = result.1.get(SET_COOKIE).expect("failed to get value");
        assert_eq!(session_id_value, extract_session_id_value(header_value));
        assert_eq!("-1", extract_cookie_max_age_value(header_value));
    }

    #[tokio::test]
    async fn logout_success_no_cookie() {
        let option_cookie: Option<Cookie> = None;
        let store = MemoryStore::new();

        let result = post_logout_internal(option_cookie, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert!(result.1.is_empty());
    }

    #[tokio::test]
    async fn logout_success_incorrect_cookie() {
        let mut headers = HeaderMap::new();
        let header_value = "name=taro"
            .parse::<HeaderValue>()
            .expect("failed to get Ok");
        headers.insert("cookie", header_value);
        let option_cookie = headers.typed_get::<Cookie>();
        let store = MemoryStore::new();

        let result = post_logout_internal(option_cookie, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert!(result.1.is_empty());
    }

    #[tokio::test]
    async fn logout_success_session_already_expired() {
        let store = MemoryStore::new();
        let mut session = Session::new();
        let user_account_id = 203;
        // 実行環境（PCの性能）に依存させないように、テストコード内ではexpiryは設定しない
        let _ = session
            .insert(KEY_TO_USER_ACCOUNT_ID, user_account_id)
            .expect("failed to get Ok");
        let session_id_value = store
            .store_session(session)
            .await
            .expect("failed to get Ok")
            .expect("failed to get value");
        let mut headers = HeaderMap::new();
        let header_value = create_cookie_format(&session_id_value)
            .parse::<HeaderValue>()
            .expect("failed to get Ok");
        headers.insert("cookie", header_value);
        let option_cookie = headers.typed_get::<Cookie>();

        // ログアウト前にセッションを削除
        let loaded_session = store
            .load_session(session_id_value)
            .await
            .expect("failed to get Ok")
            .expect("failed to get value");
        let _ = store
            .destroy_session(loaded_session)
            .await
            .expect("failed to get Ok");
        assert_eq!(0, store.count().await);

        let result = post_logout_internal(option_cookie, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
        assert_eq!(StatusCode::OK, result.0);
        assert!(result.1.is_empty());
    }
}
