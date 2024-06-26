// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use common::smtp::{SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS};
use common::{ApiError, ErrResp, RespResult, WEB_SITE_NAME};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use entity::{consultation_req, user_account};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use super::validate_consultation_req_id_is_positive;
use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::authentication::authenticated_handlers::authenticated_users::verified_user::VerifiedUser;
use crate::handlers::session::authentication::authenticated_handlers::consultation::{
    consultation_req_exists, ConsultationRequest,
};

static CONSULTATION_REQ_REJECTION_MAIL_SUBJECT: Lazy<String> =
    Lazy::new(|| format!("[{}] 相談申し込み拒否通知", WEB_SITE_NAME));

pub(crate) async fn post_consultation_request_rejection(
    VerifiedUser { user_info }: VerifiedUser,
    State(smtp_client): State<SmtpClient>,
    State(pool): State<DatabaseConnection>,
    Json(param): Json<ConsultationRequestRejectionParam>,
) -> RespResult<ConsultationRequestRejectionResult> {
    let consultation_req_id = param.consultation_req_id;
    let op = ConsultationRequestRejectionImpl { pool };
    handle_consultation_request_rejection(
        user_info.account_id,
        consultation_req_id,
        op,
        smtp_client,
    )
    .await
}

#[derive(Deserialize)]
pub(crate) struct ConsultationRequestRejectionParam {
    consultation_req_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationRequestRejectionResult {}

async fn handle_consultation_request_rejection(
    consultant_id: i64,
    consultation_req_id: i64,
    op: impl ConsultationRequestRejection,
    send_mail: impl SendMail,
) -> RespResult<ConsultationRequestRejectionResult> {
    validate_consultation_req_id_is_positive(consultation_req_id)?;

    let req = op
        .find_consultation_req_by_consultation_req_id(consultation_req_id)
        .await?;
    let req = consultation_req_exists(req, consultation_req_id)?;
    validate_consultation_req_for_delete(&req, consultant_id)?;

    op.delete_consultation_req(req.consultation_req_id).await?;

    send_consultation_req_rejection_mail_if_user_exists(
        req.user_account_id,
        req.consultation_req_id,
        &op,
        &send_mail,
    )
    .await?;

    info!("rejected consultation request ({:?})", req);
    Ok((StatusCode::OK, Json(ConsultationRequestRejectionResult {})))
}

#[async_trait]
trait ConsultationRequestRejection {
    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp>;

    async fn delete_consultation_req(&self, consultation_req_id: i64) -> Result<(), ErrResp>;

    async fn find_user_email_address_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<String>, ErrResp>;
}

struct ConsultationRequestRejectionImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationRequestRejection for ConsultationRequestRejectionImpl {
    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp> {
        super::super::find_consultation_req_by_consultation_req_id(&self.pool, consultation_req_id)
            .await
    }

    async fn delete_consultation_req(&self, consultation_req_id: i64) -> Result<(), ErrResp> {
        consultation_req::Entity::delete_by_id(consultation_req_id)
            .exec(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to delete consultation_req (consultation_req_id: {}): {}",
                    consultation_req_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(())
    }

    async fn find_user_email_address_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<String>, ErrResp> {
        let model_option = user_account::Entity::find_by_id(user_account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find user_account (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model_option.map(|m| m.email_address))
    }
}

fn validate_consultation_req_for_delete(
    consultation_req: &ConsultationRequest,
    consultant_id: i64,
) -> Result<(), ErrResp> {
    if consultation_req.consultant_id != consultant_id {
        error!(
            "consultant_id ({}) does not match consultation_req.consultant_id ({})",
            consultant_id, consultation_req.consultant_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationReqFound as u32,
            }),
        ));
    }
    Ok(())
}

async fn send_consultation_req_rejection_mail_if_user_exists(
    user_account_id: i64,
    consultation_req_id: i64,
    op: &impl ConsultationRequestRejection,
    send_mail: &impl SendMail,
) -> Result<(), ErrResp> {
    let user_email_address = op
        .find_user_email_address_by_user_account_id(user_account_id)
        .await?;
    // メールアドレスが取得出来ない = アカウント削除済みを意味するのでそのケースは通知の必要なし
    if let Some(user_email_address) = user_email_address {
        info!(
            "send consultation request rejection mail (consultation_req_id: {}) to {}",
            consultation_req_id, user_email_address
        );
        send_mail
            .send_mail(
                user_email_address.as_str(),
                SYSTEM_EMAIL_ADDRESS.as_str(),
                CONSULTATION_REQ_REJECTION_MAIL_SUBJECT.as_str(),
                create_text(consultation_req_id).as_str(),
            )
            .await?;
    }
    Ok(())
}

