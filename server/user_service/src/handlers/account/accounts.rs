// Copyright 2021 Ken Miura

use axum::async_trait;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::DateTime;
use chrono::{Duration, FixedOffset};
use common::smtp::{
    SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SMTP_HOST, SMTP_PASSWORD, SMTP_PORT,
    SMTP_USERNAME, SYSTEM_EMAIL_ADDRESS,
};
use common::util::validator::uuid_validator::validate_uuid;
use common::{ApiError, ErrResp, RespResult, WEB_SITE_NAME};
use common::{JAPANESE_TIME_ZONE, VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR};
use entity::prelude::{UserAccount, UserTempAccount};
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use entity::user_account;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use tracing::{error, info};

use super::TempAccount;
use crate::err::unexpected_err_resp;
use crate::err::Code::{AccountAlreadyExists, NoTempAccountFound, TempAccountExpired};

static SUBJECT: Lazy<String> = Lazy::new(|| format!("[{}] 新規登録完了通知", WEB_SITE_NAME));

/// アカウントを作成する<br>
/// <br>
/// # Errors
/// UUIDが不正な形式の場合、ステータスコード400、エラーコード[common::err::Code::InvalidUuidFormat]を返す<br>
/// すでにアカウントがある場合、ステータスコード400、エラーコード[AccountAlreadyExists]を返す<br>
/// 一時アカウントが見つからない場合、ステータスコード400、エラーコード[NoTempAccountFound]を返す<br>
/// 一時アカウントが期限切れの場合、ステータスコード400、エラーコード[TempAccountExpired]を返す<br>
pub(crate) async fn post_accounts(
    State(pool): State<DatabaseConnection>,
    Json(temp_account): Json<TempAccountId>,
) -> RespResult<AccountsResult> {
    let current_date_time = chrono::Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = AccountsOperationImpl::new(pool);
    let smtp_client = SmtpClient::new(
        SMTP_HOST.to_string(),
        *SMTP_PORT,
        SMTP_USERNAME.to_string(),
        SMTP_PASSWORD.to_string(),
    );
    handle_accounts_req(
        &temp_account.temp_account_id,
        &current_date_time,
        op,
        smtp_client,
    )
    .await
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct AccountsResult {}

async fn handle_accounts_req(
    temp_account_id: &str,
    current_date_time: &DateTime<FixedOffset>,
    op: impl AccountsOperation,
    send_mail: impl SendMail,
) -> RespResult<AccountsResult> {
    validate_uuid(temp_account_id).map_err(|e| {
        error!("failed to validate {}: {}", temp_account_id, e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: common::err::Code::InvalidUuidFormat as u32,
            }),
        )
    })?;
    let temp_account_option = op.find_temp_account_by_id(temp_account_id).await?;
    let temp_account = temp_account_option.ok_or_else(|| {
        error!("no temp account (id: {}) found", temp_account_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: NoTempAccountFound as u32,
            }),
        )
    })?;
    let duration = *current_date_time - temp_account.created_at;
    if duration > Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR) {
        error!(
            "temp account (created at {}) already expired at {}",
            &temp_account.created_at, current_date_time
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: TempAccountExpired as u32,
            }),
        ));
    }
    let exists = op.user_exists(&temp_account.email_address).await?;
    if exists {
        error!(
            "failed to create account: user account ({}) already exists",
            &temp_account.email_address
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: AccountAlreadyExists as u32,
            }),
        ));
    }
    let account = NewAccount {
        email_address: temp_account.email_address.clone(),
        hashed_password: temp_account.hashed_password,
        last_login_time: None,
        created_at: *current_date_time,
        disabled_at: None,
    };
    op.create_account(&account).await?;
    info!(
        "accout ({}) was created at {}",
        temp_account.email_address, current_date_time
    );
    let text = create_text();
    send_mail
        .send_mail(
            &temp_account.email_address,
            SYSTEM_EMAIL_ADDRESS,
            &SUBJECT,
            &text,
        )
        .await?;
    Ok((StatusCode::OK, Json(AccountsResult {})))
}

#[derive(Clone, Debug)]
struct NewAccount {
    email_address: String,
    hashed_password: Vec<u8>,
    last_login_time: Option<DateTime<FixedOffset>>,
    created_at: DateTime<FixedOffset>,
    disabled_at: Option<DateTime<FixedOffset>>,
}

#[derive(Deserialize)]
pub(crate) struct TempAccountId {
    #[serde(rename = "temp-account-id")]
    temp_account_id: String,
}

