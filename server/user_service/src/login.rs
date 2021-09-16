// Copyright 2021 Ken Miura

use std::time::Duration;

use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::extract::Extension;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::Json;
use chrono::{DateTime, Utc};
use common::schema::ccs_schema::user_account::dsl::{email_address, last_login_time, user_account};
use common::util::is_password_match;
use common::{model::user::Account, DatabaseConnection, ValidCred};
use common::{ApiError, ErrResp};
use diesel::query_builder::functions::update;
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use hyper::header::SET_COOKIE;

use crate::err_code::EMAIL_OR_PWD_INCORRECT;
use crate::util::session::LOGIN_SESSION_EXPIRY;
use crate::util::{
    session::create_cookie_format, session::KEY_TO_USER_ACCOUNT_ID, unexpected_err_resp,
};

/// ログインを行う<br>
/// ログインに成功した場合、ステータスコードに200、ヘッダにセッションにアクセスするためのcoookieをセットして応答する<br>
/// <br>
/// # Errors
/// email addressもしくはpasswordが正しくない場合、ステータスコード401、エラーコード[EMAIL_OR_PWD_INCORRECT]を返す<br>
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

/// ログインリクエストの結果を示す型
pub(crate) type LoginResult = Result<LoginResp, ErrResp>;

/// ログインに成功した場合に返却される型
pub(crate) type LoginResp = (StatusCode, HeaderMap);

async fn post_login_internal(
    email_addr: &str,
    password: &str,
    login_time: &DateTime<Utc>,
    op: impl LoginOperation,
    store: impl SessionStore,
) -> LoginResult {
    let accounts = op.filter_account_by_email_addr(email_addr)?;
    let _ = ensure_account_is_only_one(email_addr, &accounts)?;
    let account = accounts[0].clone();
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
    let session_id_value = match option {
        Some(s) => s,
        None => {
            tracing::error!(
                "failed to get cookie name for user account id ({})",
                user_account_id
            );
            return Err(unexpected_err_resp());
        }
    };
    let _ = op.update_last_login(user_account_id, login_time)?;
    tracing::info!("{} logged-in at {}", email_addr, login_time);
    let mut headers = HeaderMap::new();
    let cookie = create_cookie_format(&session_id_value)
        .parse::<HeaderValue>()
        .map_err(|e| {
            tracing::error!(
                "failed to parse cookie (session_id: {}): {}",
                session_id_value,
                e
            );
            unexpected_err_resp()
        })?;
    headers.insert(SET_COOKIE, cookie);
    Ok((StatusCode::OK, headers))
}

fn ensure_account_is_only_one(email_addr: &str, accounts: &[Account]) -> Result<(), ErrResp> {
    let num = accounts.len();
    if num == 0 {
        tracing::error!("unauthorized: {}", email_addr);
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: EMAIL_OR_PWD_INCORRECT,
            }),
        ));
    }
    if num != 1 {
        tracing::error!("multiple email addresses: {}", email_addr);
        return Err(unexpected_err_resp());
    }
    Ok(())
}

trait LoginOperation {
    fn filter_account_by_email_addr(&self, email_addr: &str) -> Result<Vec<Account>, ErrResp>;
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
    fn filter_account_by_email_addr(&self, email_addr: &str) -> Result<Vec<Account>, ErrResp> {
        let result = user_account
            .filter(email_address.eq(email_addr))
            .load::<Account>(&self.conn);
        match result {
            Ok(accounts) => Ok(accounts),
            Err(e) => {
                tracing::error!("failed to load accounts ({}): {}", email_addr, e);
                Err(unexpected_err_resp())
            }
        }
    }

    fn set_login_session_expiry(&self, session: &mut Session) {
        session.expire_in(self.expiry);
    }

