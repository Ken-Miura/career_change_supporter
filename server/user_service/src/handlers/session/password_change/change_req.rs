// Copyright 2021 Ken Miura

use axum::async_trait;
use axum::extract::State;
use axum::{http::StatusCode, Json};
use chrono::{DateTime, FixedOffset};
use common::smtp::{
    INQUIRY_EMAIL_ADDRESS, SMTP_HOST, SMTP_PASSWORD, SMTP_PORT, SMTP_USERNAME, SYSTEM_EMAIL_ADDRESS,
};
use common::util::validator::email_address_validator::validate_email_address;
use common::{
    smtp::{SendMail, SmtpClient},
    ErrResp, RespResult,
};
use common::{
    ApiError, JAPANESE_TIME_ZONE, URL_FOR_FRONT_END, VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE,
    WEB_SITE_NAME,
};
use entity::prelude::PwdChangeReq;
use entity::pwd_change_req;
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use uuid::fmt::Simple;
use uuid::Uuid;

use crate::err::unexpected_err_resp;
use crate::err::Code::ReachPasswordChangeReqLimit;

// TODO: 運用しながら上限を調整する
const MAX_NUM_OF_PWD_CHANGE_REQ: u64 = 8;

static SUBJECT: Lazy<String> =
    Lazy::new(|| format!("[{}] パスワード変更用URLのお知らせ", WEB_SITE_NAME));

/// パスワード変更の要求を受け付ける<br>
/// # NOTE
/// （アカウントの存在確認に悪用されないように）アカウントが存在しないこと（すること）は確認しない<br>
/// アカウントが存在しない場合、実際のパスワード変更時にエラーとする<br>
/// <br>
/// # Errors
/// メールアドレスが不正な形式の場合、ステータスコード400、エラーコード[common::err::Code::InvalidEmailAddressFormat]を返す
/// MAX_NUM_OF_PWD_CHANGE_REQ以上新規パスワードがある場合、ステータスコード400、エラーコード[ReachPasswordChangeReqLimit]を返す
pub(crate) async fn post_password_change_req(
    State(pool): State<DatabaseConnection>,
    Json(account): Json<Account>,
) -> RespResult<PasswordChangeReqResult> {
    let uuid = Uuid::new_v4().simple();
    let current_date_time = chrono::Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = PasswordChangeReqOperationImpl::new(pool);
    let smtp_client = SmtpClient::new(
        SMTP_HOST.to_string(),
        *SMTP_PORT,
        SMTP_USERNAME.to_string(),
        SMTP_PASSWORD.to_string(),
    );
    handle_password_change_req(
        &account.email_address,
        &URL_FOR_FRONT_END.to_string(),
        &uuid,
        &current_date_time,
        op,
        smtp_client,
    )
    .await
}

#[derive(Deserialize)]
pub(crate) struct Account {
    email_address: String,
}

#[derive(Serialize, Debug)]
pub(crate) struct PasswordChangeReqResult {}

async fn handle_password_change_req(
    email_addr: &str,
    url: &str,
    simple_uuid: &Simple,
    requested_time: &DateTime<FixedOffset>,
    op: impl PasswordChangeReqOperation,
    send_mail: impl SendMail,
) -> RespResult<PasswordChangeReqResult> {
    validate_email_address(email_addr).map_err(|e| {
        error!("failed to validate email address ({}): {}", email_addr, e,);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: common::err::Code::InvalidEmailAddressFormat as u32,
            }),
        )
    })?;
    let uuid = simple_uuid.to_string();
    let uuid_for_url = uuid.clone();
    let cnt = op.num_of_pwd_change_req(email_addr).await?;
    // DBの分離レベルがSerializeでないため、MAX_NUM_OF_PWD_CHANGE_REQを超える可能性を考慮し、">="とする
    if cnt >= MAX_NUM_OF_PWD_CHANGE_REQ {
        error!(
            "reach max password change trial (cnt: {}, max: {})",
            cnt, MAX_NUM_OF_PWD_CHANGE_REQ
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: ReachPasswordChangeReqLimit as u32,
            }),
        ));
    }
    let new_pwd_change_req = PasswordChangeReq {
        pwd_change_id: uuid,
        email_address: email_addr.to_string(),
        requested_at: *requested_time,
    };
    op.create_new_pwd_change_req(new_pwd_change_req).await?;
    info!(
        "{} created new password change request with request id: {} at {}",
        email_addr, simple_uuid, requested_time
    );
    let text = create_text(url, &uuid_for_url);
    send_mail
        .send_mail(email_addr, SYSTEM_EMAIL_ADDRESS, &SUBJECT, &text)
        .await?;
    Ok((StatusCode::OK, Json(PasswordChangeReqResult {})))
}

#[derive(Clone, Debug)]
struct PasswordChangeReq {
    pwd_change_id: String,
    email_address: String,
    requested_at: DateTime<FixedOffset>,
}

fn create_text(url: &str, uuid_str: &str) -> String {
    // TODO: 文面の調整
    format!(
        r"!!注意!! まだパスワード変更は完了していません。

下記URLに、PCまたはスマートフォンでアクセスしてパスワード変更手続きの完了をお願いいたします。
{}/password-update?pwd-change-req-id={}

※このURLの有効期間は手続き受付時より{}分間です。URLが無効となった場合は、最初からやり直してください。
※本メールにお心あたりが無い場合、他の方が誤ってあなたのメールアドレスを入力した可能性があります。お心あたりがない場合、本メールは破棄していただくようお願いいたします。

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        url, uuid_str, VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE, INQUIRY_EMAIL_ADDRESS
    )
}

