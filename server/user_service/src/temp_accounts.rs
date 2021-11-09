// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
use chrono::{DateTime, Utc};
use common::model::user::NewTempAccount;
use common::schema::ccs_schema::user_temp_account::dsl::{
    email_address as temp_user_email_addr, user_temp_account,
};
use common::schema::ccs_schema::user_temp_account::table as user_temp_account_table;
use common::smtp::{INQUIRY_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS};
use common::util::hash_password;
use common::{
    smtp::{SendMail, SmtpClient, SOCKET_FOR_SMTP_SERVER},
    DatabaseConnection, ErrResp, RespResult, ValidCred,
};
use common::{ApiError, URL_FOR_FRONT_END};
use diesel::dsl::count_star;
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::query_dsl::select_dsl::SelectDsl;
use diesel::RunQueryDsl;
use diesel::{insert_into, ExpressionMethods};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection,
};
use once_cell::sync::Lazy;
use serde::Serialize;
use uuid::{adapter::Simple, Uuid};

use crate::err_code::REACH_TEMP_ACCOUNTS_LIMIT;
use crate::util::{unexpected_err_resp, WEB_SITE_NAME};

// TODO: 運用しながら上限を調整する
const MAX_NUM_OF_TEMP_ACCOUNTS: i64 = 5;

// TODO: 文面の調整
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
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<TempAccountsResult> {
    let uuid = Uuid::new_v4().to_simple();
    let current_date_time = chrono::Utc::now();
    let op = TempAccountsOperationImpl::new(conn);
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
    post_temp_accounts_internal(
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
pub(crate) struct TempAccountsResult {
    email_address: String,
}

// これをテスト対象と考える。
async fn post_temp_accounts_internal(
    email_addr: &str,
    password: &str,
    url: &str,
    simple_uuid: &Simple,
    register_time: &DateTime<Utc>,
    op: impl TempAccountsOperation,
    send_mail: impl SendMail,
) -> RespResult<TempAccountsResult> {
    let hashed_pwd = hash_password(password).map_err(|e| {
        tracing::error!("failed to handle password: {}", e);
        unexpected_err_resp()
    })?;
    let uuid = simple_uuid.to_string();
    let uuid_for_url = uuid.clone();
    let _ = async move {
        let cnt = op.num_of_temp_accounts(email_addr)?;
        // DBの分離レベルがSerializeでないため、MAX_NUM_OF_TEMP_ACCOUNTSを超える可能性を考慮し、">="とする
        if cnt >= MAX_NUM_OF_TEMP_ACCOUNTS {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: REACH_TEMP_ACCOUNTS_LIMIT,
                }),
            ));
        }
        let temp_account = NewTempAccount {
            user_temp_account_id: &uuid,
            email_address: email_addr,
            hashed_password: &hashed_pwd,
            created_at: register_time,
        };
        op.create_temp_account(&temp_account)
    }
    .await?;
    let text = create_text(url, &uuid_for_url);
    let _ =
        async { send_mail.send_mail(email_addr, SYSTEM_EMAIL_ADDRESS, &SUBJECT, &text) }.await?;
    Ok((
        StatusCode::OK,
        Json(TempAccountsResult {
            email_address: email_addr.to_string(),
        }),
    ))
}

fn create_text(url: &str, uuid_str: &str) -> String {
    // TODO: 文面の調整
    format!(
        r"!!注意!! まだ新規登録は完了していません。

このたびは、{}の新規登録手続きをしていただき、ありがとうございます。

下記URLに、PCまたはスマートフォンでアクセスしてご登録手続きの完了をお願いいたします。
{}/accounts?temp-account-id={}

※このURLの有効期間は手続き受付時より24時間です。URLが無効となった場合は、最初からやり直してください。
※本メールにお心あたりが無い場合、他の方が誤ってあなたのメールアドレスを入力した可能性があります。お心あたりがない場合、本メールは破棄していただくようお願いいたします。

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        WEB_SITE_NAME, url, uuid_str, INQUIRY_EMAIL_ADDRESS
    )
}