fn create_text(consultation_req_id: i64) -> String {
    format!(
        r"相談申し込み（相談申し込み番号: {}）が拒否されました。
        
【お問い合わせ先】
Email: {}",
        consultation_req_id,
        INQUIRY_EMAIL_ADDRESS.as_str()
    )
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;
    use common::JAPANESE_TIME_ZONE;

    use crate::handlers::tests::SendMailMock;

    use super::*;

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<ConsultationRequestRejectionResult>,
    }

    #[derive(Debug)]
    struct Input {
        consultant_id: i64,
        consultation_req_id: i64,
        op: ConsultationRequestRejectionMock,
        smtp_client: SendMailMock,
    }

    #[derive(Clone, Debug)]
    struct ConsultationRequestRejectionMock {
        consultation_req_id: i64,
        consultation_req: Option<ConsultationRequest>,
        user_account_id: i64,
        user_email_address: Option<String>,
    }

    #[async_trait]
    impl ConsultationRequestRejection for ConsultationRequestRejectionMock {
        async fn find_consultation_req_by_consultation_req_id(
            &self,
            consultation_req_id: i64,
        ) -> Result<Option<ConsultationRequest>, ErrResp> {
            assert_eq!(self.consultation_req_id, consultation_req_id);
            if let Some(consultation_req) = self.consultation_req.clone() {
                assert_eq!(consultation_req.consultation_req_id, consultation_req_id);
            }
            Ok(self.consultation_req.clone())
        }

        async fn delete_consultation_req(&self, consultation_req_id: i64) -> Result<(), ErrResp> {
            assert_eq!(self.consultation_req_id, consultation_req_id);
            Ok(())
        }

        async fn find_user_email_address_by_user_account_id(
            &self,
            user_account_id: i64,
        ) -> Result<Option<String>, ErrResp> {
            assert_eq!(self.user_account_id, user_account_id);
            Ok(self.user_email_address.clone())
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        let account_id_of_consultant = 1;
        let consultation_req_id = 3;
        let account_id_of_user = 2;
        let user_email_address = "test2@test.com".to_string();
        let mail_text = create_text(consultation_req_id);
        let dummy_consultation_req = create_dummy_consultation_req(
            consultation_req_id,
            account_id_of_consultant,
            account_id_of_user,
        );
        vec![
            TestCase {
                name: "success case (normal)".to_string(),
                input: Input {
                    consultant_id: account_id_of_consultant,
                    consultation_req_id,
                    op: ConsultationRequestRejectionMock {
                        consultation_req_id,
                        consultation_req: Some(dummy_consultation_req.clone()),
                        user_account_id: account_id_of_user,
                        user_email_address: Some(user_email_address.clone()),
                    },
                    smtp_client: SendMailMock::new(
                        user_email_address.clone(),
                        SYSTEM_EMAIL_ADDRESS.to_string(),
                        CONSULTATION_REQ_REJECTION_MAIL_SUBJECT.to_string(),
                        mail_text.clone(),
                    ),
                },
                expected: Ok((StatusCode::OK, Json(ConsultationRequestRejectionResult {}))),
            },
            TestCase {
                name: "success case (no user email address found)".to_string(),
                input: Input {
                    consultant_id: account_id_of_consultant,
                    consultation_req_id,
                    op: ConsultationRequestRejectionMock {
                        consultation_req_id,
                        consultation_req: Some(dummy_consultation_req.clone()),
                        user_account_id: account_id_of_user,
                        user_email_address: None,
                    },
                    smtp_client: SendMailMock::new(
                        user_email_address.clone(),
                        SYSTEM_EMAIL_ADDRESS.to_string(),
                        CONSULTATION_REQ_REJECTION_MAIL_SUBJECT.to_string(),
                        mail_text.clone(),
                    ),
                },
                expected: Ok((StatusCode::OK, Json(ConsultationRequestRejectionResult {}))),
            },
            TestCase {
                name: "fail NonPositiveConsultationReqId (id: 0)".to_string(),
                input: Input {
                    consultant_id: account_id_of_consultant,
                    consultation_req_id: 0,
                    op: ConsultationRequestRejectionMock {
                        consultation_req_id,
                        consultation_req: Some(dummy_consultation_req.clone()),
                        user_account_id: account_id_of_user,
                        user_email_address: Some(user_email_address.clone()),
                    },
                    smtp_client: SendMailMock::new(
                        user_email_address.clone(),
                        SYSTEM_EMAIL_ADDRESS.to_string(),
                        CONSULTATION_REQ_REJECTION_MAIL_SUBJECT.to_string(),
                        mail_text.clone(),
                    ),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NonPositiveConsultationReqId as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NonPositiveConsultationReqId (id: -1)".to_string(),
                input: Input {
                    consultant_id: account_id_of_consultant,
                    consultation_req_id: -1,
                    op: ConsultationRequestRejectionMock {
                        consultation_req_id,
                        consultation_req: Some(dummy_consultation_req),
                        user_account_id: account_id_of_user,
                        user_email_address: Some(user_email_address.clone()),
                    },
                    smtp_client: SendMailMock::new(
                        user_email_address.clone(),
                        SYSTEM_EMAIL_ADDRESS.to_string(),
                        CONSULTATION_REQ_REJECTION_MAIL_SUBJECT.to_string(),
                        mail_text.clone(),
                    ),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NonPositiveConsultationReqId as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoConsultationReqFound (no consultation request found)".to_string(),
                input: Input {
                    consultant_id: account_id_of_consultant,
                    consultation_req_id,
                    op: ConsultationRequestRejectionMock {
                        consultation_req_id,
                        consultation_req: None,
                        user_account_id: account_id_of_user,
                        user_email_address: Some(user_email_address.clone()),
                    },
                    smtp_client: SendMailMock::new(
                        user_email_address.clone(),
                        SYSTEM_EMAIL_ADDRESS.to_string(),
                        CONSULTATION_REQ_REJECTION_MAIL_SUBJECT.to_string(),
                        mail_text.clone(),
                    ),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoConsultationReqFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoConsultationReqFound (account id of consultant does not match consultant id)".to_string(),
                input: Input {
                    consultant_id: account_id_of_consultant,
                    consultation_req_id,
                    op: ConsultationRequestRejectionMock {
                        consultation_req_id,
                        consultation_req: Some(create_dummy_consultation_req(
                            consultation_req_id,
                            account_id_of_consultant + 1,
                            account_id_of_user,
                        )),
                        user_account_id: account_id_of_user,
                        user_email_address: Some(user_email_address.clone()),
                    },
                    smtp_client: SendMailMock::new(
                        user_email_address,
                        SYSTEM_EMAIL_ADDRESS.to_string(),
                        CONSULTATION_REQ_REJECTION_MAIL_SUBJECT.to_string(),
                        mail_text,
                    ),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoConsultationReqFound as u32,
                    }),
                )),
            },
        ]
    });

    fn create_dummy_consultation_req(
        consultation_req_id: i64,
        account_id_of_consultant: i64,
        account_id_of_user: i64,
    ) -> ConsultationRequest {
        ConsultationRequest {
            consultation_req_id,
            user_account_id: account_id_of_user,
            consultant_id: account_id_of_consultant,
            fee_per_hour_in_yen: 5000,
            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2022, 12, 1, 7, 0, 0)
                .unwrap(),
            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2022, 12, 2, 23, 0, 0)
                .unwrap(),
            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2022, 12, 3, 11, 0, 0)
                .unwrap(),
            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2022, 12, 3, 11, 0, 0)
                .unwrap(),
        }
    }

    #[tokio::test]
    async fn handle_handle_consultation_request_rejection() {
        for test_case in TEST_CASE_SET.iter() {
            let account_id = test_case.input.consultant_id;
            let consultation_req_id = test_case.input.consultation_req_id;
            let op = test_case.input.op.clone();
            let smtp_client = test_case.input.smtp_client.clone();

            let result = handle_consultation_request_rejection(
                account_id,
                consultation_req_id,
                op,
                smtp_client,
            )
            .await;

            let message = format!("test case \"{}\" failed", test_case.name.clone());
            if test_case.expected.is_ok() {
                let resp = result.expect("failed to get Ok");
                let expected = test_case.expected.as_ref().expect("failed to get Ok");
                assert_eq!(expected.0, resp.0, "{}", message);
                assert_eq!(expected.1 .0, resp.1 .0, "{}", message);
            } else {
                let resp = result.expect_err("failed to get Err");
                let expected = test_case.expected.as_ref().expect_err("failed to get Err");
                assert_eq!(expected.0, resp.0, "{}", message);
                assert_eq!(expected.1 .0, resp.1 .0, "{}", message);
            }
        }
    }
}