    fn update_last_login(&self, id: i32, login_time: &DateTime<Utc>) -> Result<(), ErrResp> {
        let _ = update(user_account.find(id))
            .set(last_login_time.eq(login_time))
            .execute(&self.conn)
            .map_err(|e| {
                tracing::error!(
                    "failed to update last login time ({}) on id ({}): {}",
                    login_time,
                    id,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_session::MemoryStore;
    use async_session::SessionStore;
    use chrono::TimeZone;
    use common::util::hash_password;
    use common::util::validator::validate_email_address;
    use common::util::validator::validate_password;

    use crate::util::session::tests::extract_session_id_value;

    use super::*;

    struct LoginOperationMock<'a> {
        account: Account,
        login_time: &'a DateTime<Utc>,
    }

    impl<'a> LoginOperationMock<'a> {
        fn new(account: Account, login_time: &'a DateTime<Utc>) -> Self {
            Self {
                account,
                login_time,
            }
        }
    }

    impl<'a> LoginOperation for LoginOperationMock<'a> {
        fn filter_account_by_email_addr(&self, email_addr: &str) -> Result<Vec<Account>, ErrResp> {
            if self.account.email_address == email_addr {
                Ok(vec![self.account.clone()])
            } else {
                Ok(vec![])
            }
        }

        fn set_login_session_expiry(&self, _session: &mut Session) {
            // テスト実行中に有効期限が過ぎるケースを考慮し、有効期限は設定しない
        }

        fn update_last_login(&self, id: i32, login_time: &DateTime<Utc>) -> Result<(), ErrResp> {
            assert_eq!(self.account.user_account_id, id);
            assert_eq!(self.login_time, login_time);
            Ok(())
        }
    }

    #[tokio::test]
    async fn login_success() {
        let id = 1102;
        let email_addr = "test@example.com";
        let pwd = "1234567890abcdABCD";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let _ = validate_password(pwd).expect("failed to get Ok");
        let hashed_pwd = hash_password(pwd).expect("failed to hash pwd");
        let creation_time = Utc.ymd(2021, 9, 11).and_hms(15, 30, 45);
        let last_login = creation_time + chrono::Duration::days(1);
        let account = Account {
            user_account_id: id,
            email_address: email_addr.to_string(),
            hashed_password: hashed_pwd,
            last_login_time: Some(last_login),
            created_at: creation_time,
        };
        let store = MemoryStore::new();
        let current_date_time = last_login + chrono::Duration::days(1);
        let op = LoginOperationMock::new(account, &current_date_time);

        let result =
            post_login_internal(email_addr, pwd, &current_date_time, op, store.clone()).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        let header_value = resp.1.get(SET_COOKIE).expect("failed to get value");
        let session_id = extract_session_id_value(header_value);
        let session = store
            .load_session(session_id)
            .await
            .expect("failed to get Ok")
            .expect("failed to get value");
        let actual_id = session
            .get::<i32>(KEY_TO_USER_ACCOUNT_ID)
            .expect("failed to get value");
        assert_eq!(id, actual_id);
    }

    #[tokio::test]
    async fn login_fail_no_email_addr_found() {
        let id = 1102;
        let email_addr1 = "test1@example.com";
        let email_addr2 = "test2@example.com";
        let pwd = "1234567890abcdABCD";
        let _ = validate_email_address(email_addr1).expect("failed to get Ok");
        let _ = validate_email_address(email_addr2).expect("failed to get Ok");
        let _ = validate_password(pwd).expect("failed to get Ok");
        let hashed_pwd = hash_password(pwd).expect("failed to hash pwd");
        let creation_time = Utc.ymd(2021, 9, 11).and_hms(15, 30, 45);
        let last_login = creation_time + chrono::Duration::days(1);
        let account = Account {
            user_account_id: id,
            email_address: email_addr1.to_string(),
            hashed_password: hashed_pwd,
            last_login_time: Some(last_login),
            created_at: creation_time,
        };
        let store = MemoryStore::new();
        let current_date_time = last_login + chrono::Duration::days(1);
        let op = LoginOperationMock::new(account, &current_date_time);

        let result =
            post_login_internal(email_addr2, pwd, &current_date_time, op, store.clone()).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::UNAUTHORIZED, resp.0);
        assert_eq!(EMAIL_OR_PWD_INCORRECT, resp.1.code);
        assert_eq!(0, store.count().await);
    }

    #[tokio::test]
    async fn login_fail_incorrect_password() {
        let id = 1102;
        let email_addr = "test1@example.com";
        let pwd1 = "1234567890abcdABCD";
        let pwd2 = "bbbbbbbbbC";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let _ = validate_password(pwd1).expect("failed to get Ok");
        let _ = validate_password(pwd2).expect("failed to get Ok");
        let hashed_pwd = hash_password(pwd1).expect("failed to hash pwd");
        let creation_time = Utc.ymd(2021, 9, 11).and_hms(15, 30, 45);
        let last_login = creation_time + chrono::Duration::days(1);
        let account = Account {
            user_account_id: id,
            email_address: email_addr.to_string(),
            hashed_password: hashed_pwd,
            last_login_time: Some(last_login),
            created_at: creation_time,
        };
        let store = MemoryStore::new();
        let current_date_time = last_login + chrono::Duration::days(1);
        let op = LoginOperationMock::new(account, &current_date_time);

        let result =
            post_login_internal(email_addr, pwd2, &current_date_time, op, store.clone()).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::UNAUTHORIZED, resp.0);
        assert_eq!(EMAIL_OR_PWD_INCORRECT, resp.1.code);
        assert_eq!(0, store.count().await);
    }
}
