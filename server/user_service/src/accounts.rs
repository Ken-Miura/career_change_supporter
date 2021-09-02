// Copyright 2021 Ken Miura

use axum::http::StatusCode;
use axum::{extract::Query, Json};
use chrono::{DateTime, Utc};
use common::model::user::NewAccount;
use common::model::user::TempAccount;
use common::smtp::{SendMail, SmtpClient, SOCKET_FOR_SMTP_SERVER};
use common::util::validator::validate_uuid;
use common::{DatabaseConnection, ErrResp, RespResult};
use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::PgConnection;
use serde::Deserialize;
use serde::Serialize;

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
    let _ = validate_uuid(temp_account_id).map_err(|e| {});
    let _ = async move {
        let temp_account = op.find_temp_account_by_id(temp_account_id).unwrap();
        // check if expired
        todo!();
        let exists = op.user_exists(&temp_account.email_address).unwrap();
        if exists {
            todo!();
        }
        let account = NewAccount {
            email_address: &temp_account.email_address,
            hashed_password: &temp_account.hashed_password,
            last_login_time: None,
        };
        op.create_account(&account)
    }
    .await;
    Ok((StatusCode::OK, Json(AccountsResult {})))
}

#[derive(Deserialize)]
pub(crate) struct TempAccountId {
    #[serde(rename = "temp-account-id")]
    temp_account_id: String,
}

trait AccountsOperation {
    // DBの分離レベルにはREAD COMITTEDを想定。
    // その想定の上でトランザクションが必要かどうかを検討し、操作を分離して実装
    fn find_temp_account_by_id(&self, temp_account_id: &str) -> Result<TempAccount, ErrResp>;
    fn user_exists(&self, email_addr: &str) -> Result<bool, ErrResp>;
    fn create_account(&self, temp_account: &NewAccount) -> Result<(), ErrResp>;
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
        todo!()
    }

    fn user_exists(&self, email_addr: &str) -> Result<bool, ErrResp> {
        todo!()
    }

    fn create_account(&self, temp_account: &NewAccount) -> Result<(), ErrResp> {
        todo!()
    }
}
