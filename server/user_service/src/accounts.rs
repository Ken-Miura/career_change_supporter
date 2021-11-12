// Copyright 2021 Ken Miura

use axum::http::StatusCode;
use axum::Json;
use chrono::Duration;
use chrono::{DateTime, Utc};
use common::model::user::NewAccount;
use common::model::user::TempAccount;
use common::schema::ccs_schema::user_account::table as user_account_table;
use common::schema::ccs_schema::user_temp_account::dsl::user_temp_account;
use common::smtp::{
    SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SOCKET_FOR_SMTP_SERVER, SYSTEM_EMAIL_ADDRESS,
};
use common::util::validator::validate_uuid;
use common::VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR;
use common::{ApiError, DatabaseConnection, ErrResp, RespResult};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::result::Error::NotFound;
use diesel::{insert_into, RunQueryDsl};
use diesel::{PgConnection, QueryDsl};
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;

use crate::err_code::{
    ACCOUNT_ALREADY_EXISTS, INVALID_UUID, NO_TEMP_ACCOUNT_FOUND, TEMP_ACCOUNT_EXPIRED,
};
use crate::util::{self, unexpected_err_resp, WEB_SITE_NAME};

static SUBJECT: Lazy<String> = Lazy::new(|| format!("[{}] 新規登録完了通知", WEB_SITE_NAME));

/// アカウントを作成する<br>
/// <br>
/// # Errors
/// すでにアカウントがある場合、ステータスコード400、エラーコード[ACCOUNT_ALREADY_EXISTS]を返す<br>
/// UUIDが不正な形式の場合、ステータスコード400、エラーコード[INVALID_UUID]を返す<br>
/// 一時アカウントが見つからない場合、ステータスコード400、エラーコード[NO_TEMP_ACCOUNT_FOUND]を返す<br>
/// 一時アカウントが期限切れの場合、ステータスコード400、エラーコード[TEMP_ACCOUNT_EXPIRED]を返す<br>
pub(crate) async fn post_accounts(
    Json(temp_account): Json<TempAccountId>,
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<AccountsResult> {
    let current_date_time = chrono::Utc::now();
    let op = AccountsOperationImpl::new(conn);
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
    post_accounts_internal(
        &temp_account.temp_account_id,
        &current_date_time,
        op,
        smtp_client,
    )
    .await
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct AccountsResult {}

async fn post_accounts_internal(
    temp_account_id: &str,
    current_date_time: &DateTime<Utc>,
    op: impl AccountsOperation,
    send_mail: impl SendMail,
) -> RespResult<AccountsResult> {
    let _ = validate_uuid(temp_account_id).map_err(|e| {
        tracing::error!("failed to validate uuid: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError { code: INVALID_UUID }),
        )
    })?;
    let email_addr = async move {
        let temp_account = op.find_temp_account_by_id(temp_account_id)?;
        let duration = *current_date_time - temp_account.created_at;
        if duration > Duration::hours(VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR) {
            tracing::error!(
                "temp account (created at {}) already expired at {}",
                &temp_account.created_at,
                current_date_time
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: TEMP_ACCOUNT_EXPIRED,
                }),
            ));
        }
        let exists = op.user_exists(&temp_account.email_address)?;
        if exists {
            tracing::error!(
                "failed to create account: user account ({}) already exists",
                &temp_account.email_address
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: ACCOUNT_ALREADY_EXISTS,
                }),
            ));
        }
        let account = NewAccount {
            email_address: &temp_account.email_address,
            hashed_password: &temp_account.hashed_password,
            last_login_time: None,
            created_at: current_date_time,
        };
        let _ = op.create_account(&account)?;
        Ok(temp_account.email_address)
    }
    .await?;
    let text = create_text();
    let _ =
        async { send_mail.send_mail(&email_addr, SYSTEM_EMAIL_ADDRESS, &SUBJECT, &text) }.await?;
    Ok((StatusCode::OK, Json(AccountsResult {})))
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

trait AccountsOperation {
    // DBの分離レベルにはREAD COMITTEDを想定。
    // その想定の上でトランザクションが必要かどうかを検討し、操作を分離して実装
    fn find_temp_account_by_id(&self, temp_account_id: &str) -> Result<TempAccount, ErrResp>;
    fn user_exists(&self, email_addr: &str) -> Result<bool, ErrResp>;
    fn create_account(&self, account: &NewAccount) -> Result<(), ErrResp>;
}

struct AccountsOperationImpl {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl AccountsOperationImpl {
    fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { conn }
    }
}

impl AccountsOperation for AccountsOperationImpl {
    fn find_temp_account_by_id(&self, temp_account_id: &str) -> Result<TempAccount, ErrResp> {
        let result = user_temp_account
            .find(temp_account_id)
            .first::<TempAccount>(&self.conn);
        match result {
            Ok(temp_account) => Ok(temp_account),
            Err(e) => {
                if e == NotFound {
                    Err((
                        StatusCode::BAD_REQUEST,
                        Json(ApiError {
                            code: NO_TEMP_ACCOUNT_FOUND,
                        }),
                    ))
                } else {
                    Err(unexpected_err_resp())
                }
            }
        }
    }

