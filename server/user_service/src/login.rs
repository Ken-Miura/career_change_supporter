// Copyright 2021 Ken Miura

use std::time::Duration;

use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::extract::Extension;
use axum::http::{HeaderMap, StatusCode};
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
    let op = LoginOperationImpl::new(conn, Some(LOGIN_SESSION_EXPIRY));
    post_login_internal(&email_addr, &password, op, store).await
}

///
pub(crate) type LoginResult = Result<LoginResp, ErrResp>;

///
pub(crate) type LoginResp = (StatusCode, HeaderMap);

async fn post_login_internal(
    email_addr: &str,
    password: &str,
    op: impl LoginOperation,
    store: impl SessionStore,
) -> LoginResult {
    todo!()
}

trait LoginOperation {
    fn find_account_by_email_addr(&self, email_addr: &str) -> Result<Account, ErrResp>;
    fn set_login_session_expiry(&self, session: &mut Session) -> Result<Account, ErrResp>;
}

struct LoginOperationImpl {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
    expiry: Option<Duration>,
}

impl LoginOperationImpl {
    fn new(
        conn: PooledConnection<ConnectionManager<PgConnection>>,
        expiry: Option<Duration>,
    ) -> Self {
        Self { conn, expiry }
    }
}

impl LoginOperation for LoginOperationImpl {
    fn find_account_by_email_addr(&self, email_addr: &str) -> Result<Account, ErrResp> {
        todo!()
    }

    fn set_login_session_expiry(&self, session: &mut Session) -> Result<Account, ErrResp> {
        todo!()
    }
}
