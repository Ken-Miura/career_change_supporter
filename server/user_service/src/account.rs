// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
use chrono::{DateTime, Utc};
use common::{DatabaseConnection, ErrResp, RespResult, ValidCred};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection,
};
use serde::Serialize;
use uuid::{adapter::Simple, Uuid};

/// 一時アカウントを作成する。<br>
/// <br>
///
/// # Errors
///
async fn _post_temp_accounts(
    ValidCred(_cred): ValidCred,
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<TempAccount> {
    // user accountの存在の確認 (メールアドレス) -> Result
    // temp account作成 (mail, password, uuid, date_time) -> Result<個数>
    // 成功時にメール送信
    let uuid = Uuid::new_v4().to_simple();
    let user = UserImpl::new(conn);
    let send_mail = SendMailImpl::new("email_address".to_string());
    let current_date_time = chrono::Utc::now();
    let ret = post_temp_accounts_internal(
        "test@test.com",
        "aaaaaaaaaA",
        uuid,
        current_date_time,
        user,
        send_mail,
    )
    .await?;
    Ok(ret)
}

#[derive(Serialize)]
struct TempAccount {
    email_address: String,
}

// これをテスト対象と考える。
async fn post_temp_accounts_internal(
    email_address: &str,
    password: &str,
    simple_uuid: Simple,
    registered_time: DateTime<Utc>,
    user: impl User,
    send_mail: impl SendMail,
) -> RespResult<TempAccount> {
    let a = async { user.user_exists(email_address) }.await;
    let b =
        async { user.create_temp_account(email_address, password, simple_uuid, registered_time) }
            .await;
    let c = async { send_mail.send_mail() }.await;
    let ret = (
        StatusCode::OK,
        Json(TempAccount {
            email_address: "test@test.com".to_string(),
        }),
    );
    Ok(ret)
}

trait User {
    fn user_exists(&self, email_address: &str) -> Result<bool, ErrResp>;
    fn create_temp_account(
        &self,
        email_address: &str,
        password: &str,
        simple_uuid: Simple,
        registered_time: DateTime<Utc>,
    ) -> Result<u32, ErrResp>;
}

struct UserImpl {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl UserImpl {
    fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { conn }
    }
}

impl User for UserImpl {
    fn user_exists(&self, email_address: &str) -> Result<bool, ErrResp> {
        todo!()
    }

    fn create_temp_account(
        &self,
        email_address: &str,
        password: &str,
        simple_uuid: Simple,
        registered_time: DateTime<Utc>,
    ) -> Result<u32, ErrResp> {
        todo!()
    }
}

trait SendMail {
    fn send_mail(&self) -> Result<(), ErrResp>;
}

struct SendMailImpl {
    email_address: String,
}

impl SendMailImpl {
    fn new(email_address: String) -> Self {
        Self { email_address }
    }
}

impl SendMail for SendMailImpl {
    fn send_mail(&self) -> Result<(), ErrResp> {
        todo!()
    }
}
