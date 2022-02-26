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
use common::{ApiError, URL_FOR_FRONT_END, VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE};
use entity::new_password;
use entity::prelude::NewPassword;
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};
use once_cell::sync::Lazy;
use serde::Serialize;
use uuid::{adapter::Simple, Uuid};

use crate::err::unexpected_err_resp;
use crate::err::Code::ReachNewPasswordsLimit;
use crate::util::{JAPANESE_TIME_ZONE, WEB_SITE_NAME};

// TODO: 運用しながら上限を調整する
const MAX_NUM_OF_NEW_PASSWORDS: usize = 8;

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
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<NewPasswordResult> {
    let uuid = Uuid::new_v4().to_simple();
    let current_date_time = chrono::Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
    let op = NewPasswordOperationImpl::new(pool);
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
    handle_new_password_req(
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
pub(crate) struct NewPasswordResult {}

// これをテスト対象と考える。
async fn handle_new_password_req(
    email_addr: &str,
    password: &str,
    url: &str,
    simple_uuid: &Simple,
    register_time: &DateTime<FixedOffset>,
    op: impl NewPasswordOperation,
    send_mail: impl SendMail,
) -> RespResult<NewPasswordResult> {
    let hashed_pwd = hash_password(password).map_err(|e| {
        tracing::error!("failed to handle password: {}", e);
        unexpected_err_resp()
    })?;
    let uuid = simple_uuid.to_string();
    let uuid_for_url = uuid.clone();
    let cnt = op.num_of_new_passwords(email_addr).await?;
    // DBの分離レベルがSerializeでないため、MAX_NUM_OF_NEW_PASSWORDSを超える可能性を考慮し、">="とする
    if cnt >= MAX_NUM_OF_NEW_PASSWORDS {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: ReachNewPasswordsLimit as u32,
            }),
        ));
    }
    let new_password = NewPasswordReq {
        new_password_id: uuid,
        email_address: email_addr.to_string(),
        hashed_password: hashed_pwd,
        created_at: *register_time,
    };
    let _ = op.create_new_password(new_password).await?;
    tracing::info!(
        "{} created new password with id: {} at {}",
        email_addr,
        simple_uuid,
        register_time
    );
    let text = create_text(url, &uuid_for_url);
    let _ =
        async { send_mail.send_mail(email_addr, SYSTEM_EMAIL_ADDRESS, &SUBJECT, &text) }.await?;
    Ok((StatusCode::OK, Json(NewPasswordResult {})))
}

#[derive(Clone, Debug)]
struct NewPasswordReq {
    new_password_id: String,
    email_address: String,
    hashed_password: Vec<u8>,
    created_at: DateTime<FixedOffset>,
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

#[async_trait]
trait NewPasswordOperation {
    // DBの分離レベルにはREAD COMITTEDを想定。
    // その想定の上でトランザクションが必要かどうかを検討し、操作を分離して実装
    async fn num_of_new_passwords(&self, email_addr: &str) -> Result<usize, ErrResp>;
    async fn create_new_password(&self, new_password_req: NewPasswordReq) -> Result<(), ErrResp>;
}

struct NewPasswordOperationImpl {
    pool: DatabaseConnection,
}

impl NewPasswordOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl NewPasswordOperation for NewPasswordOperationImpl {
    async fn num_of_new_passwords(&self, email_addr: &str) -> Result<usize, ErrResp> {
        let num = NewPassword::find()
            .filter(new_password::Column::EmailAddress.eq(email_addr))
            .count(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "failed to count new password (email address: {}): {}",
                    email_addr,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(num)
    }

    async fn create_new_password(&self, new_password_req: NewPasswordReq) -> Result<(), ErrResp> {
        let model = new_password::ActiveModel {
            new_password_id: Set(new_password_req.new_password_id.clone()),
            email_address: Set(new_password_req.email_address.clone()),
            hashed_password: Set(new_password_req.hashed_password),
            created_at: Set(new_password_req.created_at),
        };
        let _ = model.insert(&self.pool).await.map_err(|e| {
            tracing::error!(
                "failed to insert new password (id: {}, email address: {}): {}",
                new_password_req.new_password_id,
                new_password_req.email_address,
                e
            );
            unexpected_err_resp()
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use chrono::{DateTime, FixedOffset};
    use common::{
        smtp::SYSTEM_EMAIL_ADDRESS,
        util::{
            is_password_match,
            validator::{validate_email_address, validate_password},
        },
        ErrResp,
    };
    use uuid::Uuid;

    use crate::{
        err::Code::ReachNewPasswordsLimit,
        new_password::{create_text, handle_new_password_req, MAX_NUM_OF_NEW_PASSWORDS, SUBJECT},
        util::{tests::SendMailMock, JAPANESE_TIME_ZONE},
    };

    use axum::http::StatusCode;

    use super::{NewPasswordOperation, NewPasswordReq};

    struct NewPasswordOperationMock<'a> {
        cnt: usize,
        uuid: &'a str,
        email_address: &'a str,
        password: &'a str,
        register_time: &'a DateTime<FixedOffset>,
    }

    impl<'a> NewPasswordOperationMock<'a> {
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
    impl<'a> NewPasswordOperation for NewPasswordOperationMock<'a> {
        async fn num_of_new_passwords(&self, email_addr: &str) -> Result<usize, ErrResp> {
            assert_eq!(self.email_address, email_addr);
            Ok(self.cnt)
        }

        async fn create_new_password(
            &self,
            new_password_req: NewPasswordReq,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.uuid, new_password_req.new_password_id);
            assert_eq!(self.email_address, new_password_req.email_address);
            let result = is_password_match(self.password, &new_password_req.hashed_password)
                .expect("failed to get Ok");
            assert!(result, "password not match");
            assert_eq!(self.register_time, &new_password_req.created_at);
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_new_password_req_success() {
        let email_address = "test@example.com";
        let new_password: &str = "aaaaaaaaaB";
        let _ = validate_email_address(email_address).expect("failed to get Ok");
        let _ = validate_password(new_password).expect("failed to get Ok");
        let url: &str = "https://localhost:8080";
        let uuid = Uuid::new_v4().to_simple();
        let uuid_str = uuid.to_string();
        let current_date_time = chrono::Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
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

        let result = handle_new_password_req(
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
    }

    #[tokio::test]
    async fn handle_new_password_req_fail_reach_max_num_of_new_passwords_limit() {
        let email_address = "test@example.com";
        let new_password: &str = "aaaaaaaaaB";
        let _ = validate_email_address(email_address).expect("failed to get Ok");
        let _ = validate_password(new_password).expect("failed to get Ok");
        let url: &str = "https://localhost:8080";
        let uuid = Uuid::new_v4().to_simple();
        let uuid_str = uuid.to_string();
        let current_date_time = chrono::Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
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

        let result = handle_new_password_req(
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
        assert_eq!(resp.1.code, ReachNewPasswordsLimit as u32);
    }
}