#[async_trait]
trait PasswordChangeReqOperation {
    // DBの分離レベルにはREAD COMITTEDを想定。
    // その想定の上でトランザクションが必要かどうかを検討し、操作を分離して実装
    async fn num_of_pwd_change_req(&self, email_addr: &str) -> Result<u64, ErrResp>;
    async fn create_new_pwd_change_req(
        &self,
        new_password_change_req: PasswordChangeReq,
    ) -> Result<(), ErrResp>;
}

struct PasswordChangeReqOperationImpl {
    pool: DatabaseConnection,
}

impl PasswordChangeReqOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PasswordChangeReqOperation for PasswordChangeReqOperationImpl {
    async fn num_of_pwd_change_req(&self, email_addr: &str) -> Result<u64, ErrResp> {
        let num = PwdChangeReq::find()
            .filter(pwd_change_req::Column::EmailAddress.eq(email_addr))
            .count(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to count num of email_address ({}) in pwd_change_req: {}",
                    email_addr, e
                );
                unexpected_err_resp()
            })?;
        Ok(num)
    }

    async fn create_new_pwd_change_req(
        &self,
        new_password_change_req: PasswordChangeReq,
    ) -> Result<(), ErrResp> {
        let model = pwd_change_req::ActiveModel {
            pwd_change_req_id: Set(new_password_change_req.pwd_change_id.clone()),
            email_address: Set(new_password_change_req.email_address.clone()),
            requested_at: Set(new_password_change_req.requested_at),
        };
        let _ = model.insert(&self.pool).await.map_err(|e| {
            error!(
                "failed to insert pwd_change_req (pwd_change_id: {}, email_address: {}, requested_at {}): {}",
                new_password_change_req.pwd_change_id,
                new_password_change_req.email_address,
                new_password_change_req.requested_at,
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
        util::validator::email_address_validator::validate_email_address, ErrResp,
        JAPANESE_TIME_ZONE,
    };
    use uuid::Uuid;

    use super::{create_text, handle_password_change_req, MAX_NUM_OF_PWD_CHANGE_REQ, SUBJECT};
    use crate::{err::Code::ReachPasswordChangeReqLimit, handlers::tests::SendMailMock};

    use axum::http::StatusCode;

    use super::{PasswordChangeReq, PasswordChangeReqOperation};

    struct PasswordChangeReqOperationMock<'a> {
        cnt: u64,
        uuid: &'a str,
        email_address: &'a str,
        requested_time: &'a DateTime<FixedOffset>,
    }

    impl<'a> PasswordChangeReqOperationMock<'a> {
        fn new(
            cnt: u64,
            uuid: &'a str,
            email_address: &'a str,
            requested_time: &'a DateTime<FixedOffset>,
        ) -> Self {
            Self {
                cnt,
                uuid,
                email_address,
                requested_time,
            }
        }
    }

    #[async_trait]
    impl<'a> PasswordChangeReqOperation for PasswordChangeReqOperationMock<'a> {
        async fn num_of_pwd_change_req(&self, email_addr: &str) -> Result<u64, ErrResp> {
            assert_eq!(self.email_address, email_addr);
            Ok(self.cnt)
        }

        async fn create_new_pwd_change_req(
            &self,
            new_password_change_req: PasswordChangeReq,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.uuid, new_password_change_req.pwd_change_id);
            assert_eq!(self.email_address, new_password_change_req.email_address);
            assert_eq!(self.requested_time, &new_password_change_req.requested_at);
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_password_change_req_success() {
        let email_address = "test@example.com";
        validate_email_address(email_address).expect("failed to get Ok");
        let url: &str = "https://localhost:8080";
        let uuid = Uuid::new_v4().simple();
        let uuid_str = uuid.to_string();
        let current_date_time = chrono::Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
        let op_mock = PasswordChangeReqOperationMock::new(
            MAX_NUM_OF_PWD_CHANGE_REQ - 1,
            &uuid_str,
            email_address,
            &current_date_time,
        );
        let send_mail_mock = SendMailMock::new(
            email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(url, &uuid_str),
        );

        let result = handle_password_change_req(
            email_address,
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
    async fn handle_password_change_req_fail_reach_max_num_of_pwd_change_req_limit() {
        let email_address = "test@example.com";
        validate_email_address(email_address).expect("failed to get Ok");
        let url: &str = "https://localhost:8080";
        let uuid = Uuid::new_v4().simple();
        let uuid_str = uuid.to_string();
        let current_date_time = chrono::Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
        let op_mock = PasswordChangeReqOperationMock::new(
            MAX_NUM_OF_PWD_CHANGE_REQ,
            &uuid_str,
            email_address,
            &current_date_time,
        );
        let send_mail_mock = SendMailMock::new(
            email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(url, &uuid_str),
        );

        let result = handle_password_change_req(
            email_address,
            url,
            &uuid,
            &current_date_time,
            op_mock,
            send_mail_mock,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1.code, ReachPasswordChangeReqLimit as u32);
    }

    #[tokio::test]
    async fn handle_password_change_req_fail_invalid_email_address() {
        let email_address = "<script>alert('test')</script>";
        let url: &str = "https://localhost:8080";
        let uuid = Uuid::new_v4().simple();
        let uuid_str = uuid.to_string();
        let current_date_time = chrono::Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
        let op_mock = PasswordChangeReqOperationMock::new(
            MAX_NUM_OF_PWD_CHANGE_REQ - 1,
            &uuid_str,
            email_address,
            &current_date_time,
        );
        let send_mail_mock = SendMailMock::new(
            email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(url, &uuid_str),
        );

        let result = handle_password_change_req(
            email_address,
            url,
            &uuid,
            &current_date_time,
            op_mock,
            send_mail_mock,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(
            resp.1.code,
            common::err::Code::InvalidEmailAddressFormat as u32
        );
    }
}