trait TempAccountsOperation {
    // DBの分離レベルにはREAD COMITTEDを想定。
    // その想定の上でトランザクションが必要かどうかを検討し、操作を分離して実装
    fn num_of_temp_accounts(&self, email_addr: &str) -> Result<i64, ErrResp>;
    fn create_temp_account(&self, temp_account: &NewTempAccount) -> Result<(), ErrResp>;
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
    fn num_of_temp_accounts(&self, email_addr: &str) -> Result<i64, ErrResp> {
        let cnt = user_temp_account
            .filter(temp_user_email_addr.eq(email_addr))
            .select(count_star())
            .get_result::<i64>(&self.conn)
            .map_err(|e| {
                tracing::error!(
                    "failed to count user temp account ({}) exists: {}",
                    email_addr,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(cnt)
    }

    fn create_temp_account(&self, temp_account: &NewTempAccount) -> Result<(), ErrResp> {
        let _ = insert_into(user_temp_account_table)
            .values(temp_account)
            .execute(&self.conn)
            .map_err(|e| {
                tracing::error!(
                    "failed to insert user temp account (id: {}, email address: {}): {}",
                    temp_account.user_temp_account_id,
                    temp_account.email_address,
                    e
                );
                unexpected_err_resp()
            });
        Ok(())
    }
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
        cnt: i64,
        uuid: &'a str,
        email_address: &'a str,
        password: &'a str,
        register_time: &'a DateTime<Utc>,
    }

    impl<'a> TempAccountsOperationMock<'a> {
        fn new(
            cnt: i64,
            uuid: &'a str,
            email_address: &'a str,
            password: &'a str,
            register_time: &'a DateTime<Utc>,
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

    impl<'a> TempAccountsOperation for TempAccountsOperationMock<'a> {
        fn num_of_temp_accounts(&self, email_addr: &str) -> Result<i64, ErrResp> {
            assert_eq!(self.email_address, email_addr);
            Ok(self.cnt)
        }

        fn create_temp_account(&self, temp_account: &NewTempAccount) -> Result<(), ErrResp> {
            assert_eq!(self.uuid, temp_account.user_temp_account_id);
            assert_eq!(self.email_address, temp_account.email_address);
            let result = is_password_match(self.password, temp_account.hashed_password)
                .expect("failed to get Ok");
            assert!(result, "password not match");
            assert_eq!(self.register_time, temp_account.created_at);
            Ok(())
        }
    }

    #[tokio::test]
    async fn temp_accounts_success() {
        let email_address = "test@example.com";
        let password: &str = "aaaaaaaaaB";
        let _ = validate_email_address(email_address).expect("failed to get Ok");
        let _ = validate_password(password).expect("failed to get Ok");
        let url: &str = "http://localhost:8080";
        let uuid = Uuid::new_v4().to_simple();
        let uuid_str = uuid.to_string();
        let current_date_time = chrono::Utc::now();
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

        let result = post_temp_accounts_internal(
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
        assert_eq!(resp.1.email_address, email_address);
    }

    #[tokio::test]
    async fn temp_accounts_fail_reach_max_num_of_temp_accounts_limit() {
        let email_address = "test@example.com";
        let password: &str = "aaaaaaaaaB";
        let _ = validate_email_address(email_address).expect("failed to get Ok");
        let _ = validate_password(password).expect("failed to get Ok");
        let url: &str = "http://localhost:8080";
        let uuid = Uuid::new_v4().to_simple();
        let uuid_str = uuid.to_string();
        let current_date_time = chrono::Utc::now();
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

        let result = post_temp_accounts_internal(
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
        assert_eq!(resp.1.code, REACH_TEMP_ACCOUNTS_LIMIT);
    }
}
