// Copyright 2021 Ken Miura

use std::time::Duration;

use async_fred_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::async_trait;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::SignedCookieJar;
use chrono::{DateTime, FixedOffset, Utc};
use common::password::is_password_match;
use common::util::create_session_cookie;
use common::{ApiError, ErrResp, DUMMY_HASHED_PASSWORD};
use common::{ValidCred, JAPANESE_TIME_ZONE};
use entity::prelude::UserAccount;
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use entity::user_account;
use serde::Serialize;
use tracing::{error, info};

use super::update_last_login;
use crate::err::unexpected_err_resp;
use crate::err::Code::{AccountDisabled, EmailOrPwdIncorrect};
use crate::handlers::session::authentication::{
    LoginStatus, KEY_TO_LOGIN_STATUS, KEY_TO_USER_ACCOUNT_ID, LOGIN_SESSION_EXPIRY,
};
use crate::handlers::session::SESSION_ID_COOKIE_NAME;
use crate::handlers::ROOT_PATH;

/// ログインを行う<br>
/// ログインに成功した場合、ステータスコードに200、ヘッダにセッションにアクセスするためのcookie、ログイン処理の状態（完了、または二段階目の認証が必要）をセットして応答する<br>
/// <br>
/// # Errors
/// - email addressもしくはpasswordが正しくない場合、ステータスコード401、エラーコード[EmailOrPwdIncorrect]を返す<br>
/// - email addressとpasswordが正しく、かつアカウントが無効化されている場合、ステータスコード400、エラーコード[AccountDisabled]を返す<br>
pub(crate) async fn post_login(
    jar: SignedCookieJar,
    State(pool): State<DatabaseConnection>,
    State(store): State<RedisSessionStore>,
    ValidCred(cred): ValidCred,
) -> Result<(StatusCode, SignedCookieJar, Json<LoginResult>), ErrResp> {
    let email_addr = cred.email_address;
    let password = cred.password;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = LoginOperationImpl::new(pool, LOGIN_SESSION_EXPIRY);

    let session_id_and_login_status =
        handle_login_req(&email_addr, &password, &current_date_time, op, store).await?;

    let session_id = session_id_and_login_status.0;
    let cookie = create_session_cookie(
        SESSION_ID_COOKIE_NAME.to_string(),
        session_id,
        ROOT_PATH.to_string(),
    );
    let login_status = session_id_and_login_status.1;

    Ok((
        StatusCode::OK,
        jar.add(cookie),
        Json(LoginResult { login_status }),
    ))
}

#[derive(Serialize, Debug)]
pub(crate) struct LoginResult {
    login_status: LoginStatus,
}

async fn handle_login_req(
    email_addr: &str,
    password: &str,
    login_time: &DateTime<FixedOffset>,
    op: impl LoginOperation,
    store: impl SessionStore,
) -> Result<(String, LoginStatus), ErrResp> {
    let account = find_account_by_email_address(email_addr, password, &op).await?;
    verify_password(email_addr, password, &account.hashed_password)?;

    ensure_account_is_not_disabled(account.user_account_id, email_addr, account.disabled).await?;

    let user_account_id = account.user_account_id;
    let login_status = login_status_from(account.mfa_enabled);
    let session = create_session(user_account_id, login_status.clone(), &op)?;
    let session_id = store_session(user_account_id, session, &store).await?;

    update_last_login_if_necessary(
        user_account_id,
        login_time,
        login_status.clone(),
        email_addr,
        &op,
    )
    .await?;

    Ok((session_id, login_status))
}

async fn find_account_by_email_address(
    email_addr: &str,
    password: &str,
    op: &impl LoginOperation,
) -> Result<Account, ErrResp> {
    let accounts = op.filter_account_by_email_addr(email_addr).await?;
    let num = accounts.len();
    if num > 1 {
        error!("multiple email addresses: {}", email_addr);
        return Err(unexpected_err_resp());
    }
    let account = accounts.get(0).cloned().ok_or_else(|| {
        // 処理時間からアカウントが見つからなかったのか、パスワードが見つからなかったのかを露見させないようにするため
        // アカウントが見つからなかった場合でもis_password_matchの処理を行い、パスワードが一致しなかった場合の計算量に近づける
        // 計算量を同等にするためだけにis_password_matchを呼ぶので、戻り値は意図的に無視する
        let _ = is_password_match(password, DUMMY_HASHED_PASSWORD);
        error!("unauthorized: no email address ({}) found", email_addr);
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: EmailOrPwdIncorrect as u32,
            }),
        )
    })?;
    Ok(account)
}

