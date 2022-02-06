// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::Json;
use chrono::Duration;
use chrono::{DateTime, Utc};
use common::model::user::Account;
use common::model::user::NewPassword;
use common::schema::ccs_schema::new_password::dsl::new_password;
use common::schema::ccs_schema::user_account::dsl::email_address;
use common::schema::ccs_schema::user_account::{hashed_password, table as user_account_table};
use common::smtp::{
    SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SOCKET_FOR_SMTP_SERVER, SYSTEM_EMAIL_ADDRESS,
};
use common::util::validator::validate_uuid;
use common::VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE;
use common::{ApiError, DatabaseConnection, ErrResp, RespResult};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::result::Error::NotFound;
use diesel::{update, RunQueryDsl};
use diesel::{ExpressionMethods, PgConnection, QueryDsl};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use tower_cookies::Cookies;

use crate::err::Code::{InvalidUuid, NewPasswordExpired, NoAccountFound, NoNewPasswordFound};
use crate::util::session::SESSION_ID_COOKIE_NAME;
use crate::util::{unexpected_err_resp, WEB_SITE_NAME};

static SUBJECT: Lazy<String> = Lazy::new(|| format!("[{}] パスワード変更完了通知", WEB_SITE_NAME));

/// 新しいパスワードに変更する<br>
/// セッションが存在する場合、新しいパスワードに変更する前にセッションを破棄する<br>
/// <br>
/// # Errors
/// アカウントがない場合、ステータスコード400、エラーコード[NO_ACCOUNT_FOUND]を返す<br>
/// UUIDが不正な形式の場合、ステータスコード400、エラーコード[INVALID_UUID]を返す<br>
/// 新しいパスワードが見つからない場合、ステータスコード400、エラーコード[NO_NEW_PASSWORD_FOUND]を返す<br>
/// 新しいパスワードが期限切れの場合、ステータスコード400、エラーコード[NEW_PASSWORD_EXPIRED]を返す<br>
pub(crate) async fn post_password_change(
    cookies: Cookies,
    Extension(store): Extension<RedisSessionStore>,
    Json(new_pwd): Json<NewPasswordId>,
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<PasswordChangeResult> {
    let option_cookie = cookies.get(SESSION_ID_COOKIE_NAME);
    if let Some(session_id) = option_cookie {
        let _ = destroy_session_if_exists(session_id.value(), &store).await?;
    }

    let current_date_time = chrono::Utc::now();
    let op = PasswordChangeOperationImpl::new(conn);
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
    post_password_change_internal(
        &new_pwd.new_password_id,
        &current_date_time,
        op,
        smtp_client,
    )
    .await
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct PasswordChangeResult {}

async fn post_password_change_internal(
    new_password_id: &str,
    current_date_time: &DateTime<Utc>,
    op: impl PasswordChangeOperation,
    send_mail: impl SendMail,
) -> RespResult<PasswordChangeResult> {
    let _ = validate_uuid(new_password_id).map_err(|e| {
        tracing::error!("failed to validate uuid: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: InvalidUuid as u32,
            }),
        )
    })?;
    let email_addr = async move {
        let new_pwd = op.find_new_password_by_id(new_password_id)?;
        let duration = *current_date_time - new_pwd.created_at;
        if duration > Duration::minutes(VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE) {
            tracing::error!(
                "new password (created at {}) already expired at {}",
                &new_pwd.created_at,
                current_date_time
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: NewPasswordExpired as u32,
                }),
            ));
        }
        let account = find_account_by_email_address(&new_pwd.email_address, &op)?;
        let _ = op.update_password(account.user_account_id, &new_pwd.hashed_password)?;
        tracing::info!(
            "{} changed password at {}",
            new_pwd.email_address,
            current_date_time
        );
        Ok(new_pwd.email_address)
    }
    .await?;
    let text = create_text();
    let _ =
        async { send_mail.send_mail(&email_addr, SYSTEM_EMAIL_ADDRESS, &SUBJECT, &text) }.await?;
    Ok((StatusCode::OK, Json(PasswordChangeResult {})))
}

async fn destroy_session_if_exists(
    session_id_value: &str,
    store: &impl SessionStore,
) -> Result<(), ErrResp> {
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
            tracing::debug!("no session in session store on password change");
            return Ok(());
        }
    };
    let _ = store.destroy_session(session).await.map_err(|e| {
        tracing::error!(
            "failed to destroy session (session_id: {}): {}",
            session_id_value,
            e
        );
        unexpected_err_resp()
    })?;
    Ok(())
}

