// Copyright 2021 Ken Miura

use async_fred_session::RedisSessionStore;
use async_session::SessionStore;
use axum::{extract::State, http::StatusCode};
use axum_extra::extract::{cookie::Cookie, SignedCookieJar};
use common::ErrResp;
use tracing::{error, info};

use super::super::{ADMIN_SESSION_ID_COOKIE_NAME, KEY_TO_ADMIN_ACCOUNT_ID};
use crate::err::unexpected_err_resp;

/// ログアウトを行う
/// <br>
/// リクエストにnameが[ADMIN_SESSION_ID_COOKIE_NAME]のCookieが含まれていない場合、ステータスコード200を返す<br>
/// セッションIDに対応するセッションがない場合（既にセッションが期限切れの場合も含む）、ステータスコード200を返す<br>
/// セッションIDに対応するセッションがある場合、セッションを削除（ログアウト）し、ステータスコード200と期限切れのCookieを返す<br>
/// （期限切れのCookieを返すのは、ブラウザ上のCookieをブラウザ自体に削除してもらうため）<br>
pub(crate) async fn post_logout(
    jar: SignedCookieJar,
    State(store): State<RedisSessionStore>,
) -> Result<(StatusCode, SignedCookieJar), ErrResp> {
    let option_cookie = jar.get(ADMIN_SESSION_ID_COOKIE_NAME);
    let session_id = match option_cookie {
        Some(s) => s.value().to_string(),
        None => {
            info!("no sessoin cookie found");
            return Ok((
                StatusCode::OK,
                jar.remove(Cookie::named(ADMIN_SESSION_ID_COOKIE_NAME)),
            ));
        }
    };
    handle_logout_req(session_id, &store).await?;
    Ok((
        StatusCode::OK,
        jar.remove(Cookie::named(ADMIN_SESSION_ID_COOKIE_NAME)),
    ))
}

async fn handle_logout_req<'a>(
    session_id: String,
    store: &impl SessionStore,
) -> Result<(), ErrResp> {
    let option_session = store
        .load_session(session_id.to_string())
        .await
        .map_err(|e| {
            error!("failed to load session: {}", e);
            unexpected_err_resp()
        })?;
    let session = match option_session {
        Some(s) => s,
        None => {
            info!("no session found on logout");
            return Ok(());
        }
    };
    let account_id = match session.get::<i64>(KEY_TO_ADMIN_ACCOUNT_ID) {
        Some(id) => {
            info!("admin (admin account id: {}) logged out", id);
            Some(id)
        }
        None => {
            info!("someone logged out");
            None
        }
    };
    store.destroy_session(session).await.map_err(|e| {
        error!(
            "failed to destroy session (admin account id: {:?}): {}",
            account_id, e
        );
        unexpected_err_resp()
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use async_session::MemoryStore;

    use crate::handlers::session::tests::{prepare_session, remove_session_from_store};

    use super::*;

    #[tokio::test]
    async fn handle_logout_req_success_session_alive() {
        let store = MemoryStore::new();
        let admin_account_id = 203;
        let session_id = prepare_session(admin_account_id, &store).await;
        assert_eq!(1, store.count().await);

        handle_logout_req(session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
    }

    #[tokio::test]
    async fn handle_logout_req_success_session_already_expired() {
        let store = MemoryStore::new();
        let admin_account_id = 203;
        let session_id = prepare_session(admin_account_id, &store).await;
        // ログアウト前にセッションを削除
        remove_session_from_store(&session_id, &store).await;
        assert_eq!(0, store.count().await);

        handle_logout_req(session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
    }
}