// Copyright 2021 Ken Miura

use async_fred_session::RedisSessionStore;
use axum::async_trait;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use axum_extra::extract::SignedCookieJar;
use chrono::DateTime;
use chrono::{Duration, FixedOffset};
use common::password::hash_password;
use common::smtp::{SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS};
use common::util::validator::{
    password_validator::validate_password, uuid_validator::validate_uuid,
};
use common::{ApiError, ErrResp, RespResult, WEB_SITE_NAME};
use common::{JAPANESE_TIME_ZONE, VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE};
use entity::prelude::{PwdChangeReq, UserAccount};
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use entity::user_account;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use tracing::{error, info};

use crate::err::unexpected_err_resp;
use crate::err::Code::{NoAccountFound, NoPwdChnageReqFound, PwdChnageReqExpired};
use crate::handlers::session::{destroy_session_if_exists, SESSION_ID_COOKIE_NAME};

static SUBJECT: Lazy<String> = Lazy::new(|| format!("[{}] パスワード変更完了通知", WEB_SITE_NAME));

/// 新しいパスワードに更新する<br>
/// セッションが存在する場合、新しいパスワードに更新する前にセッションを破棄する<br>
/// <br>
/// # Note
/// - 古いパスワード、新しいパスワードが同一の場合でもパスワードは更新可能
/// - 一度もログインしたことがないアカウントでもパスワード更新は可能
/// <br>
/// # Errors
/// アカウントがない場合、ステータスコード400、エラーコード[NoAccountFound]を返す<br>
/// UUIDが不正な形式の場合、ステータスコード400、エラーコード[common::err::Code::InvalidUuidFormat]を返す<br>
/// パスワードが不正な形式の場合、ステータスコード400、エラーコード[common::err::Code::InvalidPasswordFormat]を返す<br>
/// パスワード変更要求が見つからない場合、ステータスコード400、エラーコード[NoPwdChnageReqFound]を返す<br>
/// パスワード変更要求が期限切れの場合、ステータスコード400、エラーコード[PwdChnageReqExpired]を返す<br>
pub(crate) async fn post_password_update(
    jar: SignedCookieJar,
    State(smtp_client): State<SmtpClient>,
    State(store): State<RedisSessionStore>,
    State(pool): State<DatabaseConnection>,
    Json(pwd_update_req): Json<PasswordUpdateReq>,
) -> RespResult<PasswordUpdateResult> {
    let option_cookie = jar.get(SESSION_ID_COOKIE_NAME);
    if let Some(session_id) = option_cookie {
        destroy_session_if_exists(session_id.value(), &store).await?;
    }

    let current_date_time = chrono::Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = PasswordUpdateOperationImpl::new(pool);
    handle_password_update_req(
        &pwd_update_req.pwd_change_req_id,
        &pwd_update_req.password,
        &current_date_time,
        op,
        smtp_client,
    )
    .await
}

#[derive(Deserialize)]
pub(crate) struct PasswordUpdateReq {
    #[serde(rename = "pwd-change-req-id")]
    pwd_change_req_id: String,
    password: String,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct PasswordUpdateResult {}

async fn handle_password_update_req(
    pwd_change_req_id: &str,
    password: &str,
    current_date_time: &DateTime<FixedOffset>,
    op: impl PasswordUpdateOperation,
    send_mail: impl SendMail,
) -> RespResult<PasswordUpdateResult> {
    validate_uuid(pwd_change_req_id).map_err(|e| {
        error!(
            "failed to validate pwd-change-req-id ({}): {}",
            pwd_change_req_id, e
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: common::err::Code::InvalidUuidFormat as u32,
            }),
        )
    })?;
    validate_password(password).map_err(|e| {
        error!("failed to validate password: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: common::err::Code::InvalidPasswordFormat as u32,
            }),
        )
    })?;
    let pwd_change_req_option = op.find_pwd_change_req_by_id(pwd_change_req_id).await?;
    let pwd_change_req = pwd_change_req_option.ok_or_else(|| {
        error!(
            "no password change request (request id: {}) found",
            pwd_change_req_id
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: NoPwdChnageReqFound as u32,
            }),
        )
    })?;
    let duration = *current_date_time - pwd_change_req.requested_at;
    if duration > Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE) {
        error!(
            "password change request (requested at {}) already expired at {}",
            &pwd_change_req.requested_at, current_date_time
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: PwdChnageReqExpired as u32,
            }),
        ));
    }
    let account_ids = op
        .filter_account_id_by_email_address(&pwd_change_req.email_address)
        .await?;
    let cnt = account_ids.len();
    if cnt > 1 {
        error!(
            "found multiple accounts (email address: {})",
            &pwd_change_req.email_address
        );
        return Err(unexpected_err_resp());
    }
    let account_id = account_ids.first().cloned().ok_or_else(|| {
        error!(
            "account (email address: {}) does not exist",
            &pwd_change_req.email_address
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: NoAccountFound as u32,
            }),
        )
    })?;
    let hashed_pwd = hash_password(password).map_err(|e| {
        error!("failed to handle password: {}", e);
        unexpected_err_resp()
    })?;
    op.update_password(account_id, &hashed_pwd).await?;
    info!(
        "{} updated password at {}",
        &pwd_change_req.email_address, current_date_time
    );

    let text = create_text();
    send_mail
        .send_mail(
            &pwd_change_req.email_address,
            SYSTEM_EMAIL_ADDRESS.as_str(),
            &SUBJECT,
            &text,
        )
        .await?;
    Ok((StatusCode::OK, Json(PasswordUpdateResult {})))
}