fn find_account_by_email_address(
    email_addr: &str,
    op: &impl PasswordChangeOperation,
) -> Result<Account, ErrResp> {
    let accounts = op.filter_account_by_email_address(email_addr)?;
    let cnt = accounts.len();
    if cnt == 0 {
        tracing::error!(
            "failed to change password: user account ({}) does not exist",
            email_addr
        );
        Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: NoAccountFound as u32,
            }),
        ))
    } else if cnt == 1 {
        Ok(accounts[0].clone())
    } else {
        tracing::error!(
            "failed to change password: found multiple accounts: {}",
            email_addr
        );
        Err(unexpected_err_resp())
    }
}

#[derive(Deserialize)]
pub(crate) struct NewPasswordId {
    #[serde(rename = "new-password-id")]
    new_password_id: String,
}

fn create_text() -> String {
    // TODO: 文面の調整
    format!(
        r"パスワード変更が完了致しました。

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        INQUIRY_EMAIL_ADDRESS
    )
}

trait PasswordChangeOperation {
    // DBの分離レベルにはREAD COMITTEDを想定。
    // その想定の上でトランザクションが必要かどうかを検討し、操作を分離して実装
    fn find_new_password_by_id(&self, new_password_id: &str) -> Result<NewPassword, ErrResp>;
    fn filter_account_by_email_address(&self, email_addr: &str) -> Result<Vec<Account>, ErrResp>;
    fn update_password(&self, id: i32, hashed_pwd: &[u8]) -> Result<(), ErrResp>;
}

struct PasswordChangeOperationImpl {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl PasswordChangeOperationImpl {
    fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { conn }
    }
}

impl PasswordChangeOperation for PasswordChangeOperationImpl {
    fn find_new_password_by_id(&self, new_password_id: &str) -> Result<NewPassword, ErrResp> {
        let result = new_password
            .find(new_password_id)
            .first::<NewPassword>(&self.conn);
        match result {
            Ok(new_pwd) => Ok(new_pwd),
            Err(e) => {
                if e == NotFound {
                    Err((
                        StatusCode::BAD_REQUEST,
                        Json(ApiError {
                            code: NoNewPasswordFound as u32,
                        }),
                    ))
                } else {
                    Err(unexpected_err_resp())
                }
            }
        }
    }

