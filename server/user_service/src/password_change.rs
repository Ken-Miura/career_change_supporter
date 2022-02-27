// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::async_trait;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::Json;
use chrono::DateTime;
use chrono::{Duration, FixedOffset};
use common::smtp::{
    SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SOCKET_FOR_SMTP_SERVER, SYSTEM_EMAIL_ADDRESS,
};
use common::util::validator::validate_uuid;
use common::VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE;
use common::{ApiError, ErrResp, RespResult};
use entity::prelude::{NewPassword, UserAccount};
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use entity::{new_password, user_account};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use tower_cookies::Cookies;

use crate::err::unexpected_err_resp;
use crate::err::Code::{InvalidUuid, NewPasswordExpired, NoAccountFound, NoNewPasswordFound};
use crate::util::session::SESSION_ID_COOKIE_NAME;
use crate::util::{JAPANESE_TIME_ZONE, WEB_SITE_NAME};

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
    Json(new_pwd): Json<NewPasswordReqId>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<PasswordChangeResult> {
    let option_cookie = cookies.get(SESSION_ID_COOKIE_NAME);
    if let Some(session_id) = option_cookie {
        let _ = destroy_session_if_exists(session_id.value(), &store).await?;
    }

    let current_date_time = chrono::Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
    let op = PasswordChangeOperationImpl::new(pool);
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
    handle_password_change_req(
        &new_pwd.new_password_id,
        &current_date_time,
        op,
        smtp_client,
    )
    .await
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct PasswordChangeResult {}

async fn handle_password_change_req(
    new_password_id: &str,
    current_date_time: &DateTime<FixedOffset>,
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
    let new_pwd_option = op.find_new_password_by_id(new_password_id).await?;
    let new_pwd = new_pwd_option.ok_or_else(|| {
        tracing::error!("no id ({}) found", new_password_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: NoNewPasswordFound as u32,
            }),
        )
    })?;
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
    let account_ids = op
        .filter_account_id_by_email_address(&new_pwd.email_address)
        .await?;
    let cnt = account_ids.len();
    if cnt > 1 {
        tracing::error!(
            "found multiple accounts (email address: {})",
            &new_pwd.email_address
        );
        return Err(unexpected_err_resp());
    }
    let account_id = account_ids.get(0).cloned().ok_or_else(|| {
        tracing::error!(
            "user account (email address: {}) does not exist",
            &new_pwd.email_address
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: NoAccountFound as u32,
            }),
        )
    })?;
    let _ = op
        .update_password(account_id, &new_pwd.hashed_password)
        .await?;
    tracing::info!(
        "{} changed password at {}",
        new_pwd.email_address,
        current_date_time
    );

    let text = create_text();
    let _ = async {
        send_mail.send_mail(
            &new_pwd.email_address,
            SYSTEM_EMAIL_ADDRESS,
            &SUBJECT,
            &text,
        )
    }
    .await?;
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

