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
    admin_account, approved_create_identity_req, create_identity_req, identity,
    sea_orm::{
        ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionError, TransactionTrait,
    },
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    err::{unexpected_err_resp, Code},
    util::{find_user_model_by_user_account_id_with_shared_lock, session::Admin},
};

use super::find_create_identity_req_model_by_user_account_id_with_exclusive_lock;

static SUBJECT: Lazy<String> = Lazy::new(|| format!("[{}] 本人確認完了通知", WEB_SITE_NAME));

pub(crate) async fn post_create_identity_request_approval(
    Admin { account_id }: Admin, // 認証されていることを保証するために必須のパラメータ
    Json(create_identity_req_approval): Json<CreateIdentityReqApproval>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<CreateIdentityReqApprovalResult> {
    let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
    let op = CreateIdentityReqApprovalOperationImpl { pool };
    let smtp_client = SmtpClient::new(
        SMTP_HOST.to_string(),
        *SMTP_PORT,
        SMTP_USERNAME.to_string(),
        SMTP_PASSWORD.to_string(),
    );
    handle_create_identity_request_approval(
        account_id,
        create_identity_req_approval.user_account_id,
        current_date_time,
        op,
        smtp_client,
    )
    .await
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateIdentityReqApproval {
    pub(crate) user_account_id: i64,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateIdentityReqApprovalResult {}

async fn handle_create_identity_request_approval(
    admin_account_id: i64,
    user_account_id: i64,
    approved_time: DateTime<FixedOffset>,
    op: impl CreateIdentityReqApprovalOperation,
    send_mail: impl SendMail,
) -> RespResult<CreateIdentityReqApprovalResult> {
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

    let approved_user = op
        .approve_create_identity_req(user_account_id, admin_email_address, approved_time)
        .await?;

    let user_email_address = approved_user.ok_or_else(|| {
        // 承認をしようとした際、既にユーザーがアカウントを削除しているケース
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

    let _ = send_mail
        .send_mail(
            &user_email_address,
            SYSTEM_EMAIL_ADDRESS,
            &SUBJECT,
            create_text().as_str(),
        )
        .await?;

    Ok((StatusCode::OK, Json(CreateIdentityReqApprovalResult {})))
}

#[async_trait]
trait CreateIdentityReqApprovalOperation {
    async fn get_admin_email_address_by_admin_account_id(
        &self,
        admin_account_id: i64,
    ) -> Result<Option<String>, ErrResp>;

    async fn approve_create_identity_req(
        &self,
        user_account_id: i64,
        approver_email_address: String,
        approved_time: DateTime<FixedOffset>,
    ) -> Result<Option<String>, ErrResp>;
}

struct CreateIdentityReqApprovalOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl CreateIdentityReqApprovalOperation for CreateIdentityReqApprovalOperationImpl {
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

    async fn approve_create_identity_req(
        &self,
        user_account_id: i64,
        approver_email_address: String,
        approved_time: DateTime<FixedOffset>,
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

                    let identity_model = generate_identity_active_model(req.clone());
                    let _ = identity_model.insert(txn).await.map_err(|e| {
                        error!(
                            "failed to insert identity (user_account_id: {}): {}",
                            user_account_id,
                            e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let approved_req = generate_approved_create_identity_req_active_model(req, approved_time, approver_email_address);
                    let _ = approved_req.insert(txn).await.map_err(|e| {
                        error!(
                            "failed to insert approved_create_identity_req (user_account_id: {}): {}",
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
                    error!("failed to approve create_identity_req: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(notification_email_address_option)
    }
}

fn generate_approved_create_identity_req_active_model(
    model: create_identity_req::Model,
    approved_time: DateTime<FixedOffset>,
    approver_email_address: String,
) -> approved_create_identity_req::ActiveModel {
    approved_create_identity_req::ActiveModel {
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
        image1_file_name_without_ext: Set(model.image1_file_name_without_ext),
        image2_file_name_without_ext: Set(model.image2_file_name_without_ext),
        approved_at: Set(approved_time),
        approved_by: Set(approver_email_address),
    }
}

fn generate_identity_active_model(model: create_identity_req::Model) -> identity::ActiveModel {
    identity::ActiveModel {
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
    }
}

fn create_text() -> String {
    // TODO: 文面の調整
    format!(
        r"本人確認が完了し、ユーザー情報を登録致しました。

本人確認が完了したため、他のユーザーに相談を申し込むことが可能になりました。相談の申し込みは、ログイン後、画面上部にある相談申し込みの項目から行うことが出来ます。

他のユーザーから相談を受けるには、ご本人確認に加え、下記の三点の登録が必要となります。他のユーザーからの相談を受けたい場合、追加で下記の三点をご登録いただくようお願いします。
・職務経歴
・相談料
・銀行口座

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        INQUIRY_EMAIL_ADDRESS
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
        identity_request::create_identity_request::create_identity_request_approval::CreateIdentityReqApprovalResult,
        util::tests::SendMailMock,
    };

    use super::{
        create_text, handle_create_identity_request_approval, CreateIdentityReqApprovalOperation,
        SUBJECT,
    };

    struct Admin {
        admin_account_id: i64,
        email_address: String,
    }

    #[derive(Clone)]
    struct User {
        user_account_id: i64,
        email_address: String,
    }

    struct CreateIdentityReqApprovalOperationMock {
        admin: Admin,
        user_option: Option<User>,
        approved_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl CreateIdentityReqApprovalOperation for CreateIdentityReqApprovalOperationMock {
        async fn get_admin_email_address_by_admin_account_id(
            &self,
            admin_account_id: i64,
        ) -> Result<Option<String>, ErrResp> {
            assert_eq!(self.admin.admin_account_id, admin_account_id);
            Ok(Some(self.admin.email_address.clone()))
        }

        async fn approve_create_identity_req(
            &self,
            user_account_id: i64,
            approver_email_address: String,
            approved_time: DateTime<FixedOffset>,
        ) -> Result<Option<String>, ErrResp> {
            if let Some(user) = self.user_option.clone() {
                assert_eq!(user.user_account_id, user_account_id);
                assert_eq!(self.admin.email_address, approver_email_address);
                assert_eq!(self.approved_time, approved_time);
                Ok(Some(user.email_address))
            } else {
                Ok(None)
            }
        }
    }

    #[tokio::test]
    async fn handle_create_identity_request_approval_success() {
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
        let approval_time = JAPANESE_TIME_ZONE.ymd(2022, 4, 1).and_hms(21, 00, 40);
        let op_mock = CreateIdentityReqApprovalOperationMock {
            admin,
            user_option,
            approved_time: approval_time,
        };
        let send_mail_mock = SendMailMock::new(
            user_email_address,
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );

        let result = handle_create_identity_request_approval(
            admin_account_id,
            user_account_id,
            approval_time,
            op_mock,
            send_mail_mock,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(CreateIdentityReqApprovalResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_create_identity_request_approval_fail_no_user_account_found() {
        let admin_account_id = 23;
        let admin = Admin {
            admin_account_id,
            email_address: String::from("admin@test.com"),
        };
        let user_account_id = 53215;
        let user_email_address = String::from("test@test.com");
        let approval_time = JAPANESE_TIME_ZONE.ymd(2022, 4, 1).and_hms(21, 00, 40);
        let op_mock = CreateIdentityReqApprovalOperationMock {
            admin,
            user_option: None,
            approved_time: approval_time,
        };
        let send_mail_mock = SendMailMock::new(
            user_email_address,
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );

        let result = handle_create_identity_request_approval(
            admin_account_id,
            user_account_id,
            approval_time,
            op_mock,
            send_mail_mock,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::NoUserAccountFound as u32, resp.1 .0.code);
    }
}
