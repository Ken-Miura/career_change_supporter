// Copyright 2021 Ken Miura

use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{
    smtp::{SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS},
    storage::StorageClient,
    ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE, WEB_SITE_NAME,
};

use axum::extract::State;
use axum::http::StatusCode;
use entity::{
    create_identity_req, rejected_create_identity_req,
    sea_orm::{
        ActiveModelTrait, ActiveValue::NotSet, DatabaseConnection, Set, TransactionError,
        TransactionTrait,
    },
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    err::{unexpected_err_resp, Code},
    handlers::session::authentication::authenticated_handlers::{
        admin::Admin, identity_request::delete_identity_images, reason_validator::validate_reason,
        user_account_operation::find_user_account_model_by_user_account_id_with_shared_lock,
    },
};

use super::{
    delete_create_identity_req,
    find_create_identity_req_model_by_user_account_id_with_exclusive_lock,
};

static SUBJECT: Lazy<String> =
    Lazy::new(|| format!("[{}] ユーザー情報登録拒否通知", WEB_SITE_NAME));

pub(crate) async fn post_create_identity_request_rejection(
    Admin { admin_info }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(smtp_client): State<SmtpClient>,
    State(storage_client): State<StorageClient>,
    State(pool): State<DatabaseConnection>,
    Json(create_identity_req_rejection): Json<CreateIdentityReqRejection>,
) -> RespResult<CreateIdentityReqRejectionResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = CreateIdentityReqRejectionOperationImpl {
        pool,
        storage_client,
    };
    handle_create_identity_request_rejection(
        admin_info.email_address,
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
    user_account_id: i64,
    rejection_reason: String,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateIdentityReqRejectionResult {}

async fn handle_create_identity_request_rejection(
    admin_email_address: String,
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

    let rejected_user = op
        .reject_create_identity_req(
            user_account_id,
            admin_email_address,
            rejection_reason.clone(),
            rejected_time,
        )
        .await?;

    let user_email_address = match rejected_user {
        Some(u) => u,
        None => {
            // 拒否をしようとした際、既にユーザーがアカウントを削除しているケース、またはDisabledになっているケース
            info!(
                "no user account (user account id: {}) found or the account is disabled",
                user_account_id
            );
            return Ok((StatusCode::OK, Json(CreateIdentityReqRejectionResult {})));
        }
    };

    send_mail
        .send_mail(
            &user_email_address,
            SYSTEM_EMAIL_ADDRESS.as_str(),
            &SUBJECT,
            create_text(rejection_reason).as_str(),
        )
        .await?;

    Ok((StatusCode::OK, Json(CreateIdentityReqRejectionResult {})))
}

#[async_trait]
trait CreateIdentityReqRejectionOperation {
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
    storage_client: StorageClient,
}

#[async_trait]
impl CreateIdentityReqRejectionOperation for CreateIdentityReqRejectionOperationImpl {
    async fn reject_create_identity_req(
        &self,
        user_account_id: i64,
        refuser_email_address: String,
        rejection_reason: String,
        rejected_time: DateTime<FixedOffset>,
    ) -> Result<Option<String>, ErrResp> {
        let storage_client = self.storage_client.clone();
        let notification_email_address_option = self
            .pool
            .transaction::<_, Option<String>, ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let user_option = find_user_account_model_by_user_account_id_with_shared_lock(txn, user_account_id).await?;

                    let req = find_create_identity_req_model_by_user_account_id_with_exclusive_lock(txn, user_account_id).await?;

                    let user = match user_option {
                        Some(m) => m,
                        None => {
                            delete_create_identity_req(user_account_id, txn).await?;
                            return Ok(None)
                        },
                    };
                    if user.disabled_at.is_some() {
                        delete_create_identity_req(user_account_id, txn).await?;
                        return Ok(None)
                    }

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

                    delete_create_identity_req(user_account_id, txn).await?;

                    let _ = delete_identity_images(storage_client, user_account_id, req.image1_file_name_without_ext, req.image2_file_name_without_ext).await?;

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
        rejection_reason,
        INQUIRY_EMAIL_ADDRESS.as_str()
    )
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::{DateTime, FixedOffset, TimeZone};
    use common::{smtp::SYSTEM_EMAIL_ADDRESS, ErrResp, JAPANESE_TIME_ZONE};

    use crate::{
        err::Code, handlers::session::authentication::authenticated_handlers::tests::SendMailMock,
    };

    use super::*;

    #[derive(Clone)]
    struct User {
        user_account_id: i64,
        email_address: String,
    }

    struct CreateIdentityReqRejectionOperationMock {
        admin_email_address: String,
        user_option: Option<User>,
        rejection_reason: String,
        rejected_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl CreateIdentityReqRejectionOperation for CreateIdentityReqRejectionOperationMock {
        async fn reject_create_identity_req(
            &self,
            user_account_id: i64,
            refuser_email_address: String,
            rejection_reason: String,
            rejected_time: DateTime<FixedOffset>,
        ) -> Result<Option<String>, ErrResp> {
            if let Some(user) = self.user_option.clone() {
                assert_eq!(user.user_account_id, user_account_id);
                assert_eq!(self.admin_email_address, refuser_email_address);
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
        let admin_email_address = String::from("admin@test.com");
        let user_account_id = 53215;
        let user_email_address = String::from("test@test.com");
        let user_option = Some(User {
            user_account_id,
            email_address: user_email_address.clone(),
        });
        let rejection_reason = "画像が不鮮明なため";
        let rejected_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 4, 5, 21, 0, 40)
            .unwrap();
        let op_mock = CreateIdentityReqRejectionOperationMock {
            admin_email_address: admin_email_address.clone(),
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
            admin_email_address,
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
        let admin_email_address = String::from("admin@test.com");
        let user_account_id = 53215;
        let user_email_address = String::from("test@test.com");
        let user_option = Some(User {
            user_account_id,
            email_address: user_email_address.clone(),
        });
        let rejection_reason = "<script>alert('test');<script>";
        let rejected_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 4, 5, 21, 0, 40)
            .unwrap();
        let op_mock = CreateIdentityReqRejectionOperationMock {
            admin_email_address: admin_email_address.clone(),
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
            admin_email_address,
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
    async fn handle_create_identity_request_rejection_success_no_user_account_found() {
        let admin_email_address = String::from("admin@test.com");
        let user_account_id = 53215;
        let user_email_address = String::from("test@test.com");
        let rejection_reason = "画像が不鮮明なため";
        let rejected_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 4, 5, 21, 0, 40)
            .unwrap();
        let op_mock = CreateIdentityReqRejectionOperationMock {
            admin_email_address: admin_email_address.clone(),
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
            admin_email_address,
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
}
