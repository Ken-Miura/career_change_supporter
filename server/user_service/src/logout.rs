// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::{extract::Extension, http::StatusCode};
use common::ErrResp;
use tower_cookies::{Cookie, Cookies};

use crate::{
    err::unexpected_err_resp,
    util::session::{KEY_TO_USER_ACCOUNT_ID, SESSION_ID_COOKIE_NAME},
};

/// ログアウトを行う
/// <br>
/// リクエストにnameが[SESSION_ID_COOKIE_NAME]のCookieが含まれていない場合、ステータスコード200を返す<br>
/// セッションIDに対応するセッションがない場合（既にセッションが期限切れの場合も含む）、ステータスコード200を返す<br>
/// セッションIDに対応するセッションがある場合、セッションを削除（ログアウト）し、ステータスコード200と期限切れのCookieを返す<br>
/// （期限切れのCookieを返すのは、ブラウザ上のCookieをブラウザ自体に削除してもらうため）<br>
pub(crate) async fn post_logout(
    cookies: Cookies,
    Extension(store): Extension<RedisSessionStore>,
) -> Result<StatusCode, ErrResp> {
    let option_cookie = cookies.get(SESSION_ID_COOKIE_NAME);
    let session_id = match option_cookie {
        Some(s) => s.value().to_string(),
        None => {
            tracing::debug!("no sessoin cookie found");
            return Ok(StatusCode::OK);
        }
    };
    let _ = handle_logout_req(session_id, &store).await?;
    // removeというメソッド名がわかりづらいが、Set-Cookieにmax-ageが0のCookieをセットしている。
    let _ = cookies.remove(Cookie::new(SESSION_ID_COOKIE_NAME, ""));
    Ok(StatusCode::OK)
}

async fn handle_logout_req<'a>(
    session_id: String,
    store: &impl SessionStore,
) -> Result<(), ErrResp> {
    let option_session = store
        .load_session(session_id.to_string())
        .await
        .map_err(|e| {
            tracing::error!("failed to load session: {}", e);
            unexpected_err_resp()
        })?;
    let session = match option_session {
        Some(s) => s,
        None => {
            tracing::debug!("no session found in session store on logout");
            return Ok(());
        }
    };
    match session.get::<i32>(KEY_TO_USER_ACCOUNT_ID) {
        Some(id) => tracing::info!("User (account id: {}) logged out", id),
        None => tracing::info!("Someone logged out"),
    };
    let _ = store.destroy_session(session).await.map_err(|e| {
        tracing::error!(
            "failed to destroy session (session_id: {}): {}",
            session_id,
            e
        );
        unexpected_err_resp()
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use async_session::MemoryStore;

    use crate::{
        logout::handle_logout_req,
        util::session::tests::{prepare_session, remove_session_from_store},
    };

    #[tokio::test]
    async fn handle_logout_req_success_session_alive() {
        let store = MemoryStore::new();
        let user_account_id = 203;
        let session_id = prepare_session(user_account_id, &store).await;
        assert_eq!(1, store.count().await);

        let _ = handle_logout_req(session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
    }

    #[tokio::test]
    async fn handle_logout_req_success_session_already_expired() {
        let store = MemoryStore::new();
        let user_account_id = 203;
        let session_id = prepare_session(user_account_id, &store).await;
        // ログアウト前にセッションを削除
        let _ = remove_session_from_store(&session_id, &store).await;
        assert_eq!(0, store.count().await);

        let _ = handle_logout_req(session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
    }
}
