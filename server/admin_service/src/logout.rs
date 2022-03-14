// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::{extract::Extension, http::StatusCode};
use common::ErrResp;
use headers::{HeaderMap, HeaderValue};
use hyper::header::SET_COOKIE;
use tower_cookies::Cookies;

use crate::{
    err::unexpected_err_resp,
    util::session::{create_expired_cookie_format, KEY_TO_USER_ACCOUNT_ID, SESSION_ID_COOKIE_NAME},
};

/// ログアウトを行う
/// <br>
/// リクエストにCookieが含まれていない場合、ステータスコード200を返す<br>
/// CookieにセッションIDを含まない場合、ステータスコード200を返す<br>
/// セッションIDの値と一致するセッションがない場合（既にセッションが期限切れの場合も含む）、ステータスコード200を返す<br>
/// セッションIDの値と一致するセッションがある場合、
/// セッションを削除（ログアウト）し、ステータスコード200と期限切れのCookie（ブラウザ上のCookieをブラウザに削除してもらうため）を返す<br>
pub(crate) async fn post_logout(
    cookies: Cookies,
    Extension(store): Extension<RedisSessionStore>,
) -> LogoutResult {
    handle_logout_req(cookies, &store).await
}

/// ログアウトリクエストの結果を示す型
pub(crate) type LogoutResult = Result<LogoutResp, ErrResp>;

/// ログアウトに成功した場合に返却される型
pub(crate) type LogoutResp = (StatusCode, HeaderMap);

async fn handle_logout_req(cookies: Cookies, store: &impl SessionStore) -> LogoutResult {
    let option_cookie = cookies.get(SESSION_ID_COOKIE_NAME);
    let session_id_value = match option_cookie {
        Some(session_id) => session_id.value().to_string(),
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
    let expired_cookie = create_expired_cookie_format()
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
    use async_session::MemoryStore;

    use crate::util::session::tests::{
        extract_cookie_max_age_value, extract_session_id_value, prepare_cookies, prepare_session,
        remove_session_from_store,
    };
    use tower_cookies::Cookie;

    use super::*;

    #[tokio::test]
    async fn handle_logout_req_success_session_alive() {
        let store = MemoryStore::new();
        let user_account_id = 203;
        let session_id_value = prepare_session(user_account_id, &store).await;
        let cookies = prepare_cookies(&session_id_value);
        assert_eq!(1, store.count().await);

        let result = handle_logout_req(cookies, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
        assert_eq!(StatusCode::OK, result.0);
        let header_value = result.1.get(SET_COOKIE).expect("failed to get value");
        assert_eq!("", extract_session_id_value(header_value));
        let max_age_value = extract_cookie_max_age_value(header_value)
            .parse::<i64>()
            .expect("failed to parse max age");
        assert!(max_age_value <= 0);
    }

    #[tokio::test]
    async fn handle_logout_req_success_no_cookie() {
        let cookies = Cookies::default();
        let store = MemoryStore::new();

        let result = handle_logout_req(cookies, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert!(result.1.is_empty());
    }

    #[tokio::test]
    async fn handle_logout_req_success_incorrect_cookie() {
        let cookies = Cookies::default();
        let cookie = Cookie::new("name", "taro");
        cookies.add(cookie);
        let store = MemoryStore::new();

        let result = handle_logout_req(cookies, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert!(result.1.is_empty());
    }

    #[tokio::test]
    async fn handle_logout_req_success_session_already_expired() {
        let store = MemoryStore::new();
        let user_account_id = 203;
        let session_id_value = prepare_session(user_account_id, &store).await;
        let option_cookie = prepare_cookies(&session_id_value);
        // ログアウト前にセッションを削除
        let _ = remove_session_from_store(&session_id_value, &store).await;
        assert_eq!(0, store.count().await);

        let result = handle_logout_req(option_cookie, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
        assert_eq!(StatusCode::OK, result.0);
        assert!(result.1.is_empty());
    }
}
