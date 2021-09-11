// Copyright 2021 Ken Miura

use std::time::Duration;

use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::extract::Extension;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::Json;
use chrono::{DateTime, Utc};
use common::util::is_password_match;
use common::{model::user::Account, DatabaseConnection, ValidCred};
use common::{ApiError, ErrResp};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection,
};
use hyper::header::SET_COOKIE;

use crate::err_code::EMAIL_OR_PWD_INCORRECT;
use crate::util::unexpected_err_resp;

const LENGTH_OF_MEETING: u64 = 60;
const TIME_FOR_SUBSEQUENT_OPERATIONS: u64 = 10;
const LOGIN_SESSION_EXPIRY: Duration =
    Duration::from_secs(60 * (LENGTH_OF_MEETING + TIME_FOR_SUBSEQUENT_OPERATIONS));
const KEY_TO_USER_ACCOUNT_ID: &str = "user_account_id";

/// ログインを行う<br>
/// <br>
/// # Errors
///
pub(crate) async fn post_login(
    ValidCred(cred): ValidCred,
    DatabaseConnection(conn): DatabaseConnection,
    Extension(store): Extension<RedisSessionStore>,
) -> LoginResult {
    let email_addr = cred.email_address;
    let password = cred.password;
    let current_date_time = Utc::now();
    let op = LoginOperationImpl::new(conn, LOGIN_SESSION_EXPIRY);
    post_login_internal(&email_addr, &password, &current_date_time, op, store).await
}

///
pub(crate) type LoginResult = Result<LoginResp, ErrResp>;

///
pub(crate) type LoginResp = (StatusCode, HeaderMap);

async fn post_login_internal(
    email_addr: &str,
    password: &str,
    login_time: &DateTime<Utc>,
    op: impl LoginOperation,
    store: impl SessionStore,
) -> LoginResult {
    let account = op.find_account_by_email_addr(email_addr)?;
    let matched = is_password_match(password, &account.hashed_password).map_err(|e| {
        tracing::error!("failed to handle password: {}", e);
        unexpected_err_resp()
    })?;
    if !matched {
        tracing::error!("unauthorized: {}", email_addr);
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: EMAIL_OR_PWD_INCORRECT,
            }),
        ));
    }
    let user_account_id = account.user_account_id;
    let mut session = Session::new();
    let _ = session
        .insert(KEY_TO_USER_ACCOUNT_ID, user_account_id)
        .map_err(|e| {
            tracing::error!(
                "failed to insert id ({}) into session: {}",
                user_account_id,
                e
            );
            unexpected_err_resp()
        })?;
    op.set_login_session_expiry(&mut session);
    let option = store.store_session(session).await.map_err(|e| {
        tracing::error!(
            "failed to store session for id ({}): {}",
            user_account_id,
            e
        );
        unexpected_err_resp()
    })?;
    let cookie_value = match option {
        Some(c) => c,
        None => {
            tracing::error!("failed to get cookie for id ({})", user_account_id);
            return Err(unexpected_err_resp());
        }
    };
    let _ = op.update_last_login(user_account_id, login_time)?;
    tracing::info!("{} logged-in at {}", email_addr, login_time);
    let mut headers = HeaderMap::new();
    let cookie = create_cookie_format(&cookie_value)
        .parse::<HeaderValue>()
        .map_err(|e| {
            tracing::error!("failed to parse cookie ({}): {}", cookie_value, e);
            unexpected_err_resp()
        })?;
    headers.insert(SET_COOKIE, cookie);
    Ok((StatusCode::OK, headers))
}

fn create_cookie_format(cookie_value: &str) -> String {
    format!(
        // TODO: SSLのセットアップが完了し次第、Secureを追加する
        //"session={}; SameSite=Strict; Path=/api/; Secure; HttpOnly",
        "session={}; SameSite=Strict; Path=/api/; HttpOnly",
        cookie_value
    )
}

trait LoginOperation {
    fn find_account_by_email_addr(&self, email_addr: &str) -> Result<Account, ErrResp>;
    fn set_login_session_expiry(&self, session: &mut Session);
    fn update_last_login(&self, id: i32, login_time: &DateTime<Utc>) -> Result<(), ErrResp>;
}

struct LoginOperationImpl {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
    expiry: Duration,
}

impl LoginOperationImpl {
    fn new(conn: PooledConnection<ConnectionManager<PgConnection>>, expiry: Duration) -> Self {
        Self { conn, expiry }
    }
}

impl LoginOperation for LoginOperationImpl {
    fn find_account_by_email_addr(&self, email_addr: &str) -> Result<Account, ErrResp> {
        todo!()
    }

    fn set_login_session_expiry(&self, session: &mut Session) {
        todo!()
    }

    fn update_last_login(&self, id: i32, login_time: &DateTime<Utc>) -> Result<(), ErrResp> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn login_success() {
        // ログイン準備
        // ログイン
        // ログイン結果
        //   ステータスコード200
        //   ヘッダにsessionを取得可能なcookieがセットされている
        //   最終ログインが更新されている
    }
}