#[derive(Deserialize)]
pub(crate) struct NewPasswordReqId {
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

#[async_trait]
trait PasswordChangeOperation {
    // DBの分離レベルにはREAD COMITTEDを想定。
    // その想定の上でトランザクションが必要かどうかを検討し、操作を分離して実装
    async fn find_new_password_by_id(
        &self,
        new_password_id: &str,
    ) -> Result<Option<NewPasswordReq>, ErrResp>;
    async fn filter_account_id_by_email_address(
        &self,
        email_addr: &str,
    ) -> Result<Vec<i32>, ErrResp>;
    async fn update_password(&self, account_id: i32, hashed_pwd: &[u8]) -> Result<(), ErrResp>;
}

#[derive(Clone, Debug)]
struct NewPasswordReq {
    email_address: String,
    hashed_password: Vec<u8>,
    created_at: DateTime<FixedOffset>,
}

struct PasswordChangeOperationImpl {
    pool: DatabaseConnection,
}

impl PasswordChangeOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PasswordChangeOperation for PasswordChangeOperationImpl {
    async fn find_new_password_by_id(
        &self,
        new_password_id: &str,
    ) -> Result<Option<NewPasswordReq>, ErrResp> {
        let model = NewPassword::find_by_id(new_password_id.to_string())
            .one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "failed to find new password (new password id: {}): {}",
                    new_password_id,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| NewPasswordReq {
            email_address: m.email_address,
            hashed_password: m.hashed_password,
            created_at: m.created_at,
        }))
    }

    async fn filter_account_id_by_email_address(
        &self,
        email_addr: &str,
    ) -> Result<Vec<i32>, ErrResp> {
        let models = UserAccount::find()
            .filter(user_account::Column::EmailAddress.eq(email_addr))
            .all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "failed to filter account id (email address: {}): {}",
                    email_addr,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| m.user_account_id)
            .collect::<Vec<i32>>())
    }

    async fn update_password(&self, account_id: i32, hashed_pwd: &[u8]) -> Result<(), ErrResp> {
        let model = user_account::ActiveModel {
            user_account_id: Set(account_id),
            hashed_password: Set(hashed_pwd.to_vec()),
            ..Default::default()
        };
        let _ = model.update(&self.pool).await.map_err(|e| {
            tracing::error!(
                "failed to update password (account id: {}): {}",
                account_id,
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
    use axum::async_trait;
    use axum::http::StatusCode;
    use axum::Json;
    use chrono::{DateTime, Duration, TimeZone, Utc};
    use common::{
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
        password_change::{create_text, handle_password_change_req, PasswordChangeResult, SUBJECT},
        util::{session::tests::prepare_session, tests::SendMailMock},
    };

    use super::{destroy_session_if_exists, Account, NewPasswordReq, PasswordChangeOperation};

    struct PasswordChangeOperationMock {
        asserted_params: AssertedParams,
        test_case_params: TestCaseParams,
    }

    struct AssertedParams {
        new_password: NewPasswordReq,
        user_account_id: i32,
        email_address: String,
        old_pwd: String,
        last_login_time: Option<DateTime<Utc>>,
        created_at: DateTime<Utc>,
    }

    struct TestCaseParams {
        no_new_password_found: bool,
        no_account_found: bool,
        new_pwd_equals_old_one: bool,
    }

    impl PasswordChangeOperationMock {
        fn new(asserted_params: AssertedParams, test_case_params: TestCaseParams) -> Self {
            Self {
                asserted_params,
                test_case_params,
            }
        }
    }

    #[async_trait]
    impl PasswordChangeOperation for PasswordChangeOperationMock {
        async fn find_new_password_by_id(
            &self,
            new_password_id: &str,
        ) -> Result<Option<NewPasswordReq>, ErrResp> {
            if self.test_case_params.no_new_password_found {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: NoNewPasswordFound as u32,
                    }),
                ));
            }
            assert_eq!(
                self.asserted_params.new_password.new_password_id,
                new_password_id
            );
            Ok(self.asserted_params.new_password.clone())
        }

        async fn filter_account_id_by_email_address(
            &self,
            email_addr: &str,
        ) -> Result<Vec<i32>, ErrResp> {
            if self.test_case_params.no_account_found {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: NoAccountFound as u32,
                    }),
                ));
            }
            assert_eq!(self.asserted_params.email_address, email_addr);
            let _ = validate_password(&self.asserted_params.old_pwd).expect("failed to get Ok");
            let hashed_pwd =
                hash_password(&self.asserted_params.old_pwd).expect("failed to get Ok");
            let account = Account {
                user_account_id: self.asserted_params.user_account_id,
                email_address: self.asserted_params.email_address.to_string(),
                hashed_password: hashed_pwd,
                last_login_time: self.asserted_params.last_login_time,
                created_at: self.asserted_params.created_at,
            };
            Ok(vec![account])
        }

        async fn update_password(&self, account_id: i32, hashed_pwd: &[u8]) -> Result<(), ErrResp> {
            assert_eq!(self.asserted_params.user_account_id, id);
            assert_eq!(
                self.asserted_params.new_password.hashed_password,
                hashed_pwd
            );
            if self.test_case_params.new_pwd_equals_old_one {
                assert!(is_password_match(&self.asserted_params.old_pwd, hashed_pwd)
                    .expect("failed to get Ok"));
            } else {
                assert!(
                    !is_password_match(&self.asserted_params.old_pwd, hashed_pwd)
                        .expect("failed to get Ok")
                );
            }
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_password_change_req_success() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        let _ = validate_password(new_pwd).expect("failed to get Ok");
        let hashed_new_pwd = hash_password(new_pwd).expect("failed to hash password");
        let new_pwd_created_at = chrono::Utc.ymd(2021, 11, 14).and_hms(21, 22, 40);
        let new_password = NewPasswordReq {
            new_password_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_new_pwd,
            created_at: new_pwd_created_at,
        };
        let old_pwd = "aaaaaaaaaB";
        let _ = validate_password(old_pwd).expect("failed to get Ok");
        let op_mock = PasswordChangeOperationMock::new(
            AssertedParams {
                new_password,
                user_account_id: 52354,
                email_address: email_addr.to_string(),
                old_pwd: old_pwd.to_string(),
                last_login_time: Some(new_pwd_created_at - Duration::days(1)),
                created_at: new_pwd_created_at - Duration::days(2),
            },
            TestCaseParams {
                no_new_password_found: false,
                no_account_found: false,
                new_pwd_equals_old_one: new_pwd == old_pwd,
            },
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
            handle_password_change_req(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(PasswordChangeResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_password_change_req_success_update_to_same_password() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        let _ = validate_password(new_pwd).expect("failed to get Ok");
        let hashed_new_pwd = hash_password(new_pwd).expect("failed to hash password");
        let new_pwd_created_at = chrono::Utc.ymd(2021, 11, 14).and_hms(21, 22, 40);
        let new_password = NewPasswordReq {
            new_password_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_new_pwd,
            created_at: new_pwd_created_at,
        };
        let old_pwd = new_pwd; // update to same password
        let _ = validate_password(old_pwd).expect("failed to get Ok");
        let op_mock = PasswordChangeOperationMock::new(
            AssertedParams {
                new_password,
                user_account_id: 52354,
                email_address: email_addr.to_string(),
                old_pwd: old_pwd.to_string(),
                last_login_time: Some(new_pwd_created_at - Duration::days(1)),
                created_at: new_pwd_created_at - Duration::days(2),
            },
            TestCaseParams {
                no_new_password_found: false,
                no_account_found: false,
                new_pwd_equals_old_one: new_pwd == old_pwd,
            },
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
            handle_password_change_req(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(PasswordChangeResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_password_change_req_success_case_where_user_has_not_logged_in_yet() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        let _ = validate_password(new_pwd).expect("failed to get Ok");
        let hashed_new_pwd = hash_password(new_pwd).expect("failed to hash password");
        let new_pwd_created_at = chrono::Utc.ymd(2021, 11, 14).and_hms(21, 22, 40);
        let new_password = NewPasswordReq {
            new_password_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_new_pwd,
            created_at: new_pwd_created_at,
        };
        let old_pwd = "aaaaaaaaaB";
        let _ = validate_password(old_pwd).expect("failed to get Ok");
        let op_mock = PasswordChangeOperationMock::new(
            AssertedParams {
                new_password,
                user_account_id: 52354,
                email_address: email_addr.to_string(),
                old_pwd: old_pwd.to_string(),
                last_login_time: None,
                created_at: new_pwd_created_at - Duration::days(2),
            },
            TestCaseParams {
                no_new_password_found: false,
                no_account_found: false,
                new_pwd_equals_old_one: new_pwd == old_pwd,
            },
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
            handle_password_change_req(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(PasswordChangeResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_password_change_req_fail_invalid_uuid() {
        let uuid = "0123456789abcABC".to_string();
        let email_addr = "test@test.com";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        let _ = validate_password(new_pwd).expect("failed to get Ok");
        let hashed_new_pwd = hash_password(new_pwd).expect("failed to hash password");
        let new_pwd_created_at = chrono::Utc.ymd(2021, 11, 14).and_hms(21, 22, 40);
        let new_password = NewPasswordReq {
            new_password_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_new_pwd,
            created_at: new_pwd_created_at,
        };
        let old_pwd = "aaaaaaaaaB";
        let _ = validate_password(old_pwd).expect("failed to get Ok");
        let op_mock = PasswordChangeOperationMock::new(
            AssertedParams {
                new_password,
                user_account_id: 52354,
                email_address: email_addr.to_string(),
                old_pwd: old_pwd.to_string(),
                last_login_time: Some(new_pwd_created_at - Duration::days(1)),
                created_at: new_pwd_created_at - Duration::days(2),
            },
            TestCaseParams {
                no_new_password_found: false,
                no_account_found: false,
                new_pwd_equals_old_one: new_pwd == old_pwd,
            },
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
            handle_password_change_req(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(InvalidUuid as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_password_change_req_fail_no_account_found() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        let _ = validate_password(new_pwd).expect("failed to get Ok");
        let hashed_new_pwd = hash_password(new_pwd).expect("failed to hash password");
        let new_pwd_created_at = chrono::Utc.ymd(2021, 11, 14).and_hms(21, 22, 40);
        let new_password = NewPasswordReq {
            new_password_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_new_pwd,
            created_at: new_pwd_created_at,
        };
        let old_pwd = "aaaaaaaaaB";
        let _ = validate_password(old_pwd).expect("failed to get Ok");
        let op_mock = PasswordChangeOperationMock::new(
            AssertedParams {
                new_password,
                user_account_id: 52354,
                email_address: email_addr.to_string(),
                old_pwd: old_pwd.to_string(),
                last_login_time: Some(new_pwd_created_at - Duration::days(1)),
                created_at: new_pwd_created_at - Duration::days(2),
            },
            TestCaseParams {
                no_new_password_found: false,
                no_account_found: true,
                new_pwd_equals_old_one: new_pwd == old_pwd,
            },
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
            handle_password_change_req(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(NoAccountFound as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_password_change_req_fail_no_new_password_found() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        let _ = validate_password(new_pwd).expect("failed to get Ok");
        let hashed_new_pwd = hash_password(new_pwd).expect("failed to hash password");
        let new_pwd_created_at = chrono::Utc.ymd(2021, 11, 14).and_hms(21, 22, 40);
        let new_password = NewPasswordReq {
            new_password_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_new_pwd,
            created_at: new_pwd_created_at,
        };
        let old_pwd = "aaaaaaaaaB";
        let _ = validate_password(old_pwd).expect("failed to get Ok");
        let op_mock = PasswordChangeOperationMock::new(
            AssertedParams {
                new_password,
                user_account_id: 52354,
                email_address: email_addr.to_string(),
                old_pwd: old_pwd.to_string(),
                last_login_time: Some(new_pwd_created_at - Duration::days(1)),
                created_at: new_pwd_created_at - Duration::days(2),
            },
            TestCaseParams {
                no_new_password_found: true,
                no_account_found: false,
                new_pwd_equals_old_one: new_pwd == old_pwd,
            },
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
            handle_password_change_req(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(NoNewPasswordReqFound as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_password_change_req_fail_new_password_expired() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        let _ = validate_password(new_pwd).expect("failed to get Ok");
        let hashed_new_pwd = hash_password(new_pwd).expect("failed to hash password");
        let new_pwd_created_at = chrono::Utc.ymd(2021, 11, 14).and_hms(21, 22, 40);
        let new_password = NewPasswordReq {
            new_password_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_new_pwd,
            created_at: new_pwd_created_at,
        };
        let old_pwd = "aaaaaaaaaB";
        let _ = validate_password(old_pwd).expect("failed to get Ok");
        let op_mock = PasswordChangeOperationMock::new(
            AssertedParams {
                new_password,
                user_account_id: 52354,
                email_address: email_addr.to_string(),
                old_pwd: old_pwd.to_string(),
                last_login_time: Some(new_pwd_created_at - Duration::days(1)),
                created_at: new_pwd_created_at - Duration::days(2),
            },
            TestCaseParams {
                no_new_password_found: false,
                no_account_found: false,
                new_pwd_equals_old_one: new_pwd == old_pwd,
            },
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
            handle_password_change_req(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(NewPasswordReqExpired as u32, resp.1.code);
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

        let _ = destroy_session_if_exists(session_id_value, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
    }
}