fn create_text() -> String {
    format!(
        r"パスワード変更が完了しました。

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        INQUIRY_EMAIL_ADDRESS.as_str()
    )
}

#[async_trait]
trait PasswordUpdateOperation {
    // DBの分離レベルにはREAD COMITTEDを想定。
    // その想定の上でトランザクションが必要かどうかを検討し、操作を分離して実装
    async fn find_pwd_change_req_by_id(
        &self,
        pwd_change_req_id: &str,
    ) -> Result<Option<PasswordChangeReq>, ErrResp>;
    async fn filter_account_id_by_email_address(
        &self,
        email_addr: &str,
    ) -> Result<Vec<i64>, ErrResp>;
    async fn update_password(&self, account_id: i64, hashed_pwd: &[u8]) -> Result<(), ErrResp>;
}

#[derive(Clone, Debug)]
struct PasswordChangeReq {
    email_address: String,
    requested_at: DateTime<FixedOffset>,
}

struct PasswordUpdateOperationImpl {
    pool: DatabaseConnection,
}

impl PasswordUpdateOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PasswordUpdateOperation for PasswordUpdateOperationImpl {
    async fn find_pwd_change_req_by_id(
        &self,
        pwd_change_req_id: &str,
    ) -> Result<Option<PasswordChangeReq>, ErrResp> {
        let model = PwdChangeReq::find_by_id(pwd_change_req_id.to_string())
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find pwd_change_req (pwd_change_req_id: {}): {}",
                    pwd_change_req_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| PasswordChangeReq {
            email_address: m.email_address,
            requested_at: m.requested_at,
        }))
    }

    async fn filter_account_id_by_email_address(
        &self,
        email_addr: &str,
    ) -> Result<Vec<i64>, ErrResp> {
        let models = UserAccount::find()
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
        Ok(models
            .into_iter()
            .map(|m| m.user_account_id)
            .collect::<Vec<i64>>())
    }

    async fn update_password(&self, account_id: i64, hashed_pwd: &[u8]) -> Result<(), ErrResp> {
        let model = user_account::ActiveModel {
            user_account_id: Set(account_id),
            hashed_password: Set(hashed_pwd.to_vec()),
            ..Default::default()
        };
        let _ = model.update(&self.pool).await.map_err(|e| {
            error!(
                "failed to update hashed_password in user_account (account id: {}): {}",
                account_id, e
            );
            unexpected_err_resp()
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;
    use common::{
        password::is_password_match,
        util::validator::email_address_validator::validate_email_address,
    };
    use uuid::Uuid;

    use crate::handlers::tests::SendMailMock;

    use super::*;

    struct PasswordUpdateOperationMock {
        account_id: i64,
        password_change_req: PasswordChangeReq,
        password_update_req: PasswordUpdateReq,
        test_case_params: TestCaseParams,
    }

    struct TestCaseParams {
        no_password_change_req_found: bool,
        no_account_found: bool,
    }

    impl PasswordUpdateOperationMock {
        fn new(
            account_id: i64,
            password_change_req: PasswordChangeReq,
            password_update_req: PasswordUpdateReq,
            test_case_params: TestCaseParams,
        ) -> Self {
            Self {
                account_id,
                password_change_req,
                password_update_req,
                test_case_params,
            }
        }
    }

    #[async_trait]
    impl PasswordUpdateOperation for PasswordUpdateOperationMock {
        async fn find_pwd_change_req_by_id(
            &self,
            pwd_change_req_id: &str,
        ) -> Result<Option<PasswordChangeReq>, ErrResp> {
            if self.test_case_params.no_password_change_req_found {
                return Ok(None);
            }
            assert_eq!(
                self.password_update_req.pwd_change_req_id,
                pwd_change_req_id
            );
            Ok(Some(self.password_change_req.clone()))
        }

        async fn filter_account_id_by_email_address(
            &self,
            email_addr: &str,
        ) -> Result<Vec<i64>, ErrResp> {
            if self.test_case_params.no_account_found {
                return Ok(vec![]);
            }
            assert_eq!(
                self.password_change_req.email_address,
                email_addr.to_string()
            );
            Ok(vec![self.account_id])
        }

        async fn update_password(&self, account_id: i64, hashed_pwd: &[u8]) -> Result<(), ErrResp> {
            assert_eq!(self.account_id, account_id);
            let result = is_password_match(&self.password_update_req.password, hashed_pwd)
                .expect("failed to get Ok");
            assert!(result);
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_password_update_req_success() {
        let email_addr = "test@test.com";
        validate_email_address(email_addr).expect("failed to get Ok");
        let pwd_change_requested_at = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 11, 14, 21, 22, 40)
            .unwrap();
        let password_change_req = PasswordChangeReq {
            email_address: email_addr.to_string(),
            requested_at: pwd_change_requested_at,
        };

        let uuid = Uuid::new_v4().simple().to_string();
        validate_uuid(&uuid).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        validate_password(new_pwd).expect("failed to get Ok");
        let password_update_req = PasswordUpdateReq {
            pwd_change_req_id: uuid.clone(),
            password: new_pwd.to_string(),
        };

        let op_mock = PasswordUpdateOperationMock::new(
            52354,
            password_change_req,
            password_update_req,
            TestCaseParams {
                no_password_change_req_found: false,
                no_account_found: false,
            },
        );
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );
        let current_date_time = pwd_change_requested_at
            + Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE);

        let result =
            handle_password_update_req(&uuid, new_pwd, &current_date_time, op_mock, send_mail_mock)
                .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(PasswordUpdateResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_password_update_req_fail_no_account_found() {
        let email_addr = "test@test.com";
        validate_email_address(email_addr).expect("failed to get Ok");
        let pwd_change_requested_at = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 11, 14, 21, 22, 40)
            .unwrap();
        let password_change_req = PasswordChangeReq {
            email_address: email_addr.to_string(),
            requested_at: pwd_change_requested_at,
        };

        let uuid = Uuid::new_v4().simple().to_string();
        validate_uuid(&uuid).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        validate_password(new_pwd).expect("failed to get Ok");
        let password_update_req = PasswordUpdateReq {
            pwd_change_req_id: uuid.clone(),
            password: new_pwd.to_string(),
        };

        let op_mock = PasswordUpdateOperationMock::new(
            52354,
            password_change_req,
            password_update_req,
            TestCaseParams {
                no_password_change_req_found: false,
                no_account_found: true,
            },
        );
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );
        let current_date_time = pwd_change_requested_at
            + Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE);

        let result =
            handle_password_update_req(&uuid, new_pwd, &current_date_time, op_mock, send_mail_mock)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(NoAccountFound as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_password_update_req_fail_no_password_change_req_found() {
        let email_addr = "test@test.com";
        validate_email_address(email_addr).expect("failed to get Ok");
        let pwd_change_requested_at = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 11, 14, 21, 22, 40)
            .unwrap();
        let password_change_req = PasswordChangeReq {
            email_address: email_addr.to_string(),
            requested_at: pwd_change_requested_at,
        };

        let uuid = Uuid::new_v4().simple().to_string();
        validate_uuid(&uuid).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        validate_password(new_pwd).expect("failed to get Ok");
        let password_update_req = PasswordUpdateReq {
            pwd_change_req_id: uuid.clone(),
            password: new_pwd.to_string(),
        };

        let op_mock = PasswordUpdateOperationMock::new(
            52354,
            password_change_req,
            password_update_req,
            TestCaseParams {
                no_password_change_req_found: true,
                no_account_found: false,
            },
        );
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );
        let current_date_time = pwd_change_requested_at
            + Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE);

        let result =
            handle_password_update_req(&uuid, new_pwd, &current_date_time, op_mock, send_mail_mock)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(NoPwdChnageReqFound as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_password_update_req_fail_password_change_req_expired() {
        let email_addr = "test@test.com";
        validate_email_address(email_addr).expect("failed to get Ok");
        let pwd_change_requested_at = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 11, 14, 21, 22, 40)
            .unwrap();
        let password_change_req = PasswordChangeReq {
            email_address: email_addr.to_string(),
            requested_at: pwd_change_requested_at,
        };

        let uuid = Uuid::new_v4().simple().to_string();
        validate_uuid(&uuid).expect("failed to get Ok");
        let new_pwd = "aaaaaaaaaA";
        validate_password(new_pwd).expect("failed to get Ok");
        let password_update_req = PasswordUpdateReq {
            pwd_change_req_id: uuid.clone(),
            password: new_pwd.to_string(),
        };

        let op_mock = PasswordUpdateOperationMock::new(
            52354,
            password_change_req,
            password_update_req,
            TestCaseParams {
                no_password_change_req_found: false,
                no_account_found: false,
            },
        );
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );
        let current_date_time = pwd_change_requested_at
            + Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE)
            + Duration::milliseconds(1);

        let result =
            handle_password_update_req(&uuid, new_pwd, &current_date_time, op_mock, send_mail_mock)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(PwdChnageReqExpired as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_password_update_req_fail_invalid_password() {
        let email_addr = "test@test.com";
        validate_email_address(email_addr).expect("failed to get Ok");
        let pwd_change_requested_at = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 11, 14, 21, 22, 40)
            .unwrap();
        let password_change_req = PasswordChangeReq {
            email_address: email_addr.to_string(),
            requested_at: pwd_change_requested_at,
        };

        let uuid = Uuid::new_v4().simple().to_string();
        validate_uuid(&uuid).expect("failed to get Ok");
        let invalid_pwd = "あいうえお";
        let password_update_req = PasswordUpdateReq {
            pwd_change_req_id: uuid.clone(),
            password: invalid_pwd.to_string(),
        };

        let op_mock = PasswordUpdateOperationMock::new(
            52354,
            password_change_req,
            password_update_req,
            TestCaseParams {
                no_password_change_req_found: false,
                no_account_found: false,
            },
        );
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );
        let current_date_time = pwd_change_requested_at
            + Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE);

        let result = handle_password_update_req(
            &uuid,
            invalid_pwd,
            &current_date_time,
            op_mock,
            send_mail_mock,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(common::err::Code::InvalidPasswordFormat as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_password_update_req_fail_invalid_uuid() {
        let email_addr = "test@test.com";
        validate_email_address(email_addr).expect("failed to get Ok");
        let pwd_change_requested_at = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 11, 14, 21, 22, 40)
            .unwrap();
        let password_change_req = PasswordChangeReq {
            email_address: email_addr.to_string(),
            requested_at: pwd_change_requested_at,
        };

        let uuid = "1' or '1' = '1';--".to_string();
        let new_pwd = "aaaaaaaaaA";
        validate_password(new_pwd).expect("failed to get Ok");
        let password_update_req = PasswordUpdateReq {
            pwd_change_req_id: uuid.clone(),
            password: new_pwd.to_string(),
        };

        let op_mock = PasswordUpdateOperationMock::new(
            52354,
            password_change_req,
            password_update_req,
            TestCaseParams {
                no_password_change_req_found: false,
                no_account_found: false,
            },
        );
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );
        let current_date_time = pwd_change_requested_at
            + Duration::minutes(VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE);

        let result =
            handle_password_update_req(&uuid, new_pwd, &current_date_time, op_mock, send_mail_mock)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(common::err::Code::InvalidUuidFormat as u32, resp.1.code);
    }
}