    fn filter_account_by_email_address(&self, email_addr: &str) -> Result<Vec<Account>, ErrResp> {
        let result = user_account_table
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

    fn update_password(&self, id: i32, hashed_pwd: &[u8]) -> Result<(), ErrResp> {
        let _ = update(user_account_table.find(id))
            .set(hashed_password.eq(hashed_pwd))
            .execute(&self.conn)
            .map_err(|e| {
                tracing::error!("failed to update password on id ({}): {}", id, e);
                unexpected_err_resp()
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use async_session::MemoryStore;
    use axum::http::StatusCode;
    use axum::Json;
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use common::{
        model::user::{Account, NewPassword},
        smtp::SYSTEM_EMAIL_ADDRESS,
        util::{
            hash_password, is_password_match,
            validator::{validate_email_address, validate_password},
        },
        ApiError, ErrResp, VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE,
    };
    use uuid::Uuid;

    use crate::{
        err::Code::{InvalidUuid, NewPasswordExpired, NoAccountFound, NoNewPasswordFound},
        password_change::{
            create_text, post_password_change_internal, PasswordChangeResult, SUBJECT,
        },
        util::{session::tests::prepare_session, tests::SendMailMock},
    };

    use super::{destroy_session_if_exists, PasswordChangeOperation};

    struct PasswordChangeOperationMock {
        no_new_password_found: bool,
        new_password: NewPassword,
        no_account_found: bool,
        user_account_id: i32,
        email_address: String,
        old_pwd: String,
        last_login_time: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
        new_pwd_equals_old_one: bool,
    }

    impl PasswordChangeOperationMock {
        fn new(
            no_new_password_found: bool,
            new_password: NewPassword,
            no_account_found: bool,
            user_account_id: i32,
            email_address: String,
            old_pwd: String,
            last_login_time: Option<DateTime<Utc>>,
            created_at: DateTime<Utc>,
            new_pwd_equals_old_one: bool,
        ) -> Self {
            Self {
                no_new_password_found,
                new_password,
                no_account_found,
                user_account_id,
                email_address,
                old_pwd,
                last_login_time,
                created_at,
                new_pwd_equals_old_one,
            }
        }
    }

    impl PasswordChangeOperation for PasswordChangeOperationMock {
        fn find_new_password_by_id(&self, new_password_id: &str) -> Result<NewPassword, ErrResp> {
            if self.no_new_password_found {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: NoNewPasswordFound as u32,
                    }),
                ));
            }
            assert_eq!(self.new_password.new_password_id, new_password_id);
            return Ok(self.new_password.clone());
        }

        fn filter_account_by_email_address(
            &self,
            email_addr: &str,
        ) -> Result<Vec<Account>, ErrResp> {
            if self.no_account_found {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: NoAccountFound as u32,
                    }),
                ));
            }
            assert_eq!(self.email_address, email_addr);
            let _ = validate_password(&self.old_pwd).expect("failed to get Ok");
            let hashed_pwd = hash_password(&self.old_pwd).expect("failed to get Ok");
            let account = Account {
                user_account_id: self.user_account_id,
                email_address: self.email_address.to_string(),
                hashed_password: hashed_pwd,
                last_login_time: self.last_login_time,
                created_at: self.created_at,
            };
            Ok(vec![account])
        }

        fn update_password(&self, id: i32, hashed_pwd: &[u8]) -> Result<(), ErrResp> {
            assert_eq!(self.user_account_id, id);
            assert_eq!(self.new_password.hashed_password, hashed_pwd);
            if self.new_pwd_equals_old_one {
                assert!(is_password_match(&self.old_pwd, hashed_pwd).expect("failed to get Ok"));
            } else {
                assert!(!is_password_match(&self.old_pwd, hashed_pwd).expect("failed to get Ok"));
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn password_change_success() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        let _ = validate_password(new_pwd).expect("failed to get Ok");
        let hashed_new_pwd = hash_password(new_pwd).expect("failed to hash password");
        let new_pwd_created_at = chrono::Utc.ymd(2021, 11, 14).and_hms(21, 22, 40);
        let new_password = NewPassword {
            new_password_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_new_pwd,
            created_at: new_pwd_created_at,
        };
        let old_pwd = "aaaaaaaaaB";
        let _ = validate_password(old_pwd).expect("failed to get Ok");
        let op_mock = PasswordChangeOperationMock::new(
            false,
            new_password,
            false,
            52354,
            email_addr.to_string(),
            old_pwd.to_string(),
            Some(new_pwd_created_at - Duration::days(1)),
            new_pwd_created_at - Duration::days(2),
            new_pwd == old_pwd,
        );
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );
        let current_date_time =
            new_pwd_created_at + Duration::minutes(VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE);

        let result =
            post_password_change_internal(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(PasswordChangeResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn password_change_success_update_to_same_password() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        let _ = validate_password(new_pwd).expect("failed to get Ok");
        let hashed_new_pwd = hash_password(new_pwd).expect("failed to hash password");
        let new_pwd_created_at = chrono::Utc.ymd(2021, 11, 14).and_hms(21, 22, 40);
        let new_password = NewPassword {
            new_password_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_new_pwd,
            created_at: new_pwd_created_at,
        };
        let old_pwd = new_pwd; // update to same password
        let _ = validate_password(old_pwd).expect("failed to get Ok");
        let op_mock = PasswordChangeOperationMock::new(
            false,
            new_password,
            false,
            52354,
            email_addr.to_string(),
            old_pwd.to_string(),
            Some(new_pwd_created_at - Duration::days(1)),
            new_pwd_created_at - Duration::days(2),
            new_pwd == old_pwd,
        );
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );
        let current_date_time =
            new_pwd_created_at + Duration::minutes(VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE);

        let result =
            post_password_change_internal(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(PasswordChangeResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn password_change_success_case_where_user_has_not_logged_in_yet() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        let _ = validate_password(new_pwd).expect("failed to get Ok");
        let hashed_new_pwd = hash_password(new_pwd).expect("failed to hash password");
        let new_pwd_created_at = chrono::Utc.ymd(2021, 11, 14).and_hms(21, 22, 40);
        let new_password = NewPassword {
            new_password_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_new_pwd,
            created_at: new_pwd_created_at,
        };
        let old_pwd = "aaaaaaaaaB";
        let _ = validate_password(old_pwd).expect("failed to get Ok");
        let op_mock = PasswordChangeOperationMock::new(
            false,
            new_password,
            false,
            52354,
            email_addr.to_string(),
            old_pwd.to_string(),
            None,
            new_pwd_created_at - Duration::days(2),
            new_pwd == old_pwd,
        );
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );
        let current_date_time =
            new_pwd_created_at + Duration::minutes(VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE);

        let result =
            post_password_change_internal(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(PasswordChangeResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn password_change_fail_invalid_uuid() {
        let uuid = "0123456789abcABC".to_string();
        let email_addr = "test@test.com";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        let _ = validate_password(new_pwd).expect("failed to get Ok");
        let hashed_new_pwd = hash_password(new_pwd).expect("failed to hash password");
        let new_pwd_created_at = chrono::Utc.ymd(2021, 11, 14).and_hms(21, 22, 40);
        let new_password = NewPassword {
            new_password_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_new_pwd,
            created_at: new_pwd_created_at,
        };
        let old_pwd = "aaaaaaaaaB";
        let _ = validate_password(old_pwd).expect("failed to get Ok");
        let op_mock = PasswordChangeOperationMock::new(
            false,
            new_password,
            false,
            52354,
            email_addr.to_string(),
            old_pwd.to_string(),
            Some(new_pwd_created_at - Duration::days(1)),
            new_pwd_created_at - Duration::days(2),
            new_pwd == old_pwd,
        );
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );
        let current_date_time =
            new_pwd_created_at + Duration::minutes(VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE);

        let result =
            post_password_change_internal(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(InvalidUuid as u32, resp.1.code);
    }

    #[tokio::test]
    async fn password_change_fail_no_account_found() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        let _ = validate_password(new_pwd).expect("failed to get Ok");
        let hashed_new_pwd = hash_password(new_pwd).expect("failed to hash password");
        let new_pwd_created_at = chrono::Utc.ymd(2021, 11, 14).and_hms(21, 22, 40);
        let new_password = NewPassword {
            new_password_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_new_pwd,
            created_at: new_pwd_created_at,
        };
        let old_pwd = "aaaaaaaaaB";
        let _ = validate_password(old_pwd).expect("failed to get Ok");
        let op_mock = PasswordChangeOperationMock::new(
            false,
            new_password,
            true,
            52354,
            email_addr.to_string(),
            old_pwd.to_string(),
            Some(new_pwd_created_at - Duration::days(1)),
            new_pwd_created_at - Duration::days(2),
            new_pwd == old_pwd,
        );
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );
        let current_date_time =
            new_pwd_created_at + Duration::minutes(VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE);

        let result =
            post_password_change_internal(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(NoAccountFound as u32, resp.1.code);
    }

    #[tokio::test]
    async fn password_change_fail_no_new_password_found() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        let _ = validate_password(new_pwd).expect("failed to get Ok");
        let hashed_new_pwd = hash_password(new_pwd).expect("failed to hash password");
        let new_pwd_created_at = chrono::Utc.ymd(2021, 11, 14).and_hms(21, 22, 40);
        let new_password = NewPassword {
            new_password_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_new_pwd,
            created_at: new_pwd_created_at,
        };
        let old_pwd = "aaaaaaaaaB";
        let _ = validate_password(old_pwd).expect("failed to get Ok");
        let op_mock = PasswordChangeOperationMock::new(
            true,
            new_password,
            false,
            52354,
            email_addr.to_string(),
            old_pwd.to_string(),
            Some(new_pwd_created_at - Duration::days(1)),
            new_pwd_created_at - Duration::days(2),
            new_pwd == old_pwd,
        );
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );
        let current_date_time =
            new_pwd_created_at + Duration::minutes(VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE);

        let result =
            post_password_change_internal(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(NoNewPasswordFound as u32, resp.1.code);
    }

    #[tokio::test]
    async fn password_change_fail_new_password_expired() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        let _ = validate_password(new_pwd).expect("failed to get Ok");
        let hashed_new_pwd = hash_password(new_pwd).expect("failed to hash password");
        let new_pwd_created_at = chrono::Utc.ymd(2021, 11, 14).and_hms(21, 22, 40);
        let new_password = NewPassword {
            new_password_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_new_pwd,
            created_at: new_pwd_created_at,
        };
        let old_pwd = "aaaaaaaaaB";
        let _ = validate_password(old_pwd).expect("failed to get Ok");
        let op_mock = PasswordChangeOperationMock::new(
            false,
            new_password,
            false,
            52354,
            email_addr.to_string(),
            old_pwd.to_string(),
            Some(new_pwd_created_at - Duration::days(1)),
            new_pwd_created_at - Duration::days(2),
            new_pwd == old_pwd,
        );
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );
        let current_date_time = new_pwd_created_at
            + Duration::minutes(VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE)
            + Duration::milliseconds(1);

        let result =
            post_password_change_internal(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(NewPasswordExpired as u32, resp.1.code);
    }

    #[tokio::test]
    async fn destroy_session_if_exists_destorys_session() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id_value = prepare_session(user_account_id, &store).await;
        assert_eq!(1, store.count().await);

        let _ = destroy_session_if_exists(&session_id_value, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
    }

    #[tokio::test]
    async fn destroy_session_if_exists_returns_ok_if_no_session_exists() {
        let store = MemoryStore::new();
        // dummy session id
        let session_id_value = "KBvGQJJVyQquK5yuEcwlbfJfjNHBMAXIKRnHbVO/0QzBMHLak1xmqhaTbDuscJSeEPL2qwZfTP5BalDDMmR8eA==";
        assert_eq!(0, store.count().await);

        let _ = destroy_session_if_exists(&session_id_value, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
    }
}
