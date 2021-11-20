// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
use chrono::{DateTime, Utc};
use common::model::user::NewNewPassword;
use common::schema::ccs_schema::new_password::dsl::{
    email_address as new_password_email_addr, new_password as new_password_col,
};
use common::schema::ccs_schema::new_password::table as new_password_table;
use common::smtp::{INQUIRY_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS};
use common::util::hash_password;
use common::{
    smtp::{SendMail, SmtpClient, SOCKET_FOR_SMTP_SERVER},
    DatabaseConnection, ErrResp, RespResult, ValidCred,
};
use common::{ApiError, URL_FOR_FRONT_END, VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE};
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

use crate::err_code::REACH_NEW_PASSWORDS_LIMIT;
use crate::util::{unexpected_err_resp, WEB_SITE_NAME};

// TODO: 運用しながら上限を調整する
const MAX_NUM_OF_NEW_PASSWORDS: i64 = 5;

static SUBJECT: Lazy<String> =
    Lazy::new(|| format!("[{}] パスワード変更用URLのお知らせ", WEB_SITE_NAME));

/// 新しいパスワードを作成する<br>
/// # NOTE
/// （アカウントの存在確認に悪用されないように）アカウントが存在しないこと（すること）は確認しない<br>
/// アカウントが存在しない場合、パスワード変更時にエラーとする<br>
/// <br>
/// # Errors
/// MAX_NUM_OF_NEW_PASSWORDS以上新規パスワードがある場合、ステータスコード400、エラーコード[REACH_NEW_PASSWORDS_LIMIT]を返す
pub(crate) async fn post_new_password(
    ValidCred(cred): ValidCred,
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<NewPasswordResult> {
    let uuid = Uuid::new_v4().to_simple();
    let current_date_time = chrono::Utc::now();
    let op = NewPasswordOperationImpl::new(conn);
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
    post_new_password_internal(
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
pub(crate) struct NewPasswordResult {
    email_address: String,
}

// これをテスト対象と考える。
async fn post_new_password_internal(
    email_addr: &str,
    password: &str,
    url: &str,
    simple_uuid: &Simple,
    register_time: &DateTime<Utc>,
    op: impl NewPasswordOperation,
    send_mail: impl SendMail,
) -> RespResult<NewPasswordResult> {
    let hashed_pwd = hash_password(password).map_err(|e| {
        tracing::error!("failed to handle password: {}", e);
        unexpected_err_resp()
    })?;
    let uuid = simple_uuid.to_string();
    let uuid_for_url = uuid.clone();
    let _ = async move {
        let cnt = op.num_of_new_passwords(email_addr)?;
        // DBの分離レベルがSerializeでないため、MAX_NUM_OF_NEW_PASSWORDSを超える可能性を考慮し、">="とする
        if cnt >= MAX_NUM_OF_NEW_PASSWORDS {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: REACH_NEW_PASSWORDS_LIMIT,
                }),
            ));
        }
        let new_password = NewNewPassword {
            new_password_id: &uuid,
            email_address: email_addr,
            hashed_password: &hashed_pwd,
            created_at: register_time,
        };
        let _ = op.create_new_password(&new_password);
        tracing::info!(
            "{} created new password with id: {} at {}",
            email_addr,
            simple_uuid,
            register_time
        );
        Ok(())
    }
    .await?;
    let text = create_text(url, &uuid_for_url);
    let _ =
        async { send_mail.send_mail(email_addr, SYSTEM_EMAIL_ADDRESS, &SUBJECT, &text) }.await?;
    Ok((
        StatusCode::OK,
        Json(NewPasswordResult {
            email_address: email_addr.to_string(),
        }),
    ))
}

fn create_text(url: &str, uuid_str: &str) -> String {
    // TODO: 文面の調整
    format!(
        r"!!注意!! まだパスワード変更は完了していません。

下記URLに、PCまたはスマートフォンでアクセスしてパスワード変更手続きの完了をお願いいたします。
{}/password-change-confirmation?new-password-id={}

※このURLの有効期間は手続き受付時より{}分間です。URLが無効となった場合は、最初からやり直してください。
※本メールにお心あたりが無い場合、他の方が誤ってあなたのメールアドレスを入力した可能性があります。お心あたりがない場合、本メールは破棄していただくようお願いいたします。

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        url, uuid_str, VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE, INQUIRY_EMAIL_ADDRESS
    )
}

trait NewPasswordOperation {
    // DBの分離レベルにはREAD COMITTEDを想定。
    // その想定の上でトランザクションが必要かどうかを検討し、操作を分離して実装
    fn num_of_new_passwords(&self, email_addr: &str) -> Result<i64, ErrResp>;
    fn create_new_password(&self, new_password: &NewNewPassword) -> Result<(), ErrResp>;
}

