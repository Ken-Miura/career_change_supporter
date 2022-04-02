// Copyright 2021 Ken Miura

use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{
    smtp::{
        SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SOCKET_FOR_SMTP_SERVER, SYSTEM_EMAIL_ADDRESS,
    },
    ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE, WEB_SITE_NAME,
};

use axum::extract::Extension;
use axum::http::StatusCode;
use entity::{
    admin_account, approved_create_identity_req, create_identity_req, identity,
    sea_orm::{
        ActiveModelTrait, DatabaseConnection, EntityTrait, QuerySelect, Set, TransactionError,
        TransactionTrait,
    },
    user_account,
};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

use crate::{
    err::unexpected_err_resp,
    util::{session::Admin, validator::reason_validator::validate_reason},
};

static SUBJECT: Lazy<String> = Lazy::new(|| format!("[{}] 本人確認完了通知", WEB_SITE_NAME));

pub(crate) async fn post_create_identity_request_rejection(
    Admin { account_id }: Admin, // 認証されていることを保証するために必須のパラメータ
    Json(create_identity_req_rejection): Json<CreateIdentityReqRejection>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<CreateIdentityReqRejectionResult> {
    let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
    let op = CreateIdentityReqRejectionOperationImpl { pool };
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
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
    let _ = validate_reason(rejection_reason.as_str()).expect("failed to get Ok");
    let admin_email_address_option = op
        .get_admin_email_address_by_admin_account_id(admin_account_id)
        .await?;
    let admin_email_address = admin_email_address_option.ok_or_else(|| {
        tracing::error!(
            "no admin account (admin account id: {}) found",
            admin_account_id
        );
        // admin accountでログインしているので、admin accountがないことはunexpected errorとして処理する
        unexpected_err_resp()
    })?;

    let _ = op
        .reject_create_identity_req(user_account_id, admin_email_address, rejected_time)
        .await?;

    let user_email_address_option = op
        .get_user_email_address_by_user_account_id(user_account_id)
        .await?;
    let user_email_address = user_email_address_option.ok_or_else(|| {
        tracing::error!(
            "no user account (user account id: {}) found",
            user_account_id
        );
        // 本人確認の承認処理後（approve_create_identity_req後）に、ユーザーがアカウントを削除した場合に発生
        // かなりのレアケースのため、unexpected errorとして処理
        unexpected_err_resp()
    })?;

    let _ = async move {
        send_mail.send_mail(
            &user_email_address,
            SYSTEM_EMAIL_ADDRESS,
            &SUBJECT,
            create_text().as_str(),
        )
    }
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
        approver_email_address: String,
        rejected_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;

    async fn get_user_email_address_by_user_account_id(
        &self,
        user_account_id: i64,
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
                tracing::error!(
                    "failed to find admin account (admin account id: {}): {}",
                    admin_account_id,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.email_address))
    }

    async fn reject_create_identity_req(
        &self,
        user_account_id: i64,
        approver_email_address: String,
        rejected_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        let _ = self
            .pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let req_option = create_identity_req::Entity::find_by_id(user_account_id)
                        .lock_exclusive()
                        .one(txn)
                        .await
                        .map_err(|e| {
                            tracing::error!(
                                "failed to find create identity request (user account id: {}): {}",
                                user_account_id,
                                e
                            );
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;
                    let req = req_option.ok_or_else(|| {
                        tracing::error!(
                            "no create identity request (user account id: {}) found",
                            user_account_id
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let identity_model = CreateIdentityReqRejectionOperationImpl::generate_identity_active_model(req.clone());
                    let _ = identity_model.insert(txn).await.map_err(|e| {
                        tracing::error!(
                            "failed to insert identity (user account id: {}): {}",
                            user_account_id,
                            e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let approved_req = CreateIdentityReqRejectionOperationImpl::generate_approved_create_identity_req_active_model(req, rejected_time, approver_email_address);
                    let _ = approved_req.insert(txn).await.map_err(|e| {
                        tracing::error!(
                            "failed to insert approved create identity req (user account id: {}): {}",
                            user_account_id,
                            e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let _ = create_identity_req::Entity::delete_by_id(user_account_id).exec(txn).await.map_err(|e| {
                        tracing::error!(
                            "failed to delete create identity request (user account id: {}): {}",
                            user_account_id,
                            e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(db_err) => {
                    tracing::error!("connection error: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    tracing::error!("failed to approve create identity req: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }

    async fn get_user_email_address_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<String>, ErrResp> {
        let model = user_account::Entity::find_by_id(user_account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "failed to find user account (user account id: {}): {}",
                    user_account_id,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.email_address))
    }
}

impl CreateIdentityReqRejectionOperationImpl {
    fn generate_approved_create_identity_req_active_model(
        model: create_identity_req::Model,
        rejected_time: DateTime<FixedOffset>,
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
            approved_at: Set(rejected_time),
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
}

fn create_text() -> String {
    // TODO: 文面の調整
    format!(
        r"本人確認が完了し、ユーザー情報を登録致しました。

本人確認が完了したため、他のユーザーに相談を申し込むことが可能になりました。相談の申し込みは、ログイン後、画面上部にある相談申し込みの項目から行うことが出来ます。

他のユーザーから相談を受けるには、ご本人確認に加え、下記の三点の登録が必要となります。他のユーザーからの相談を受けたい場合、追加で下記の三点をプロフィールよりご登録いただくようお願いします。
・職歴
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

    #[tokio::test]
    async fn handle_create_identity_request_approval_success() {}
}
