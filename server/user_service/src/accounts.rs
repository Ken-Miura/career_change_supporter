// Copyright 2021 Ken Miura

use axum::http::StatusCode;
use axum::{extract::Query, Json};
use chrono::{DateTime, Utc};
use common::model::user::NewAccount;
use common::model::user::TempAccount;
use common::schema::ccs_schema::user_account::table as user_account_table;
use common::schema::ccs_schema::user_temp_account::dsl::user_temp_account;
use common::smtp::{
    SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SOCKET_FOR_SMTP_SERVER, SYSTEM_EMAIL_ADDRESS,
};
use common::util::validator::validate_uuid;
use common::{ApiError, DatabaseConnection, ErrResp, RespResult};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::result::Error::NotFound;
use diesel::{insert_into, RunQueryDsl};
use diesel::{PgConnection, QueryDsl};
use serde::Deserialize;
use serde::Serialize;

use crate::err_code::{
    ACCOUNT_ALREADY_EXISTS, INVALID_UUID, NO_TEMP_ACCOUNT_FOUND, TEMP_ACCOUNT_EXPIRED,
};
use crate::util::{self, unexpected_err_resp};

// TODO: 文面の調整
const SUBJECT: &str = "[就職転職に失敗しないためのサイト] ユーザー登録完了通知";

/// アカウントを作成する<br>
/// <br>
/// # Errors
pub(crate) async fn get_accounts(
    temp_account: Query<TempAccountId>,
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<AccountsResult> {
    let current_date_time = chrono::Utc::now();
    let op = AccountsOperationImpl::new(conn);
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
    get_accounts_internal(
        &temp_account.temp_account_id,
        &current_date_time,
        op,
        smtp_client,
    )
    .await
}

#[derive(Serialize, Debug)]
pub(crate) struct AccountsResult {}

async fn get_accounts_internal(
    temp_account_id: &str,
    current_date_time: &DateTime<Utc>,
    op: impl AccountsOperation,
    send_mail: impl SendMail,
) -> RespResult<AccountsResult> {
    let _ = validate_uuid(temp_account_id).map_err(|e| {
        tracing::error!("invalid uuid ({}): {}", temp_account_id, e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError { code: INVALID_UUID }),
        )
    })?;
    let email_addr = async move {
        let temp_account = op.find_temp_account_by_id(temp_account_id)?;
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
        let duration = *current_date_time - temp_account.created_at;
        if duration.num_days() > 0 {
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
        let account = NewAccount {
            email_address: &temp_account.email_address,
            hashed_password: &temp_account.hashed_password,
            last_login_time: None,
        };
        let _ = op.create_account(&account)?;
        Ok(temp_account.email_address)
    }
    .await?;
    let text = create_text();
    let _ =
        async { send_mail.send_mail(&email_addr, SYSTEM_EMAIL_ADDRESS, SUBJECT, &text) }.await?;
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
        r"ユーザー登録が完了いたしました。このたびは就職転職に失敗しないためのサイトへのご登録ありがとうございます。

アドバイザーに相談を申し込むには、ご本人確認が必要となります。引き続き、ログイン後、プロフィールよりご本人確認の申請をお願いいたします。

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        INQUIRY_EMAIL_ADDRESS
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
        util::user_exists(&self.conn, email_addr)
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