fn verify_password(
    email_addr: &str,
    password: &str,
    hashed_password: &[u8],
) -> Result<(), ErrResp> {
    let matched = is_password_match(password, hashed_password).map_err(|e| {
        error!("failed to handle password: {}", e);
        unexpected_err_resp()
    })?;
    if !matched {
        error!(
            "unauthorized: password not match (email address: {})",
            email_addr
        );
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: EmailOrPwdIncorrect as u32,
            }),
        ));
    }
    Ok(())
}

async fn ensure_account_is_not_disabled(
    account_id: i64,
    email_addr: &str,
    disabled: bool,
) -> Result<(), ErrResp> {
    if disabled {
        error!(
            "disabled account (account id: {}, email address: {})",
            account_id, email_addr
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: AccountDisabled as u32,
            }),
        ));
    };
    Ok(())
}

fn login_status_from(mfa_enabled: bool) -> LoginStatus {
    // 二段階認証が有効化されている場合、ユーザー名とパスワードだけでは認証は終わらないため、NeedMoreVerificationとなる。
    // 逆に無効化されている場合、ユーザー名とパスワードだけでは認証は終わるのでFinishとなる。
    if mfa_enabled {
        LoginStatus::NeedMoreVerification
    } else {
        LoginStatus::Finish
    }
}

fn create_session(
    user_account_id: i64,
    login_status: LoginStatus,
    op: &impl LoginOperation,
) -> Result<Session, ErrResp> {
    let mut session = Session::new();

    session
        .insert(KEY_TO_USER_ACCOUNT_ID, user_account_id)
        .map_err(|e| {
            error!(
                "failed to insert account id ({}) into session: {}",
                user_account_id, e
            );
            unexpected_err_resp()
        })?;

    let ls = String::from(login_status);
    session
        .insert(KEY_TO_LOGIN_STATUS, ls.clone())
        .map_err(|e| {
            error!("failed to insert login_status ({}) into session: {}", ls, e);
            unexpected_err_resp()
        })?;

    op.set_login_session_expiry(&mut session);
    Ok(session)
}

async fn store_session(
    user_account_id: i64,
    session: Session,
    store: &impl SessionStore,
) -> Result<String, ErrResp> {
    let option = store.store_session(session).await.map_err(|e| {
        error!(
            "failed to store session for account id ({}): {}",
            user_account_id, e
        );
        unexpected_err_resp()
    })?;
    let session_id = match option {
        Some(s) => s,
        None => {
            error!(
                "failed to get cookie name for user account id ({})",
                user_account_id
            );
            return Err(unexpected_err_resp());
        }
    };
    Ok(session_id)
}

async fn update_last_login_if_necessary(
    user_account_id: i64,
    login_time: &DateTime<FixedOffset>,
    login_status: LoginStatus,
    email_addr: &str,
    op: &impl LoginOperation,
) -> Result<(), ErrResp> {
    match login_status {
        LoginStatus::Finish => {
            op.update_last_login(user_account_id, login_time).await?;
            info!(
                "{} (account id: {}) logged-in at {}",
                email_addr, user_account_id, login_time
            );
        }
        LoginStatus::NeedMoreVerification => {
            info!(
                "{} (account id: {}) tried login at {} (MFA is enabled, so authentication has not been done yet)",
                email_addr, user_account_id, login_time
            );
        }
    };
    Ok(())
}

#[async_trait]
trait LoginOperation {
    async fn filter_account_by_email_addr(&self, email_addr: &str)
        -> Result<Vec<Account>, ErrResp>;
    fn set_login_session_expiry(&self, session: &mut Session);
    async fn update_last_login(
        &self,
        account_id: i64,
        login_time: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

#[derive(Clone, Debug)]
struct Account {
    user_account_id: i64,
    hashed_password: Vec<u8>,
    disabled: bool,
    mfa_enabled: bool,
}

struct LoginOperationImpl {
    pool: DatabaseConnection,
    expiry: Duration,
}

impl LoginOperationImpl {
    fn new(pool: DatabaseConnection, expiry: Duration) -> Self {
        Self { pool, expiry }
    }
}

#[async_trait]
impl LoginOperation for LoginOperationImpl {
    async fn filter_account_by_email_addr(
        &self,
        email_addr: &str,
    ) -> Result<Vec<Account>, ErrResp> {
        let account_models = UserAccount::find()
            .filter(user_account::Column::EmailAddress.eq(email_addr))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter user_account (email_address: {}): {}",
                    email_addr, e
                );
                unexpected_err_resp()
            })?;
        Ok(account_models
            .iter()
            .map(|model| Account {
                user_account_id: model.user_account_id,
                hashed_password: model.hashed_password.clone(),
                disabled: model.disabled_at.is_some(),
                mfa_enabled: model.mfa_enabled_at.is_some(),
            })
            .collect::<Vec<Account>>())
    }

    fn set_login_session_expiry(&self, session: &mut Session) {
        session.expire_in(self.expiry);
    }

    async fn update_last_login(
        &self,
        account_id: i64,
        login_time: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        update_last_login(account_id, login_time, &self.pool).await
    }
}

