// Copyright 2021 Ken Miura

use async_fred_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::{extract::State, http::StatusCode};
use axum_extra::extract::{cookie::Cookie, SignedCookieJar};
use common::ErrResp;
use tracing::{error, info};

use crate::{
    err::unexpected_err_resp,
    handlers::session::{KEY_TO_LOGIN_STATUS, KEY_TO_USER_ACCOUNT_ID, SESSION_ID_COOKIE_NAME},
};

/// ログアウトを行う
/// <br>
/// リクエストにnameが[SESSION_ID_COOKIE_NAME]のCookieが含まれていない場合、ステータスコード200を返す<br>
/// セッションIDに対応するセッションがない場合（既にセッションが期限切れの場合も含む）、ステータスコード200を返す<br>
/// セッションIDに対応するセッションがある場合、セッションを削除（ログアウト）し、ステータスコード200と期限切れのCookieを返す<br>
/// （期限切れのCookieを返すのは、ブラウザ上のCookieをブラウザ自体に削除してもらうため）<br>
pub(crate) async fn post_logout(
    jar: SignedCookieJar,
    State(store): State<RedisSessionStore>,
) -> Result<(StatusCode, SignedCookieJar), ErrResp> {
    let option_cookie = jar.get(SESSION_ID_COOKIE_NAME);
    let session_id = match option_cookie {
        Some(s) => s.value().to_string(),
        None => {
            info!("no sessoin cookie found");
            return Ok((
                StatusCode::OK,
                jar.remove(Cookie::named(SESSION_ID_COOKIE_NAME)),
            ));
        }
    };
    handle_logout_req(session_id, &store).await?;
    Ok((
        StatusCode::OK,
        jar.remove(Cookie::named(SESSION_ID_COOKIE_NAME)),
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

    record_logout_info(&session);

    store.destroy_session(session).await.map_err(|e| {
        error!("failed to destroy session: {}", e);
        unexpected_err_resp()
    })?;
    Ok(())
}

fn record_logout_info(session: &Session) {
    let account_id = session.get::<i64>(KEY_TO_USER_ACCOUNT_ID);
    let login_status = session.get::<String>(KEY_TO_LOGIN_STATUS);
    info!(
        "user logged out: session info (account_id: {:?}, login_status: {:?})",
        account_id, login_status
    );
}

#[cfg(test)]
mod tests {
    use async_session::MemoryStore;

    use crate::{
        handlers::session::{
            authentication::logout::handle_logout_req,
            tests::{prepare_session, remove_session_from_store},
        },
        util::login_status::LoginStatus,
    };

    #[tokio::test]
    async fn handle_logout_req_success_session_alive() {
        let store = MemoryStore::new();
        let user_account_id = 203;
        let session_id = prepare_session(user_account_id, LoginStatus::Finish, &store).await;
        assert_eq!(1, store.count().await);

        handle_logout_req(session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
    }

    // 通常のUI操作でこのパターンが発生することはないが、テストして問題ないことは確認しておく
    #[tokio::test]
    async fn handle_logout_req_success_session_alive_during_mfa_login_sequence() {
        let store = MemoryStore::new();
        let user_account_id = 203;
        let session_id =
            prepare_session(user_account_id, LoginStatus::NeedMoreVerification, &store).await;
        assert_eq!(1, store.count().await);

        handle_logout_req(session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
    }

    #[tokio::test]
    async fn handle_logout_req_success_session_already_expired() {
        let store = MemoryStore::new();
        let user_account_id = 203;
        let session_id = prepare_session(user_account_id, LoginStatus::Finish, &store).await;
        // ログアウト前にセッションを削除
        remove_session_from_store(&session_id, &store).await;
        assert_eq!(0, store.count().await);

        handle_logout_req(session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
    }
}
