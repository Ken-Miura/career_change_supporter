// Copyright 2021 Ken Miura

use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{
    smtp::{
        SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SMTP_HOST, SMTP_PASSWORD, SMTP_PORT,
        SMTP_USERNAME, SYSTEM_EMAIL_ADDRESS,
    },
    ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE, WEB_SITE_NAME,
};

use axum::extract::Extension;
use axum::http::StatusCode;
use entity::{
    admin_account, create_identity_req, rejected_create_identity_req,
    sea_orm::{
        ActiveModelTrait, ActiveValue::NotSet, DatabaseConnection, EntityTrait, Set,
        TransactionError, TransactionTrait,
    },
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    err::{unexpected_err_resp, Code},
    identity_request::delete_identity_images,
    util::{
        find_user_model_by_user_account_id_with_shared_lock, session::Admin,
        validator::reason_validator::validate_reason,
    },
};

use super::find_create_identity_req_model_by_user_account_id_with_exclusive_lock;

static SUBJECT: Lazy<String> =
    Lazy::new(|| format!("[{}] ユーザー情報登録拒否通知", WEB_SITE_NAME));

pub(crate) async fn post_create_identity_request_rejection(
    Admin { account_id }: Admin, // 認証されていることを保証するために必須のパラメータ
    Json(create_identity_req_rejection): Json<CreateIdentityReqRejection>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<CreateIdentityReqRejectionResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = CreateIdentityReqRejectionOperationImpl { pool };
    let smtp_client = SmtpClient::new(
        SMTP_HOST.to_string(),
        *SMTP_PORT,
        SMTP_USERNAME.to_string(),
        SMTP_PASSWORD.to_string(),
    );
    handle_create_identity_request_rejection(
        account_id,
        create_identity_req_rejection.user_account_id,
        create_identity_req_rejection.rejection_reason,
        current_date_time,
        op,
        smtp_client,
    )
    .await
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateIdentityReqRejection {
    pub(crate) user_account_id: i64,
    pub(crate) rejection_reason: String,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateIdentityReqRejectionResult {}

async fn handle_create_identity_request_rejection(
    admin_account_id: i64,
    user_account_id: i64,
    rejection_reason: String,
    rejected_time: DateTime<FixedOffset>,
    op: impl CreateIdentityReqRejectionOperation,
    send_mail: impl SendMail,
) -> RespResult<CreateIdentityReqRejectionResult> {
    validate_reason(rejection_reason.as_str()).map_err(|e| {
        error!("invalid format reason ({}): {}", rejection_reason, e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidFormatReason as u32,
            }),
        )
    })?;
    let admin_email_address_option = op
        .get_admin_email_address_by_admin_account_id(admin_account_id)
        .await?;
    let admin_email_address = admin_email_address_option.ok_or_else(|| {
        error!(
            "no admin account (admin account id: {}) found",
            admin_account_id
        );
        // admin accountでログインしているので、admin accountがないことはunexpected errorとして処理する
        unexpected_err_resp()
    })?;

    let rejected_user = op
        .reject_create_identity_req(
            user_account_id,
            admin_email_address,
            rejection_reason.clone(),
            rejected_time,
        )
        .await?;

    let user_email_address = rejected_user.ok_or_else(|| {
        // 拒否をしようとした際、既にユーザーがアカウントを削除しているケース
        error!(
            "no user account (user account id: {}) found",
            user_account_id
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoUserAccountFound as u32,
            }),
        )
    })?;

    send_mail
        .send_mail(
            &user_email_address,
            SYSTEM_EMAIL_ADDRESS,
            &SUBJECT,
            create_text(rejection_reason).as_str(),
        )
        .await?;

    Ok((StatusCode::OK, Json(CreateIdentityReqRejectionResult {})))
}

#[async_trait]
trait CreateIdentityReqRejectionOperation {
    async fn get_admin_email_address_by_admin_account_id(
        &self,
        admin_account_id: i64,
    ) -> Result<Option<String>, ErrResp>;

    async fn reject_create_identity_req(
        &self,
        user_account_id: i64,
        refuser_email_address: String,
        rejection_reason: String,
        rejected_time: DateTime<FixedOffset>,
    ) -> Result<Option<String>, ErrResp>;
}

struct CreateIdentityReqRejectionOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl CreateIdentityReqRejectionOperation for CreateIdentityReqRejectionOperationImpl {
    async fn get_admin_email_address_by_admin_account_id(
        &self,
        admin_account_id: i64,
    ) -> Result<Option<String>, ErrResp> {
        let model = admin_account::Entity::find_by_id(admin_account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find admin_account (admin_account_id: {}): {}",
                    admin_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.email_address))
    }

    async fn reject_create_identity_req(
        &self,
        user_account_id: i64,
        refuser_email_address: String,
        rejection_reason: String,
        rejected_time: DateTime<FixedOffset>,
    ) -> Result<Option<String>, ErrResp> {
        let notification_email_address_option = self
            .pool
            .transaction::<_, Option<String>, ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let user_option = find_user_model_by_user_account_id_with_shared_lock(txn, user_account_id).await?;
                    let user = match user_option {
                        Some(m) => m,
                        None => { return Ok(None) },
                    };

                    let req = find_create_identity_req_model_by_user_account_id_with_exclusive_lock(txn, user_account_id).await?;

                    let rejected_req_active_model = generate_rejected_create_identity_req_active_model(req.clone(), rejected_time, rejection_reason, refuser_email_address);
                    let _ = rejected_req_active_model.insert(txn).await.map_err(|e| {
                        error!(
                            "failed to insert rejected_create_identity_req (user_account_id: {}): {}",
                            user_account_id,
                            e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let _ = create_identity_req::Entity::delete_by_id(user_account_id).exec(txn).await.map_err(|e| {
                        error!(
                            "failed to delete create_identity_req (user_account_id: {}): {}",
                            user_account_id,
                            e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let _ = delete_identity_images(user_account_id, req.image1_file_name_without_ext, req.image2_file_name_without_ext).await?;

                    Ok(Some(user.email_address))
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(db_err) => {
                    error!("connection error: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    error!("failed to reject create_identity_req: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(notification_email_address_option)
    }
}

fn generate_rejected_create_identity_req_active_model(
    model: create_identity_req::Model,
    rejected_time: DateTime<FixedOffset>,
    rejection_reason: String,
    refuser_email_address: String,
) -> rejected_create_identity_req::ActiveModel {
    rejected_create_identity_req::ActiveModel {
        rjd_cre_identity_id: NotSet,
        user_account_id: Set(model.user_account_id),
        last_name: Set(model.last_name),
        first_name: Set(model.first_name),
        last_name_furigana: Set(model.last_name_furigana),
        first_name_furigana: Set(model.first_name_furigana),
        date_of_birth: Set(model.date_of_birth),
        prefecture: Set(model.prefecture),
        city: Set(model.city),
        address_line1: Set(model.address_line1),
        address_line2: Set(model.address_line2),
        telephone_number: Set(model.telephone_number),
        reason: Set(rejection_reason),
        rejected_at: Set(rejected_time),
        rejected_by: Set(refuser_email_address),
    }
}

fn create_text(rejection_reason: String) -> String {
    // TODO: 文面の調整
    format!(
        r"下記の【拒否理由】により、ユーザー情報の登録を拒否いたしました。お手数ですが、再度本人確認依頼をお願いいたします。

【拒否理由】
{}

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        rejection_reason, INQUIRY_EMAIL_ADDRESS
    )
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::{DateTime, FixedOffset, TimeZone};
    use common::{smtp::SYSTEM_EMAIL_ADDRESS, ErrResp, JAPANESE_TIME_ZONE};

    use crate::{
        err::Code,
        identity_request::create_identity_request::create_identity_request_rejection::{
            create_text, CreateIdentityReqRejectionResult, SUBJECT,
        },
        util::tests::SendMailMock,
    };

    use super::{handle_create_identity_request_rejection, CreateIdentityReqRejectionOperation};

    struct Admin {
        admin_account_id: i64,
        email_address: String,
    }

    #[derive(Clone)]
    struct User {
        user_account_id: i64,
        email_address: String,
    }

    struct CreateIdentityReqRejectionOperationMock {
        admin: Admin,
        user_option: Option<User>,
        rejection_reason: String,
        rejected_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl CreateIdentityReqRejectionOperation for CreateIdentityReqRejectionOperationMock {
        async fn get_admin_email_address_by_admin_account_id(
            &self,
            admin_account_id: i64,
        ) -> Result<Option<String>, ErrResp> {
            assert_eq!(self.admin.admin_account_id, admin_account_id);
            Ok(Some(self.admin.email_address.clone()))
        }

        async fn reject_create_identity_req(
            &self,
            user_account_id: i64,
            refuser_email_address: String,
            rejection_reason: String,
            rejected_time: DateTime<FixedOffset>,
        ) -> Result<Option<String>, ErrResp> {
            if let Some(user) = self.user_option.clone() {
                assert_eq!(user.user_account_id, user_account_id);
                assert_eq!(self.admin.email_address, refuser_email_address);
                assert_eq!(self.rejection_reason, rejection_reason);
                assert_eq!(self.rejected_time, rejected_time);
                Ok(Some(user.email_address))
            } else {
                Ok(None)
            }
        }
    }

    #[tokio::test]
    async fn handle_create_identity_request_rejection_success() {
        let admin_account_id = 23;
        let admin = Admin {
            admin_account_id,
            email_address: String::from("admin@test.com"),
        };
        let user_account_id = 53215;
        let user_email_address = String::from("test@test.com");
        let user_option = Some(User {
            user_account_id,
            email_address: user_email_address.clone(),
        });
        let rejection_reason = "画像が不鮮明なため";
        let rejected_time = JAPANESE_TIME_ZONE.ymd(2022, 4, 5).and_hms(21, 00, 40);
        let op_mock = CreateIdentityReqRejectionOperationMock {
            admin,
            user_option,
            rejection_reason: rejection_reason.to_string(),
            rejected_time,
        };
        let send_mail_mock = SendMailMock::new(
            user_email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(rejection_reason.to_string()),
        );

        let result = handle_create_identity_request_rejection(
            admin_account_id,
            user_account_id,
            rejection_reason.to_string(),
            rejected_time,
            op_mock,
            send_mail_mock,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(CreateIdentityReqRejectionResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_create_identity_request_rejection_fail_invalid_format_reason() {
        let admin_account_id = 23;
        let admin = Admin {
            admin_account_id,
            email_address: String::from("admin@test.com"),
        };
        let user_account_id = 53215;
        let user_email_address = String::from("test@test.com");
        let user_option = Some(User {
            user_account_id,
            email_address: user_email_address.clone(),
        });
        let rejection_reason = "<script>alert('test');<script>";
        let rejected_time = JAPANESE_TIME_ZONE.ymd(2022, 4, 5).and_hms(21, 00, 40);
        let op_mock = CreateIdentityReqRejectionOperationMock {
            admin,
            user_option,
            rejection_reason: rejection_reason.to_string(),
            rejected_time,
        };
        let send_mail_mock = SendMailMock::new(
            user_email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(rejection_reason.to_string()),
        );

        let result = handle_create_identity_request_rejection(
            admin_account_id,
            user_account_id,
            rejection_reason.to_string(),
            rejected_time,
            op_mock,
            send_mail_mock,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::InvalidFormatReason as u32, resp.1 .0.code);
    }

    #[tokio::test]
    async fn handle_create_identity_request_rejection_fail_no_user_account_found() {
        let admin_account_id = 23;
        let admin = Admin {
            admin_account_id,
            email_address: String::from("admin@test.com"),
        };
        let user_account_id = 53215;
        let user_email_address = String::from("test@test.com");
        let rejection_reason = "画像が不鮮明なため";
        let rejected_time = JAPANESE_TIME_ZONE.ymd(2022, 4, 5).and_hms(21, 00, 40);
        let op_mock = CreateIdentityReqRejectionOperationMock {
            admin,
            user_option: None,
            rejection_reason: rejection_reason.to_string(),
            rejected_time,
        };
        let send_mail_mock = SendMailMock::new(
            user_email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(rejection_reason.to_string()),
        );

        let result = handle_create_identity_request_rejection(
            admin_account_id,
            user_account_id,
            rejection_reason.to_string(),
            rejected_time,
            op_mock,
            send_mail_mock,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::NoUserAccountFound as u32, resp.1 .0.code);
    }
}
