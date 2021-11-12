// Copyright 2021 Ken Miura

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

use crate::err_code::{
    INVALID_UUID, NEW_PASSWORD_EXPIRED, NO_ACCOUNT_FOUND, NO_NEW_PASSWORD_FOUND,
};
use crate::util::{unexpected_err_resp, WEB_SITE_NAME};

static SUBJECT: Lazy<String> = Lazy::new(|| format!("[{}] パスワード変更完了通知", WEB_SITE_NAME));

/// 新しいパスワードに変更する<br>
/// <br>
/// # Errors
/// アカウントがない場合、ステータスコード400、エラーコード[NO_ACCOUNT_FOUND]を返す<br>
/// UUIDが不正な形式の場合、ステータスコード400、エラーコード[INVALID_UUID]を返す<br>
/// 新しいパスワードが見つからない場合、ステータスコード400、エラーコード[NO_NEW_PASSWORD_FOUND]を返す<br>
/// 新しいパスワードが期限切れの場合、ステータスコード400、エラーコード[NEW_PASSWORD_EXPIRED]を返す<br>
pub(crate) async fn post_password_change(
    Json(new_pwd): Json<NewPasswordId>,
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<PasswordChangeResult> {
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
            Json(ApiError { code: INVALID_UUID }),
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
                    code: NEW_PASSWORD_EXPIRED,
                }),
            ));
        }
        let account = find_account_by_email_address(&new_pwd.email_address, &op)?;
        let _ = op.update_password(account.user_account_id, &new_pwd.hashed_password)?;
        Ok(new_pwd.email_address)
    }
    .await?;
    let text = create_text();
    let _ =
        async { send_mail.send_mail(&email_addr, SYSTEM_EMAIL_ADDRESS, &SUBJECT, &text) }.await?;
    Ok((StatusCode::OK, Json(PasswordChangeResult {})))
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
                code: NO_ACCOUNT_FOUND,
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
                            code: NO_NEW_PASSWORD_FOUND,
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
mod tests {}
