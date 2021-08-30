// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
use chrono::{DateTime, Utc};
use common::model::user::NewTempAccount;
use common::schema::ccs_schema::user_account::dsl::{
    email_address as user_email_addr, user_account,
};
use common::schema::ccs_schema::user_temp_account::dsl::{
    email_address as temp_user_email_addr, user_temp_account,
};
use common::schema::ccs_schema::user_temp_account::table as user_temp_account_table;
use common::util::hash_password;
use common::{
    smtp::{SendMail, SmtpClient, SOCKET_FOR_SMTP_SERVER},
    DatabaseConnection, ErrResp, RespResult, ValidCred,
};
use diesel::dsl::count_star;
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::query_dsl::select_dsl::SelectDsl;
use diesel::RunQueryDsl;
use diesel::{insert_into, ExpressionMethods};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection,
};
use serde::Serialize;
use uuid::{adapter::Simple, Uuid};

use crate::util::unexpected_err_resp;

/// 一時アカウントを作成する。<br>
/// <br>
///
/// # Errors
///
pub(crate) async fn post_temp_accounts(
    ValidCred(cred): ValidCred,
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<TempAccountsResult> {
    let uuid = Uuid::new_v4().to_simple();
    let current_date_time = chrono::Utc::now();
    let op = TempAccountsOperationImpl::new(conn);
    //let op = TempAccountsOperationMock::new("conn".to_string());
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
    let ret = post_temp_accounts_internal(
        &cred.email_address,
        &cred.password,
        uuid,
        current_date_time,
        op,
        smtp_client,
    )
    .await?;
    Ok(ret)
}

#[derive(Serialize)]
pub(crate) struct TempAccountsResult {
    email_address: String,
}

// これをテスト対象と考える。
async fn post_temp_accounts_internal(
    email_addr: &str,
    password: &str,
    simple_uuid: Simple,
    register_time: DateTime<Utc>,
    op: impl TempAccountsOperation,
    send_mail: impl SendMail,
) -> RespResult<TempAccountsResult> {
    let hashed_pwd = hash_password(password).map_err(|e| {
        tracing::error!("failed to handle password: {}", e);
        unexpected_err_resp()
    })?;
    // NOTE: このasyncブロックのありなしで、
    // the trait `Handler<_, _>` is not implemented for `fn(ValidCred, DatabaseConnection) -> impl std::future::Future {post_temp_accounts}`
    // のコンパイルエラーのありなしが変わる。なぜasyncのありなしが外側の関数（post_temp_accounts）のtrait boundに影響を与えるのか不明。
    // 
    // -> PooledConnection<ConnectionManager<PgConnection>>がSendを実装していないことが原因のように見える。
    let _ = async {
        let exists = op.user_exists(email_addr)?;
        if exists {
            todo!()
        }
        let cnt = op.num_of_temp_accounts(email_addr)?;
        if cnt > 6 {
            todo!()
        }
        let temp_account = NewTempAccount {
            user_temp_account_id: &simple_uuid.to_string(),
            email_address: email_addr,
            hashed_password: &hashed_pwd,
            created_at: &register_time,
        };
        op.create_temp_account(temp_account)
    }
    .await?;
    let _ = async {
        send_mail.send_mail("to@test.com", "from@test.com", "サブジェクト", "テキスト")
    }
    .await?;
    Ok((
        StatusCode::OK,
        Json(TempAccountsResult {
            email_address: email_addr.to_string(),
        }),
    ))
}

trait TempAccountsOperation {
    // DBの分離レベルにはREAD COMITTEDを想定。
    // その想定の上でトランザクションが必要かどうかを検討し、操作を分離して実装
    fn user_exists(&self, email_addr: &str) -> Result<bool, ErrResp>;
    fn num_of_temp_accounts(&self, email_addr: &str) -> Result<i64, ErrResp>;
    fn create_temp_account(&self, temp_account: NewTempAccount) -> Result<(), ErrResp>;
}

struct TempAccountsOperationImpl {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl TempAccountsOperationImpl {
    fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { conn }
    }
}

impl TempAccountsOperation for TempAccountsOperationImpl {
    fn user_exists(&self, email_addr: &str) -> Result<bool, ErrResp> {
        todo!()
    }

    fn num_of_temp_accounts(&self, email_addr: &str) -> Result<i64, ErrResp> {
        todo!()
    }

    fn create_temp_account(&self, temp_account: NewTempAccount) -> Result<(), ErrResp> {
        todo!()
    }
}

struct TempAccountsOperationMock {
    member: String
}

impl TempAccountsOperationMock {
    fn new(member: String) -> Self { Self { member } }
}

impl TempAccountsOperation for TempAccountsOperationMock {
    fn user_exists(&self, email_addr: &str) -> Result<bool, ErrResp> {
        todo!()
    }

    fn num_of_temp_accounts(&self, email_addr: &str) -> Result<i64, ErrResp> {
        todo!()
    }

    fn create_temp_account(&self, temp_account: NewTempAccount) -> Result<(), ErrResp> {
        todo!()
    }
}