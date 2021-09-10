// Copyright 2021 Ken Miura

use std::time::Duration;

use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::extract::Extension;
use axum::http::{HeaderMap, StatusCode};
use chrono::{DateTime, Utc};
use common::ErrResp;
use common::{model::user::Account, DatabaseConnection, ValidCred};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection,
};

const LOGIN_SESSION_EXPIRY: Duration = Duration::from_secs(60 * 70);

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
    todo!()
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