fn create_text() -> String {
    // TODO: 文面の調整
    format!(
        r"新規登録が完了いたしました。このたびは{}へのご登録ありがとうございます。

他のユーザーに相談を申し込むには、ご本人確認が必要となります。引き続き、ログイン後、プロフィールよりご本人確認の申請をお願いいたします。

他のユーザーから相談を受けるには、ご本人確認に加え、下記の三点の登録が必要となります。他のユーザーからの相談を受けたい場合、本人確認完了後、下記の三点をプロフィールよりご登録いただくようお願いします。
・職歴
・相談料
・銀行口座

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        WEB_SITE_NAME, INQUIRY_EMAIL_ADDRESS
    )
}

#[async_trait]
trait AccountsOperation {
    // DBの分離レベルにはREAD COMITTEDを想定。
    // その想定の上でトランザクションが必要かどうかを検討し、操作を分離して実装
    async fn find_temp_account_by_id(
        &self,
        temp_account_id: &str,
    ) -> Result<Option<TempAccount>, ErrResp>;
    async fn user_exists(&self, email_addr: &str) -> Result<bool, ErrResp>;
    async fn create_account(&self, account: &NewAccount) -> Result<(), ErrResp>;
}

struct AccountsOperationImpl {
    pool: DatabaseConnection,
}

impl AccountsOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AccountsOperation for AccountsOperationImpl {
    async fn find_temp_account_by_id(
        &self,
        temp_account_id: &str,
    ) -> Result<Option<TempAccount>, ErrResp> {
        let temp_account_model = UserTempAccount::find_by_id(temp_account_id.to_string())
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find user_temp_account (temp_account_id: {}): {}",
                    temp_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(temp_account_model.map(|model| TempAccount {
            user_temp_account_id: model.user_temp_account_id,
            email_address: model.email_address,
            hashed_password: model.hashed_password,
            created_at: model.created_at,
        }))
    }

