// Copyright 2021 Ken Miura

use axum::async_trait;
use axum::extract::Extension;
use axum::{http::StatusCode, Json};
use chrono::{DateTime, FixedOffset};
use common::smtp::{INQUIRY_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS};
use common::util::hash_password;
use common::{
    smtp::{SendMail, SmtpClient, SOCKET_FOR_SMTP_SERVER},
    ErrResp, RespResult, ValidCred,
};
use common::{ApiError, URL_FOR_FRONT_END, VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR};
use entity::prelude::UserTempAccount;
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};
use entity::user_temp_account;
use once_cell::sync::Lazy;
use serde::Serialize;
use uuid::{adapter::Simple, Uuid};

use crate::err::unexpected_err_resp;
use crate::err::Code::ReachTempAccountsLimit;
use crate::util::{JAPANESE_TIME_ZONE, WEB_SITE_NAME};

// TODO: 運用しながら上限を調整する
const MAX_NUM_OF_TEMP_ACCOUNTS: usize = 5;

static SUBJECT: Lazy<String> = Lazy::new(|| format!("[{}] 新規登録用URLのお知らせ", WEB_SITE_NAME));

/// 一時アカウントを作成する。<br>
/// # NOTE
/// （アカウントの存在確認に悪用されないように）既にアカウントがあるかどうかのチェックはしない<br>
/// 既にアカウントがある場合は、アカウント作成時にエラーとする<br>
/// <br>
/// # Errors
/// MAX_NUM_OF_TEMP_ACCOUNTS以上一時アカウントがある場合、ステータスコード400、エラーコード[REACH_TEMP_ACCOUNTS_LIMIT]を返す
pub(crate) async fn post_temp_accounts(
    ValidCred(cred): ValidCred,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<TempAccountsResult> {
    let uuid = Uuid::new_v4().to_simple();
    let current_date_time = chrono::Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
    let op = TempAccountsOperationImpl::new(pool);
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
    handle_temp_accounts_req(
        &cred.email_address,
        &cred.password,
        &URL_FOR_FRONT_END.to_string(),
        &uuid,
        &current_date_time,
        op,
        smtp_client,
    )
    .await
}

#[derive(Serialize, Debug)]
pub(crate) struct TempAccountsResult {}

// これをテスト対象と考える。
async fn handle_temp_accounts_req(
    email_addr: &str,
    password: &str,
    url: &str,
    simple_uuid: &Simple,
    register_time: &DateTime<FixedOffset>,
    op: impl TempAccountsOperation,
    send_mail: impl SendMail,
) -> RespResult<TempAccountsResult> {
    let hashed_pwd = hash_password(password).map_err(|e| {
        tracing::error!("failed to handle password: {}", e);
        unexpected_err_resp()
    })?;
    let uuid = simple_uuid.to_string();
    let uuid_for_url = uuid.clone();
    let cnt = op.num_of_temp_accounts(email_addr).await?;
    // DBの分離レベルがSerializeでないため、MAX_NUM_OF_TEMP_ACCOUNTSを超える可能性を考慮し、">="とする
    if cnt >= MAX_NUM_OF_TEMP_ACCOUNTS {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: ReachTempAccountsLimit as u32,
            }),
        ));
    }
    let temp_account = TempAccount {
        user_temp_account_id: uuid,
        email_address: email_addr.to_string(),
        hashed_password: hashed_pwd,
        created_at: *register_time,
    };
    let _ = op.create_temp_account(&temp_account).await?;
    tracing::info!(
        "{} created temporary account with id: {} at {}",
        email_addr,
        simple_uuid,
        register_time
    );
    let text = create_text(url, &uuid_for_url);
    let _ =
        async { send_mail.send_mail(email_addr, SYSTEM_EMAIL_ADDRESS, &SUBJECT, &text) }.await?;
    Ok((StatusCode::OK, Json(TempAccountsResult {})))
}

fn create_text(url: &str, uuid_str: &str) -> String {
    // TODO: 文面の調整
    format!(
        r"!!注意!! まだ新規登録は完了していません。

このたびは、{}の新規登録手続きをしていただき、ありがとうございます。

下記URLに、PCまたはスマートフォンでアクセスしてご登録手続きの完了をお願いいたします。
{}/account-creation?temp-account-id={}

※このURLの有効期間は手続き受付時より{}時間です。URLが無効となった場合は、最初からやり直してください。
※本メールにお心あたりが無い場合、他の方が誤ってあなたのメールアドレスを入力した可能性があります。お心あたりがない場合、本メールは破棄していただくようお願いいたします。

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        WEB_SITE_NAME, url, uuid_str, VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR, INQUIRY_EMAIL_ADDRESS
    )
}

#[async_trait]
trait TempAccountsOperation {
    // DBの分離レベルにはREAD COMITTEDを想定。
    // その想定の上でトランザクションが必要かどうかを検討し、操作を分離して実装
    async fn num_of_temp_accounts(&self, email_addr: &str) -> Result<usize, ErrResp>;
    async fn create_temp_account(&self, temp_account: &TempAccount) -> Result<(), ErrResp>;
}

