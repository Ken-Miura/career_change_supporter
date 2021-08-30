// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
use chrono::{DateTime, Utc};
use common::schema::ccs_schema::user_account::dsl::{email_address, user_account};
use common::ApiError;
use common::{
    smtp::{SendMail, SmtpClient, SOCKET_FOR_SMTP_SERVER},
    DatabaseConnection, ErrResp, RespResult, ValidCred,
};
use diesel::dsl::count_star;
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::query_dsl::select_dsl::SelectDsl;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection,
};
use serde::Serialize;
use uuid::{adapter::Simple, Uuid};

use crate::err_code;

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
    let op = TempAccountsOperationImpl::new(conn);
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
    let current_date_time = chrono::Utc::now();
    let ret = post_temp_accounts_internal(
        "test@test.com",
        "aaaaaaaaaA",
        uuid,
        current_date_time,
        op,
        smtp_client,
    )
    .await?;
    Ok(ret)
}

#[derive(Serialize)]
struct TempAccount {
    email_addr: String,
}

// これをテスト対象と考える。
async fn post_temp_accounts_internal(
    email_addr: &str,
    password: &str,
    simple_uuid: Simple,
    registered_time: DateTime<Utc>,
    op: impl TempAccountsOperation,
    send_mail: impl SendMail,
) -> RespResult<TempAccount> {
    let _a = async {
        let _ = op.user_exists(email_addr);
        op.create_temp_account(email_addr, password, simple_uuid, registered_time)
    }
    .await;
    let _b = async {
        send_mail.send_mail("to@test.com", "from@test.com", "サブジェクト", "テキスト")
    }
    .await;
    let ret = (
        StatusCode::OK,
        Json(TempAccount {
            email_addr: "test@test.com".to_string(),
        }),
    );
    Ok(ret)
}

trait TempAccountsOperation {
    fn user_exists(&self, email_addr: &str) -> Result<bool, ErrResp>;
    fn create_temp_account(
        &self,
        email_addr: &str,
        password: &str,
        simple_uuid: Simple,
        registered_time: DateTime<Utc>,
    ) -> Result<u32, ErrResp>;
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
        let cnt = user_account
            .filter(email_address.eq(email_addr))
            .select(count_star())
            .get_result::<i64>(&self.conn)
            .map_err(|e| {
                tracing::error!(
                    "failed to check if user account ({}) exists: {}",
                    email_addr,
                    e
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError {
                        code: err_code::UNEXPECTED_ERR,
                    }),
                )
            })?;
        Ok(cnt != 0)
    }

    fn create_temp_account(
        &self,
        email_addr: &str,
        password: &str,
        simple_uuid: Simple,
        registered_time: DateTime<Utc>,
    ) -> Result<u32, ErrResp> {
        todo!()
    }
}