    async fn user_exists(&self, email_addr: &str) -> Result<bool, ErrResp> {
        let models = UserAccount::find()
            .filter(user_account::Column::EmailAddress.eq(email_addr))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find user_account (email_address: {}): {}",
                    email_addr, e
                );
                unexpected_err_resp()
            })?;
        Ok(!models.is_empty())
    }

    async fn create_account(&self, account: &NewAccount) -> Result<(), ErrResp> {
        let user_account_model = user_account::ActiveModel {
            email_address: Set(account.email_address.clone()),
            hashed_password: Set(account.hashed_password.clone()),
            last_login_time: Set(account.last_login_time),
            created_at: Set(account.created_at),
            disabled_at: Set(account.disabled_at),
            ..Default::default()
        };
        let _ = user_account_model.insert(&self.pool).await.map_err(|e| {
            error!(
                "failed to insert user_account (email_address: {}): {}",
                account.email_address, e
            );
            unexpected_err_resp()
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::{DateTime, Duration, FixedOffset, TimeZone};
    use common::{
        password::hash_password,
        smtp::SYSTEM_EMAIL_ADDRESS,
        util::validator::{
            email_address_validator::validate_email_address, password_validator::validate_password,
            uuid_validator::validate_uuid,
        },
        ErrResp, JAPANESE_TIME_ZONE, VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR,
    };
    use uuid::Uuid;

    use crate::{
        err::Code::{AccountAlreadyExists, NoTempAccountFound, TempAccountExpired},
        handlers::account::TempAccount,
    };
    use crate::{
        handlers::account::accounts::{create_text, handle_accounts_req, AccountsResult, SUBJECT},
        util::tests::SendMailMock,
    };

    use super::{AccountsOperation, NewAccount};

    struct AccountsOperationMock<'a> {
        temp_account: &'a TempAccount,
        no_temp_account_found: bool,
        exists: bool,
        current_date_time: &'a DateTime<FixedOffset>,
    }

    impl<'a> AccountsOperationMock<'a> {
        fn new(
            temp_account: &'a TempAccount,
            no_temp_account_found: bool,
            exists: bool,
            current_date_time: &'a DateTime<FixedOffset>,
        ) -> Self {
            Self {
                temp_account,
                no_temp_account_found,
                exists,
                current_date_time,
            }
        }
    }

    #[async_trait]
    impl<'a> AccountsOperation for AccountsOperationMock<'a> {
        async fn find_temp_account_by_id(
            &self,
            temp_account_id: &str,
        ) -> Result<Option<TempAccount>, ErrResp> {
            assert_eq!(&self.temp_account.user_temp_account_id, temp_account_id);
            if self.no_temp_account_found {
                return Ok(None);
            }
            Ok(Some(self.temp_account.clone()))
        }

        async fn user_exists(&self, email_addr: &str) -> Result<bool, ErrResp> {
            assert_eq!(&self.temp_account.email_address, email_addr);
            Ok(self.exists)
        }

        async fn create_account(&self, account: &NewAccount) -> Result<(), ErrResp> {
            assert_eq!(&self.temp_account.email_address, &account.email_address);
            assert_eq!(&self.temp_account.hashed_password, &account.hashed_password);
            assert_eq!(None, account.last_login_time);
            assert_eq!(self.current_date_time, &account.created_at);
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_accounts_req_success() {
        let uuid = Uuid::new_v4().simple().to_string();
        let email_addr = "test@test.com";
        let hashed_pwd = hash_password("aaaaaaaaaA").expect("failed to hash password");
        let register_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 9, 5, 21, 00, 40)
            .unwrap();
        let temp_account = TempAccount {
            user_temp_account_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_pwd,
            created_at: register_date_time,
        };
        let current_date_time =
            register_date_time + Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR);
        let op_mock = AccountsOperationMock::new(&temp_account, false, false, &current_date_time);
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );

        let result = handle_accounts_req(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(AccountsResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_accounts_req_fail_temp_account_expired() {
        let uuid = Uuid::new_v4().simple().to_string();
        let email_addr = "test@test.com";
        let pwd = "aaaaaaaaaA";
        validate_uuid(&uuid).expect("failed to get Ok");
        validate_email_address(email_addr).expect("failed to get Ok");
        validate_password(pwd).expect("failed to get Ok");
        let hashed_pwd = hash_password(pwd).expect("failed to hash password");
        let register_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 9, 5, 21, 00, 40)
            .unwrap();
        let temp_account = TempAccount {
            user_temp_account_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_pwd,
            created_at: register_date_time,
        };
        let current_date_time = register_date_time
            + Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR)
            + Duration::milliseconds(1);
        let op_mock = AccountsOperationMock::new(&temp_account, false, false, &current_date_time);
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );

        let result = handle_accounts_req(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(TempAccountExpired as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_accounts_req_fail_no_temp_account_found() {
        let uuid = Uuid::new_v4().simple().to_string();
        let email_addr = "test@test.com";
        let pwd = "aaaaaaaaaA";
        validate_uuid(&uuid).expect("failed to get Ok");
        validate_email_address(email_addr).expect("failed to get Ok");
        validate_password(pwd).expect("failed to get Ok");
        let hashed_pwd = hash_password(pwd).expect("failed to hash password");
        let register_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 9, 5, 21, 00, 40)
            .unwrap();
        let temp_account = TempAccount {
            user_temp_account_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_pwd,
            created_at: register_date_time,
        };
        let current_date_time =
            register_date_time + Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR);
        let op_mock = AccountsOperationMock::new(&temp_account, true, false, &current_date_time);
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );

        let result = handle_accounts_req(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(NoTempAccountFound as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_accounts_req_fail_account_exists() {
        let uuid = Uuid::new_v4().simple().to_string();
        let email_addr = "test@test.com";
        let pwd = "aaaaaaaaaA";
        validate_uuid(&uuid).expect("failed to get Ok");
        validate_email_address(email_addr).expect("failed to get Ok");
        validate_password(pwd).expect("failed to get Ok");
        let hashed_pwd = hash_password(pwd).expect("failed to hash password");
        let register_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 9, 5, 21, 00, 40)
            .unwrap();
        let temp_account = TempAccount {
            user_temp_account_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_pwd,
            created_at: register_date_time,
        };
        let current_date_time =
            register_date_time + Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR);
        let op_mock = AccountsOperationMock::new(&temp_account, false, true, &current_date_time);
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );

        let result = handle_accounts_req(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(AccountAlreadyExists as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_accounts_req_fail_invalid_uuid() {
        let uuid = "1234abcdあいうえお<script>alert('test');</script>".to_string();
        let email_addr = "test@test.com";
        let pwd = "aaaaaaaaaA";
        validate_email_address(email_addr).expect("failed to get Ok");
        validate_password(pwd).expect("failed to get Ok");
        let hashed_pwd = hash_password(pwd).expect("failed to hash password");
        let register_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 9, 5, 21, 00, 40)
            .unwrap();
        let temp_account = TempAccount {
            user_temp_account_id: uuid.clone(),
            email_address: email_addr.to_string(),
            hashed_password: hashed_pwd,
            created_at: register_date_time,
        };
        let current_date_time =
            register_date_time + Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR);
        let op_mock = AccountsOperationMock::new(&temp_account, false, false, &current_date_time);
        let send_mail_mock = SendMailMock::new(
            email_addr.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );

        let result = handle_accounts_req(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(common::err::Code::InvalidUuidFormat as u32, resp.1.code);
    }
}