#[cfg(test)]
mod tests {

    use async_session::MemoryStore;
    use chrono::TimeZone;
    use common::{
        password::hash_password,
        util::validator::{
            email_address_validator::validate_email_address, password_validator::validate_password,
        },
    };

    use super::*;

    struct LoginOperationMock<'a> {
        account: Account,
        email_addr: &'a str,
        login_time: &'a DateTime<FixedOffset>,
    }

    impl<'a> LoginOperationMock<'a> {
        fn new(
            account: Account,
            email_addr: &'a str,
            login_time: &'a DateTime<FixedOffset>,
        ) -> Self {
            Self {
                account,
                email_addr,
                login_time,
            }
        }
    }

    #[async_trait]
    impl<'a> LoginOperation for LoginOperationMock<'a> {
        async fn filter_account_by_email_addr(
            &self,
            email_addr: &str,
        ) -> Result<Vec<Account>, ErrResp> {
            if self.email_addr == email_addr {
                Ok(vec![self.account.clone()])
            } else {
                Ok(vec![])
            }
        }

        fn set_login_session_expiry(&self, _session: &mut Session) {
            // テスト実行中に有効期限が過ぎるケースを考慮し、有効期限は設定しない
        }

        async fn update_last_login(
            &self,
            account_id: i64,
            login_time: &DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            if self.account.mfa_enabled {
                panic!("mfa_enabled is true, but update_last_login is called");
            }
            assert_eq!(self.account.user_account_id, account_id);
            assert_eq!(self.login_time, login_time);
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_login_req_success() {
        let id = 1102;
        let email_addr = "test@example.com";
        let pwd = "1234567890abcdABCD";
        validate_email_address(email_addr).expect("failed to get Ok");
        validate_password(pwd).expect("failed to get Ok");
        let hashed_pwd = hash_password(pwd).expect("failed to hash pwd");
        let creation_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 9, 11, 15, 30, 45)
            .unwrap();
        let last_login = creation_time + chrono::Duration::days(1);
        let account = Account {
            user_account_id: id,
            hashed_password: hashed_pwd,
            disabled: false,
            mfa_enabled: false,
        };
        let store = MemoryStore::new();
        let current_date_time = last_login + chrono::Duration::days(1);
        let op = LoginOperationMock::new(account, email_addr, &current_date_time);

        let result = handle_login_req(email_addr, pwd, &current_date_time, op, store.clone()).await;

        let session_id_and_login_status = result.expect("failed to get Ok");
        let session_id = session_id_and_login_status.0;
        let login_status = session_id_and_login_status.1;

        let session = store
            .load_session(session_id)
            .await
            .expect("failed to get Ok")
            .expect("failed to get value");

        let actual_id = session
            .get::<i64>(KEY_TO_USER_ACCOUNT_ID)
            .expect("failed to get value");
        assert_eq!(id, actual_id);

        assert_eq!(login_status.clone(), LoginStatus::Finish);
        let actual_login_status = session
            .get::<String>(KEY_TO_LOGIN_STATUS)
            .expect("failed to get value");
        assert_eq!(String::from(login_status), actual_login_status);
    }

    #[tokio::test]
    async fn handle_login_req_success_mfa_enabled() {
        let id = 1102;
        let email_addr = "test@example.com";
        let pwd = "1234567890abcdABCD";
        validate_email_address(email_addr).expect("failed to get Ok");
        validate_password(pwd).expect("failed to get Ok");
        let hashed_pwd = hash_password(pwd).expect("failed to hash pwd");
        let creation_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 9, 11, 15, 30, 45)
            .unwrap();
        let last_login = creation_time + chrono::Duration::days(1);
        let account = Account {
            user_account_id: id,
            hashed_password: hashed_pwd,
            disabled: false,
            mfa_enabled: true,
        };
        let store = MemoryStore::new();
        let current_date_time = last_login + chrono::Duration::days(1);
        let op = LoginOperationMock::new(account, email_addr, &current_date_time);

        let result = handle_login_req(email_addr, pwd, &current_date_time, op, store.clone()).await;

        let session_id_and_login_status = result.expect("failed to get Ok");
        let session_id = session_id_and_login_status.0;
        let login_status = session_id_and_login_status.1;

        let session = store
            .load_session(session_id)
            .await
            .expect("failed to get Ok")
            .expect("failed to get value");

        let actual_id = session
            .get::<i64>(KEY_TO_USER_ACCOUNT_ID)
            .expect("failed to get value");
        assert_eq!(id, actual_id);

        assert_eq!(login_status.clone(), LoginStatus::NeedMoreVerification);
        let actual_login_status = session
            .get::<String>(KEY_TO_LOGIN_STATUS)
            .expect("failed to get value");
        assert_eq!(String::from(login_status), actual_login_status);
    }

    #[tokio::test]
    async fn handle_login_req_fail_no_email_addr_found() {
        let id = 1102;
        let email_addr1 = "test1@example.com";
        let email_addr2 = "test2@example.com";
        let pwd = "1234567890abcdABCD";
        validate_email_address(email_addr1).expect("failed to get Ok");
        validate_email_address(email_addr2).expect("failed to get Ok");
        validate_password(pwd).expect("failed to get Ok");
        let hashed_pwd = hash_password(pwd).expect("failed to hash pwd");
        let creation_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 9, 11, 15, 30, 45)
            .unwrap();
        let last_login = creation_time + chrono::Duration::days(1);
        let account = Account {
            user_account_id: id,
            hashed_password: hashed_pwd,
            disabled: false,
            mfa_enabled: false,
        };
        let store = MemoryStore::new();
        let current_date_time = last_login + chrono::Duration::days(1);
        let op = LoginOperationMock::new(account, email_addr1, &current_date_time);

        let result =
            handle_login_req(email_addr2, pwd, &current_date_time, op, store.clone()).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::UNAUTHORIZED, resp.0);
        assert_eq!(EmailOrPwdIncorrect as u32, resp.1.code);
        assert_eq!(0, store.count().await);
    }

    #[tokio::test]
    async fn handle_login_req_fail_incorrect_password() {
        let id = 1102;
        let email_addr = "test1@example.com";
        let pwd1 = "1234567890abcdABCD";
        let pwd2 = "bbbbbbbbbC";
        validate_email_address(email_addr).expect("failed to get Ok");
        validate_password(pwd1).expect("failed to get Ok");
        validate_password(pwd2).expect("failed to get Ok");
        let hashed_pwd = hash_password(pwd1).expect("failed to hash pwd");
        let creation_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 9, 11, 15, 30, 45)
            .unwrap();
        let last_login = creation_time + chrono::Duration::days(1);
        let account = Account {
            user_account_id: id,
            hashed_password: hashed_pwd,
            disabled: false,
            mfa_enabled: false,
        };
        let store = MemoryStore::new();
        let current_date_time = last_login + chrono::Duration::days(1);
        let op = LoginOperationMock::new(account, email_addr, &current_date_time);

        let result =
            handle_login_req(email_addr, pwd2, &current_date_time, op, store.clone()).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::UNAUTHORIZED, resp.0);
        assert_eq!(EmailOrPwdIncorrect as u32, resp.1.code);
        assert_eq!(0, store.count().await);
    }

    #[tokio::test]
    async fn handle_login_req_fail_account_disabled() {
        let id = 1102;
        let email_addr = "test1@example.com";
        let pwd = "1234567890abcdABCD";
        validate_email_address(email_addr).expect("failed to get Ok");
        validate_password(pwd).expect("failed to get Ok");
        let hashed_pwd = hash_password(pwd).expect("failed to hash pwd");
        let creation_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 9, 11, 15, 30, 45)
            .unwrap();
        let last_login = creation_time + chrono::Duration::days(1);
        let account = Account {
            user_account_id: id,
            hashed_password: hashed_pwd,
            disabled: true,
            mfa_enabled: false,
        };
        let store = MemoryStore::new();
        let current_date_time = last_login + chrono::Duration::days(1);
        let op = LoginOperationMock::new(account, email_addr, &current_date_time);

        let result = handle_login_req(email_addr, pwd, &current_date_time, op, store.clone()).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(AccountDisabled as u32, resp.1.code);
        assert_eq!(0, store.count().await);
    }
}