    fn user_exists(&self, email_addr: &str) -> Result<bool, ErrResp> {
        util::db_operation::user_exists(&self.conn, email_addr)
    }

    fn create_account(&self, account: &NewAccount) -> Result<(), ErrResp> {
        let _ = insert_into(user_account_table)
            .values(account)
            .execute(&self.conn)
            .map_err(|e| {
                tracing::error!(
                    "failed to insert user account ({}): {}",
                    account.email_address,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone};
    use common::util::{
        hash_password,
        validator::{validate_email_address, validate_password},
    };
    use uuid::Uuid;

    use crate::util::tests::SendMailMock;

    use super::*;

    struct AccountsOperationMock<'a> {
        temp_account: &'a TempAccount,
        no_temp_account_found: bool,
        exists: bool,
        current_date_time: &'a DateTime<Utc>,
    }

    impl<'a> AccountsOperationMock<'a> {
        fn new(
            temp_account: &'a TempAccount,
            no_temp_account_found: bool,
            exists: bool,
            current_date_time: &'a DateTime<Utc>,
        ) -> Self {
            Self {
                temp_account,
                no_temp_account_found,
                exists,
                current_date_time,
            }
        }
    }

    impl<'a> AccountsOperation for AccountsOperationMock<'a> {
        fn find_temp_account_by_id(&self, temp_account_id: &str) -> Result<TempAccount, ErrResp> {
            assert_eq!(&self.temp_account.user_temp_account_id, temp_account_id);
            if self.no_temp_account_found {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: NO_TEMP_ACCOUNT_FOUND,
                    }),
                ));
            }
            Ok(self.temp_account.clone())
        }

        fn user_exists(&self, email_addr: &str) -> Result<bool, ErrResp> {
            assert_eq!(&self.temp_account.email_address, email_addr);
            Ok(self.exists)
        }

        fn create_account(&self, account: &NewAccount) -> Result<(), ErrResp> {
            assert_eq!(&self.temp_account.email_address, account.email_address);
            assert_eq!(&self.temp_account.hashed_password, account.hashed_password);
            assert_eq!(None, account.last_login_time);
            assert_eq!(&self.current_date_time, &account.created_at);
            Ok(())
        }
    }

    #[tokio::test]
    async fn accounts_success() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let hashed_pwd = hash_password("aaaaaaaaaA").expect("failed to hash password");
        let register_date_time = chrono::Utc.ymd(2021, 9, 5).and_hms(21, 00, 40);
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

        let result =
            post_accounts_internal(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(AccountsResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn accounts_fail_invalid_uuid() {
        let uuid = "0123456789abcABC".to_string();
        let email_addr = "test@test.com";
        let hashed_pwd = hash_password("aaaaaaaaaA").expect("failed to hash password");
        let register_date_time = chrono::Utc.ymd(2021, 9, 5).and_hms(21, 00, 40);
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

        let result =
            post_accounts_internal(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(INVALID_UUID, resp.1.code);
    }

    #[tokio::test]
    async fn accounts_fail_temp_account_expired() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let pwd = "aaaaaaaaaA";
        let _ = validate_uuid(&uuid).expect("failed to get Ok");
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let _ = validate_password(pwd).expect("failed to get Ok");
        let hashed_pwd = hash_password(pwd).expect("failed to hash password");
        let register_date_time = chrono::Utc.ymd(2021, 9, 5).and_hms(21, 00, 40);
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

        let result =
            post_accounts_internal(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(TEMP_ACCOUNT_EXPIRED, resp.1.code);
    }

    #[tokio::test]
    async fn accounts_fail_no_temp_account_found() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let pwd = "aaaaaaaaaA";
        let _ = validate_uuid(&uuid).expect("failed to get Ok");
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let _ = validate_password(pwd).expect("failed to get Ok");
        let hashed_pwd = hash_password(pwd).expect("failed to hash password");
        let register_date_time = chrono::Utc.ymd(2021, 9, 5).and_hms(21, 00, 40);
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

        let result =
            post_accounts_internal(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(NO_TEMP_ACCOUNT_FOUND, resp.1.code);
    }

    #[tokio::test]
    async fn accounts_fail_account_exists() {
        let uuid = Uuid::new_v4().to_simple().to_string();
        let email_addr = "test@test.com";
        let pwd = "aaaaaaaaaA";
        let _ = validate_uuid(&uuid).expect("failed to get Ok");
        let _ = validate_email_address(email_addr).expect("failed to get Ok");
        let _ = validate_password(pwd).expect("failed to get Ok");
        let hashed_pwd = hash_password(pwd).expect("failed to hash password");
        let register_date_time = chrono::Utc.ymd(2021, 9, 5).and_hms(21, 00, 40);
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

        let result =
            post_accounts_internal(&uuid, &current_date_time, op_mock, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(ACCOUNT_ALREADY_EXISTS, resp.1.code);
    }
}