struct TempAccountsOperationImpl {
    pool: DatabaseConnection,
}

impl TempAccountsOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TempAccountsOperation for TempAccountsOperationImpl {
    async fn num_of_temp_accounts(&self, email_addr: &str) -> Result<usize, ErrResp> {
        let num = UserTempAccount::find()
            .filter(user_temp_account::Column::EmailAddress.eq(email_addr))
            .count(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "failed to count user temp account for {}: {}",
                    email_addr,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(num)
    }

    async fn create_temp_account(&self, temp_account: &TempAccount) -> Result<(), ErrResp> {
        let temp_account_model = user_temp_account::ActiveModel {
            user_temp_account_id: Set(temp_account.user_temp_account_id.to_string()),
            email_address: Set(temp_account.email_address.to_string()),
            hashed_password: Set(temp_account.hashed_password.clone()),
            created_at: Set(temp_account.created_at),
        };
        let _ = temp_account_model.insert(&self.pool).await.map_err(|e| {
            tracing::error!(
                "failed to insert temp account (id: {}, email address: {}): {}",
                temp_account.user_temp_account_id,
                temp_account.email_address,
                e
            );
            unexpected_err_resp()
        })?;
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TempAccount {
    pub(crate) user_temp_account_id: String,
    pub(crate) email_address: String,
    pub(crate) hashed_password: Vec<u8>,
    pub(crate) created_at: DateTime<FixedOffset>,
}

#[cfg(test)]
mod tests {

    use common::util::{
        is_password_match,
        validator::{validate_email_address, validate_password},
    };

    use crate::util::tests::SendMailMock;

    use super::*;

    struct TempAccountsOperationMock<'a> {
        cnt: usize,
        uuid: &'a str,
        email_address: &'a str,
        password: &'a str,
        register_time: &'a DateTime<FixedOffset>,
    }

    impl<'a> TempAccountsOperationMock<'a> {
        fn new(
            cnt: usize,
            uuid: &'a str,
            email_address: &'a str,
            password: &'a str,
            register_time: &'a DateTime<FixedOffset>,
        ) -> Self {
            Self {
                cnt,
                uuid,
                email_address,
                password,
                register_time,
            }
        }
    }

    #[async_trait]
    impl<'a> TempAccountsOperation for TempAccountsOperationMock<'a> {
        async fn num_of_temp_accounts(&self, email_addr: &str) -> Result<usize, ErrResp> {
            assert_eq!(self.email_address, email_addr);
            Ok(self.cnt)
        }

        async fn create_temp_account(&self, temp_account: &TempAccount) -> Result<(), ErrResp> {
            assert_eq!(self.uuid, temp_account.user_temp_account_id);
            assert_eq!(self.email_address, temp_account.email_address);
            let result = is_password_match(self.password, &temp_account.hashed_password)
                .expect("failed to get Ok");
            assert!(result, "password not match");
            assert_eq!(self.register_time, &temp_account.created_at);
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_temp_accounts_req_success() {
        let email_address = "test@example.com";
        let password: &str = "aaaaaaaaaB";
        let _ = validate_email_address(email_address).expect("failed to get Ok");
        let _ = validate_password(password).expect("failed to get Ok");
        let url: &str = "https://localhost:8080";
        let uuid = Uuid::new_v4().to_simple();
        let uuid_str = uuid.to_string();
        let current_date_time = chrono::Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
        let op_mock = TempAccountsOperationMock::new(
            MAX_NUM_OF_TEMP_ACCOUNTS - 1,
            &uuid_str,
            email_address,
            password,
            &current_date_time,
        );
        let send_mail_mock = SendMailMock::new(
            email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(url, &uuid_str),
        );

        let result = handle_temp_accounts_req(
            email_address,
            password,
            url,
            &uuid,
            &current_date_time,
            op_mock,
            send_mail_mock,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(resp.0, StatusCode::OK);
    }

    #[tokio::test]
    async fn handle_temp_accounts_req_fail_reach_max_num_of_temp_accounts_limit() {
        let email_address = "test@example.com";
        let password: &str = "aaaaaaaaaB";
        let _ = validate_email_address(email_address).expect("failed to get Ok");
        let _ = validate_password(password).expect("failed to get Ok");
        let url: &str = "https://localhost:8080";
        let uuid = Uuid::new_v4().to_simple();
        let uuid_str = uuid.to_string();
        let current_date_time = chrono::Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
        let op_mock = TempAccountsOperationMock::new(
            MAX_NUM_OF_TEMP_ACCOUNTS,
            &uuid_str,
            email_address,
            password,
            &current_date_time,
        );
        let send_mail_mock = SendMailMock::new(
            email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(url, &uuid_str),
        );

        let result = handle_temp_accounts_req(
            email_address,
            password,
            url,
            &uuid,
            &current_date_time,
            op_mock,
            send_mail_mock,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1.code, ReachTempAccountsLimit as u32);
    }
}