struct NewPasswordOperationImpl {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl NewPasswordOperationImpl {
    fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { conn }
    }
}

impl NewPasswordOperation for NewPasswordOperationImpl {
    fn num_of_new_passwords(&self, email_addr: &str) -> Result<i64, ErrResp> {
        let cnt = new_password_col
            .filter(new_password_email_addr.eq(email_addr))
            .select(count_star())
            .get_result::<i64>(&self.conn)
            .map_err(|e| {
                tracing::error!("failed to count new password for {}: {}", email_addr, e);
                unexpected_err_resp()
            })?;
        Ok(cnt)
    }

    fn create_new_password(&self, new_password: &NewNewPassword) -> Result<(), ErrResp> {
        let _ = insert_into(new_password_table)
            .values(new_password)
            .execute(&self.conn)
            .map_err(|e| {
                tracing::error!(
                    "failed to insert new password (id: {}, email address: {}): {}",
                    new_password.new_password_id,
                    new_password.email_address,
                    e
                );
                unexpected_err_resp()
            });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use common::{
        model::user::NewNewPassword,
        smtp::SYSTEM_EMAIL_ADDRESS,
        util::{
            is_password_match,
            validator::{validate_email_address, validate_password},
        },
        ErrResp,
    };
    use uuid::Uuid;

    use crate::{
        err_code::REACH_NEW_PASSWORDS_LIMIT,
        new_password::{
            create_text, post_new_password_internal, MAX_NUM_OF_NEW_PASSWORDS, SUBJECT,
        },
        util::tests::SendMailMock,
    };

    use axum::http::StatusCode;

    use super::NewPasswordOperation;

    struct NewPasswordOperationMock<'a> {
        cnt: i64,
        uuid: &'a str,
        email_address: &'a str,
        password: &'a str,
        register_time: &'a DateTime<Utc>,
    }

    impl<'a> NewPasswordOperationMock<'a> {
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

    impl<'a> NewPasswordOperation for NewPasswordOperationMock<'a> {
        fn num_of_new_passwords(&self, email_addr: &str) -> Result<i64, ErrResp> {
            assert_eq!(self.email_address, email_addr);
            Ok(self.cnt)
        }

        fn create_new_password(&self, new_password: &NewNewPassword) -> Result<(), ErrResp> {
            assert_eq!(self.uuid, new_password.new_password_id);
            assert_eq!(self.email_address, new_password.email_address);
            let result = is_password_match(self.password, new_password.hashed_password)
                .expect("failed to get Ok");
            assert!(result, "password not match");
            assert_eq!(self.register_time, new_password.created_at);
            Ok(())
        }
    }

    #[tokio::test]
    async fn new_password_success() {
        let email_address = "test@example.com";
        let new_password: &str = "aaaaaaaaaB";
        let _ = validate_email_address(email_address).expect("failed to get Ok");
        let _ = validate_password(new_password).expect("failed to get Ok");
        let url: &str = "https://localhost:8080";
        let uuid = Uuid::new_v4().to_simple();
        let uuid_str = uuid.to_string();
        let current_date_time = chrono::Utc::now();
        let op_mock = NewPasswordOperationMock::new(
            MAX_NUM_OF_NEW_PASSWORDS - 1,
            &uuid_str,
            email_address,
            new_password,
            &current_date_time,
        );
        let send_mail_mock = SendMailMock::new(
            email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(url, &uuid_str),
        );

        let result = post_new_password_internal(
            email_address,
            new_password,
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
    async fn new_password_fail_reach_max_num_of_new_passwords_limit() {
        let email_address = "test@example.com";
        let new_password: &str = "aaaaaaaaaB";
        let _ = validate_email_address(email_address).expect("failed to get Ok");
        let _ = validate_password(new_password).expect("failed to get Ok");
        let url: &str = "https://localhost:8080";
        let uuid = Uuid::new_v4().to_simple();
        let uuid_str = uuid.to_string();
        let current_date_time = chrono::Utc::now();
        let op_mock = NewPasswordOperationMock::new(
            MAX_NUM_OF_NEW_PASSWORDS,
            &uuid_str,
            email_address,
            new_password,
            &current_date_time,
        );
        let send_mail_mock = SendMailMock::new(
            email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(url, &uuid_str),
        );

        let result = post_new_password_internal(
            email_address,
            new_password,
            url,
            &uuid,
            &current_date_time,
            op_mock,
            send_mail_mock,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1.code, REACH_NEW_PASSWORDS_LIMIT);
    }
}
