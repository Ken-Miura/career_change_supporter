// Copyright 2022 Ken Miura

use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{
    smtp::{
        SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SMTP_HOST, SMTP_PASSWORD, SMTP_PORT,
        SMTP_USERNAME, SYSTEM_EMAIL_ADDRESS,
    },
    storage::{self, CAREER_IMAGES_BUCKET_NAME},
    ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE, WEB_SITE_NAME,
};

use axum::extract::State;
use axum::http::StatusCode;
use entity::{
    admin_account, create_career_req, rejected_create_career_req,
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
    util::{
        find_user_model_by_user_account_id_with_shared_lock, session::Admin,
        validator::reason_validator::validate_reason,
    },
};

use super::find_create_career_req_model_by_create_career_req_id_with_exclusive_lock;

static SUBJECT: Lazy<String> = Lazy::new(|| format!("[{}] 職務経歴登録拒否通知", WEB_SITE_NAME));

pub(crate) async fn post_create_career_request_rejection(
    Admin { account_id }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(create_career_req_rejection): Json<CreateCareerReqRejection>,
) -> RespResult<CreateCareerReqRejectionResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = CreateCareerReqRejectionOperationImpl { pool };
    let smtp_client = SmtpClient::new(
        SMTP_HOST.to_string(),
        *SMTP_PORT,
        SMTP_USERNAME.to_string(),
        SMTP_PASSWORD.to_string(),
    );
    handle_create_career_request_rejection(
        account_id,
        create_career_req_rejection.create_career_req_id,
        create_career_req_rejection.rejection_reason,
        current_date_time,
        op,
        smtp_client,
    )
    .await
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateCareerReqRejection {
    pub(crate) create_career_req_id: i64,
    pub(crate) rejection_reason: String,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateCareerReqRejectionResult {}

async fn handle_create_career_request_rejection(
    admin_account_id: i64,
    create_career_req_id: i64,
    rejection_reason: String,
    rejected_time: DateTime<FixedOffset>,
    op: impl CreateCareerReqRejectionOperation,
    send_mail: impl SendMail,
) -> RespResult<CreateCareerReqRejectionResult> {
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
    let user_account_id_option = op
        .get_user_account_id_by_create_career_req_id(create_career_req_id)
        .await?;
    let user_account_id = user_account_id_option.ok_or_else(|| {
        error!(
            "no create career request (create career request id: {}) found",
            create_career_req_id
        );
        unexpected_err_resp()
    })?;

    let rejected_user = op
        .reject_create_career_req(
            user_account_id,
            create_career_req_id,
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

    Ok((StatusCode::OK, Json(CreateCareerReqRejectionResult {})))
}

#[async_trait]
trait CreateCareerReqRejectionOperation {
    async fn get_admin_email_address_by_admin_account_id(
        &self,
        admin_account_id: i64,
    ) -> Result<Option<String>, ErrResp>;

    async fn get_user_account_id_by_create_career_req_id(
        &self,
        create_career_req_id: i64,
    ) -> Result<Option<i64>, ErrResp>;

    async fn reject_create_career_req(
        &self,
        user_account_id: i64,
        create_career_req_id: i64,
        refuser_email_address: String,
        rejection_reason: String,
        rejected_time: DateTime<FixedOffset>,
    ) -> Result<Option<String>, ErrResp>;
}

struct CreateCareerReqRejectionOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl CreateCareerReqRejectionOperation for CreateCareerReqRejectionOperationImpl {
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

    async fn get_user_account_id_by_create_career_req_id(
        &self,
        create_career_req_id: i64,
    ) -> Result<Option<i64>, ErrResp> {
        let model = create_career_req::Entity::find_by_id(create_career_req_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find create_career_req (create_career_req_id: {}): {}",
                    create_career_req_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.user_account_id))
    }

    async fn reject_create_career_req(
        &self,
        user_account_id: i64,
        create_career_req_id: i64,
        refuser_email_address: String,
        rejection_reason: String,
        rejected_time: DateTime<FixedOffset>,
    ) -> Result<Option<String>, ErrResp> {
        let notification_email_address_option = self
            .pool
            .transaction::<_, Option<String>, ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let user_option =
                        find_user_model_by_user_account_id_with_shared_lock(txn, user_account_id)
                            .await?;
                    let user = match user_option {
                        Some(m) => m,
                        None => return Ok(None),
                    };

                    let req =
                        find_create_career_req_model_by_create_career_req_id_with_exclusive_lock(
                            txn,
                            create_career_req_id,
                        )
                        .await?;

                    let rejected_req_active_model =
                        generate_rejected_create_career_req_active_model(
                            req.clone(),
                            rejected_time,
                            rejection_reason,
                            refuser_email_address,
                        );
                    let _ = rejected_req_active_model.insert(txn).await.map_err(|e| {
                        error!(
                      "failed to insert rejected_create_career_req (create_career_req_id: {}): {}",
                        create_career_req_id, e
                      );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let _ = create_career_req::Entity::delete_by_id(create_career_req_id)
                        .exec(txn)
                        .await
                        .map_err(|e| {
                            error!(
                                "failed to delete create_career_req (create_career_req_id: {}): {}",
                                create_career_req_id, e
                            );
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;

                    let _ = delete_career_images(
                        user_account_id,
                        req.image1_file_name_without_ext,
                        req.image2_file_name_without_ext,
                    )
                    .await?;

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
                    error!("failed to reject create_career_req: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(notification_email_address_option)
    }
}

fn generate_rejected_create_career_req_active_model(
    model: create_career_req::Model,
    rejected_time: DateTime<FixedOffset>,
    rejection_reason: String,
    refuser_email_address: String,
) -> rejected_create_career_req::ActiveModel {
    rejected_create_career_req::ActiveModel {
        rjd_cre_career_req_id: NotSet,
        user_account_id: Set(model.user_account_id),
        company_name: Set(model.company_name),
        department_name: Set(model.department_name),
        office: Set(model.office),
        career_start_date: Set(model.career_start_date),
        career_end_date: Set(model.career_end_date),
        contract_type: Set(model.contract_type),
        profession: Set(model.profession),
        annual_income_in_man_yen: Set(model.annual_income_in_man_yen),
        is_manager: Set(model.is_manager),
        position_name: Set(model.position_name),
        is_new_graduate: Set(model.is_new_graduate),
        note: Set(model.note),
        reason: Set(rejection_reason),
        rejected_at: Set(rejected_time),
        rejected_by: Set(refuser_email_address),
    }
}

async fn delete_career_images(
    user_account_id: i64,
    image1_file_name_without_ext: String,
    image2_file_name_without_ext: Option<String>,
) -> Result<(), ErrRespStruct> {
    let image1_key = format!("{}/{}.png", user_account_id, image1_file_name_without_ext);
    storage::delete_object(CAREER_IMAGES_BUCKET_NAME, image1_key.as_str())
        .await
        .map_err(|e| {
            error!(
                "failed to delete career image1 (key: {}): {}",
                image1_key, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;

    if let Some(image2_file_name_without_ext) = image2_file_name_without_ext {
        let image2_key = format!("{}/{}.png", user_account_id, image2_file_name_without_ext);
        storage::delete_object(CAREER_IMAGES_BUCKET_NAME, image2_key.as_str())
            .await
            .map_err(|e| {
                error!(
                    "failed to delete career image2 (key: {}): {}",
                    image2_key, e
                );
                ErrRespStruct {
                    err_resp: unexpected_err_resp(),
                }
            })?;
    }

    Ok(())
}

fn create_text(rejection_reason: String) -> String {
    // TODO: 文面の調整
    format!(
        r"下記の【拒否理由】により、職務経歴の登録を拒否いたしました。お手数ですが、再度職務経歴確認依頼をお願いいたします。

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
        create_career_request::create_career_request_rejection::{
            create_text, handle_create_career_request_rejection, CreateCareerReqRejectionResult,
            SUBJECT,
        },
        err::Code,
        util::tests::SendMailMock,
    };

    use super::CreateCareerReqRejectionOperation;

    struct Admin {
        admin_account_id: i64,
        email_address: String,
    }

    #[derive(Clone)]
    struct User {
        user_account_id: i64,
        email_address: String,
    }

    #[derive(Clone)]
    struct CreateCareerReqMock {
        create_career_req_id: i64,
        user_account_id: i64,
    }

    struct CreateCareerReqRejectionOperationMock {
        admin: Admin,
        user_option: Option<User>,
        create_career_req_mock: CreateCareerReqMock,
        rejection_reason: String,
        rejected_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl CreateCareerReqRejectionOperation for CreateCareerReqRejectionOperationMock {
        async fn get_admin_email_address_by_admin_account_id(
            &self,
            admin_account_id: i64,
        ) -> Result<Option<String>, ErrResp> {
            assert_eq!(self.admin.admin_account_id, admin_account_id);
            Ok(Some(self.admin.email_address.clone()))
        }

        async fn get_user_account_id_by_create_career_req_id(
            &self,
            create_career_req_id: i64,
        ) -> Result<Option<i64>, ErrResp> {
            assert_eq!(
                self.create_career_req_mock.create_career_req_id,
                create_career_req_id
            );
            Ok(Some(self.create_career_req_mock.user_account_id))
        }

        async fn reject_create_career_req(
            &self,
            user_account_id: i64,
            create_career_req_id: i64,
            refuser_email_address: String,
            rejection_reason: String,
            rejected_time: DateTime<FixedOffset>,
        ) -> Result<Option<String>, ErrResp> {
            if let Some(user) = self.user_option.clone() {
                assert_eq!(user.user_account_id, user_account_id);
                assert_eq!(self.admin.email_address, refuser_email_address);
                assert_eq!(
                    self.create_career_req_mock.create_career_req_id,
                    create_career_req_id
                );
                assert_eq!(self.rejection_reason, rejection_reason);
                assert_eq!(self.rejected_time, rejected_time);
                Ok(Some(user.email_address))
            } else {
                Ok(None)
            }
        }
    }

    #[tokio::test]
    async fn handle_create_career_request_rejection_success() {
        let admin_account_id = 23;
        let admin = Admin {
            admin_account_id,
            email_address: String::from("admin@test.com"),
        };
        let user_account_id = 53;
        let user_email_address = String::from("test@test.com");
        let user_option = Some(User {
            user_account_id,
            email_address: user_email_address.clone(),
        });
        let create_career_req_id = 51514;
        let create_career_req = CreateCareerReqMock {
            create_career_req_id,
            user_account_id,
        };
        let rejection_reason = "画像が不鮮明なため";
        let rejected_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 4, 5, 21, 0, 40)
            .unwrap();
        let op_mock = CreateCareerReqRejectionOperationMock {
            admin,
            user_option,
            create_career_req_mock: create_career_req,
            rejection_reason: rejection_reason.to_string(),
            rejected_time,
        };
        let send_mail_mock = SendMailMock::new(
            user_email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(rejection_reason.to_string()),
        );

        let result = handle_create_career_request_rejection(
            admin_account_id,
            create_career_req_id,
            rejection_reason.to_string(),
            rejected_time,
            op_mock,
            send_mail_mock,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(CreateCareerReqRejectionResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_create_career_request_rejection_fail_invalid_format_reason() {
        let admin_account_id = 23;
        let admin = Admin {
            admin_account_id,
            email_address: String::from("admin@test.com"),
        };
        let user_account_id = 53;
        let user_email_address = String::from("test@test.com");
        let user_option = Some(User {
            user_account_id,
            email_address: user_email_address.clone(),
        });
        let create_career_req_id = 51514;
        let create_career_req = CreateCareerReqMock {
            create_career_req_id,
            user_account_id,
        };
        let rejection_reason = "<script>alert('test');<script>";
        let rejected_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 4, 5, 21, 0, 40)
            .unwrap();
        let op_mock = CreateCareerReqRejectionOperationMock {
            admin,
            user_option,
            create_career_req_mock: create_career_req,
            rejection_reason: rejection_reason.to_string(),
            rejected_time,
        };
        let send_mail_mock = SendMailMock::new(
            user_email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(rejection_reason.to_string()),
        );

        let result = handle_create_career_request_rejection(
            admin_account_id,
            create_career_req_id,
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
    async fn handle_create_career_request_rejection_fail_no_user_account_found() {
        let admin_account_id = 23;
        let admin = Admin {
            admin_account_id,
            email_address: String::from("admin@test.com"),
        };
        let user_account_id = 53;
        let user_email_address = String::from("test@test.com");
        let create_career_req_id = 51514;
        let create_career_req = CreateCareerReqMock {
            create_career_req_id,
            user_account_id,
        };
        let rejection_reason = "画像が不鮮明なため";
        let rejected_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 4, 5, 21, 0, 40)
            .unwrap();
        let op_mock = CreateCareerReqRejectionOperationMock {
            admin,
            user_option: None,
            create_career_req_mock: create_career_req,
            rejection_reason: rejection_reason.to_string(),
            rejected_time,
        };
        let send_mail_mock = SendMailMock::new(
            user_email_address.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(rejection_reason.to_string()),
        );

        let result = handle_create_career_request_rejection(
            admin_account_id,
            create_career_req_id,
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
