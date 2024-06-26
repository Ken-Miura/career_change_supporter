// Copyright 2022 Ken Miura

use async_session::serde_json::json;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use common::{
    opensearch::{index_document, update_document, INDEX_NAME},
    smtp::{SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS},
    ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE, WEB_SITE_NAME,
};

use axum::extract::State;
use axum::http::StatusCode;
use entity::{
    approved_create_career_req, career, create_career_req, document,
    sea_orm::{
        ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DatabaseConnection,
        DatabaseTransaction, EntityTrait, PaginatorTrait, QueryFilter, Set, TransactionError,
        TransactionTrait,
    },
};
use once_cell::sync::Lazy;
use opensearch::OpenSearch;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{
    err::unexpected_err_resp,
    handlers::session::authentication::authenticated_handlers::{
        admin::Admin,
        document_operation::find_document_model_by_user_account_id_with_exclusive_lock,
        user_account_operation::find_user_account_model_by_user_account_id_with_shared_lock,
    },
};

use super::{
    super::find_create_career_req_model_by_create_career_req_id_with_exclusive_lock,
    delete_create_career_req,
};

static SUBJECT: Lazy<String> = Lazy::new(|| format!("[{}] 職務経歴確認完了通知", WEB_SITE_NAME));

pub(crate) async fn post_create_career_request_approval(
    Admin { admin_info }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(smtp_client): State<SmtpClient>,
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
    Json(create_career_req_approval): Json<CreateCareerReqApproval>,
) -> RespResult<CreateCareerReqApprovalResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = CreateCareerReqApprovalOperationImpl { pool, index_client };
    handle_create_career_request_approval(
        admin_info.email_address,
        create_career_req_approval.create_career_req_id,
        current_date_time,
        op,
        smtp_client,
    )
    .await
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateCareerReqApproval {
    create_career_req_id: i64,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateCareerReqApprovalResult {}

async fn handle_create_career_request_approval(
    admin_email_address: String,
    create_career_req_id: i64,
    approved_time: DateTime<FixedOffset>,
    op: impl CreateCareerReqApprovalOperation,
    send_mail: impl SendMail,
) -> RespResult<CreateCareerReqApprovalResult> {
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

    let approved_user = op
        .approve_create_career_req(
            user_account_id,
            create_career_req_id,
            admin_email_address,
            approved_time,
        )
        .await?;

    let user_email_address = match approved_user {
        Some(u) => u,
        None => {
            // 承認をしようとした際、既にユーザーがアカウントを削除しているケース、またはDisabledになっているケース
            info!(
                "no user account (user account id: {}) found or the account is disabled",
                user_account_id
            );
            return Ok((StatusCode::OK, Json(CreateCareerReqApprovalResult {})));
        }
    };

    send_mail
        .send_mail(
            &user_email_address,
            SYSTEM_EMAIL_ADDRESS.as_str(),
            &SUBJECT,
            create_text().as_str(),
        )
        .await?;

    Ok((StatusCode::OK, Json(CreateCareerReqApprovalResult {})))
}

#[async_trait]
trait CreateCareerReqApprovalOperation {
    async fn get_user_account_id_by_create_career_req_id(
        &self,
        create_career_req_id: i64,
    ) -> Result<Option<i64>, ErrResp>;

    async fn approve_create_career_req(
        &self,
        user_account_id: i64,
        create_career_req_id: i64,
        approver_email_address: String,
        approved_time: DateTime<FixedOffset>,
    ) -> Result<Option<String>, ErrResp>;
}

struct CreateCareerReqApprovalOperationImpl {
    pool: DatabaseConnection,
    index_client: OpenSearch,
}

#[async_trait]
impl CreateCareerReqApprovalOperation for CreateCareerReqApprovalOperationImpl {
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

    async fn approve_create_career_req(
        &self,
        user_account_id: i64,
        create_career_req_id: i64,
        approver_email_address: String,
        approved_time: DateTime<FixedOffset>,
    ) -> Result<Option<String>, ErrResp> {
        let index_client = self.index_client.clone();
        let notification_email_address_option = self
            .pool
            .transaction::<_, Option<String>, ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let user_option =
                        find_user_account_model_by_user_account_id_with_shared_lock(txn, user_account_id).await?;

                    let req = find_create_career_req_model_by_create_career_req_id_with_exclusive_lock(
                        txn,
                        create_career_req_id,
                    )
                    .await?;

                    let user = match user_option {
                        Some(m) => m,
                        None => return {
                            delete_create_career_req(req.create_career_req_id, txn).await?;
                            Ok(None)
                        },
                    };
                    if user.disabled_at.is_some() {
                        delete_create_career_req(req.create_career_req_id, txn).await?;
                        return Ok(None)
                    }

                    let career_active_model = generate_career_active_model(req.clone());
                    let career_model = career_active_model.insert(txn).await.map_err(|e| {
                        error!(
                            "failed to insert career (user_account_id: {}): {}",
                            user_account_id, e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let approved_req = generate_approved_create_career_req_active_model(
                        req,
                        approved_time,
                        approver_email_address,
                    );
                    let _ = approved_req.insert(txn).await.map_err(|e| {
                        error!(
                            "failed to insert approved_create_career_req (create_career_req_id: {}): {}",
                            create_career_req_id, e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    delete_create_career_req(create_career_req_id, txn).await?;

                    let num_of_careers = career::Entity::find()
                        .filter(career::Column::UserAccountId.eq(user_account_id))
                        .count(txn)
                        .await
                        .map_err(|e| {
                            error!(
                                "failed to count career (user_account_id: {}): {}",
                                user_account_id, e
                            );
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;

                    let document_option =
                    find_document_model_by_user_account_id_with_exclusive_lock(txn, user_account_id).await?;
                    if let Some(document) = document_option {
                        info!("update document for \"careers\" (user_account_id: {}, document_id: {}, career_model: {:?})", user_account_id, document.document_id, career_model);
                        let _ = insert_new_career_into_document(
                            INDEX_NAME,
                            document.document_id.to_string().as_str(),
                            career_model,
                            num_of_careers,
                            approved_time,
                            index_client
                        )
                        .await?;
                    } else {
                        // document_idとしてuser_account_idを利用
                        let document_id = user_account_id;
                        info!("create document for \"careers\" (user_account_id: {}, document_id: {}, career_model: {:?})", user_account_id, document_id, career_model);
                        let _ = insert_document(txn, user_account_id, document_id).await?;
                        let _ = add_new_document_with_career(
                            INDEX_NAME,
                            document_id.to_string().as_str(),
                            career_model,
                            num_of_careers,
                            approved_time,
                            index_client
                        )
                        .await?;
                    };

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
                    error!("failed to approve create_career_req: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(notification_email_address_option)
    }
}

fn generate_approved_create_career_req_active_model(
    model: create_career_req::Model,
    approved_time: DateTime<FixedOffset>,
    approver_email_address: String,
) -> approved_create_career_req::ActiveModel {
    approved_create_career_req::ActiveModel {
        appr_cre_career_req_id: NotSet,
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
        image1_file_name_without_ext: Set(model.image1_file_name_without_ext),
        image2_file_name_without_ext: Set(model.image2_file_name_without_ext),
        approved_at: Set(approved_time),
        approved_by: Set(approver_email_address),
    }
}

fn generate_career_active_model(model: create_career_req::Model) -> career::ActiveModel {
    career::ActiveModel {
        career_id: NotSet,
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
    }
}

async fn insert_document(
    txn: &DatabaseTransaction,
    user_account_id: i64,
    document_id: i64,
) -> Result<(), ErrRespStruct> {
    let document = document::ActiveModel {
        user_account_id: Set(user_account_id),
        document_id: Set(document_id),
    };
    let _ = document.insert(txn).await.map_err(|e| {
        error!(
            "failed to insert document (user_account_id: {}, document_id: {}): {}",
            user_account_id, document_id, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

async fn add_new_document_with_career(
    index_name: &str,
    document_id: &str,
    career_model: career::Model,
    num_of_careers: u64,
    current_time: DateTime<FixedOffset>,
    client: OpenSearch,
) -> Result<(), ErrRespStruct> {
    let employed = career_model.career_end_date.is_none();
    let years_of_service = if let Some(career_end_date) = career_model.career_end_date {
        calculate_years_of_service(career_model.career_start_date, career_end_date)
    } else {
        calculate_years_of_service(
            career_model.career_start_date,
            current_time.naive_local().date(),
        )
    };
    let new_document = json!({
        "user_account_id": career_model.user_account_id,
        "careers": [{
            "career_id": career_model.career_id,
            "company_name": career_model.company_name,
            "department_name": career_model.department_name,
            "office": career_model.office,
            "years_of_service": years_of_service,
            "employed": employed,
            "contract_type": career_model.contract_type,
            "profession": career_model.profession,
            "annual_income_in_man_yen": career_model.annual_income_in_man_yen,
            "is_manager": career_model.is_manager,
            "position_name": career_model.position_name,
            "is_new_graduate": career_model.is_new_graduate,
            "note": career_model.note,
        }],
        "num_of_careers": num_of_careers,
        "fee_per_hour_in_yen": null,
        "is_bank_account_registered": false,
        "rating": null,
        "num_of_rated": 0,
        "disabled": false
    });
    index_document(index_name, document_id, &new_document, &client)
        .await
        .map_err(|e| {
            error!(
                "failed to index new document with career (document_id: {}, career_id: {})",
                document_id, career_model.career_id
            );
            ErrRespStruct { err_resp: e }
        })?;
    Ok(())
}

fn calculate_years_of_service(from: NaiveDate, to: NaiveDate) -> i64 {
    let days_in_year = 365; // 1日の誤差（1年が365日か366日か）は、年という単位に対して無視して良いと判断し、365日固定で計算する
    let days_of_service = (to - from).num_days();
    days_of_service / days_in_year
}

async fn insert_new_career_into_document(
    index_name: &str,
    document_id: &str,
    career_model: career::Model,
    num_of_careers: u64,
    current_time: DateTime<FixedOffset>,
    client: OpenSearch,
) -> Result<(), ErrRespStruct> {
    let employed = career_model.career_end_date.is_none();
    let years_of_service = if let Some(career_end_date) = career_model.career_end_date {
        calculate_years_of_service(career_model.career_start_date, career_end_date)
    } else {
        calculate_years_of_service(
            career_model.career_start_date,
            current_time.naive_local().date(),
        )
    };
    let source = format!(
        "ctx._source.careers.add(params.career); ctx._source.num_of_careers = {}",
        num_of_careers
    );
    let script = json!({
        "script": {
            "source": source,
            "params": {
              "career": {
                "career_id": career_model.career_id,
                "company_name": career_model.company_name,
                "department_name": career_model.department_name,
                "office": career_model.office,
                "years_of_service": years_of_service,
                "employed": employed,
                "contract_type": career_model.contract_type,
                "profession": career_model.profession,
                "annual_income_in_man_yen": career_model.annual_income_in_man_yen,
                "is_manager": career_model.is_manager,
                "position_name": career_model.position_name,
                "is_new_graduate": career_model.is_new_graduate,
                "note": career_model.note,
              }
            }
        }
    });
    update_document(index_name, document_id, &script, &client)
        .await
        .map_err(|e| {
            error!(
                "failed to insert new career into document (document_id: {}, career_id: {})",
                document_id, career_model.career_id
            );
            ErrRespStruct { err_resp: e }
        })?;
    Ok(())
}

fn create_text() -> String {
    format!(
        r"職務経歴確認が完了し、職務経歴を登録致しました。

他のユーザーから相談を受けるには、職務経歴に加えて下記の二点の登録が必要となります。まだご登録されていない場合、下記の二点をご登録いただくようお願いします。
・相談料
・銀行口座

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        INQUIRY_EMAIL_ADDRESS.as_str()
    )
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::{DateTime, FixedOffset, NaiveDate, TimeZone};
    use common::{smtp::SYSTEM_EMAIL_ADDRESS, ErrResp, JAPANESE_TIME_ZONE};
    use once_cell::sync::Lazy;

    use crate::handlers::session::authentication::authenticated_handlers::tests::SendMailMock;

    use super::*;

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

    struct CreateCareerReqApprovalOperationMock {
        admin_email_address: String,
        user_option: Option<User>,
        create_career_req_mock: CreateCareerReqMock,
        approved_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl CreateCareerReqApprovalOperation for CreateCareerReqApprovalOperationMock {
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

        async fn approve_create_career_req(
            &self,
            user_account_id: i64,
            create_career_req_id: i64,
            approver_email_address: String,
            approved_time: DateTime<FixedOffset>,
        ) -> Result<Option<String>, ErrResp> {
            if let Some(user) = self.user_option.clone() {
                assert_eq!(user.user_account_id, user_account_id);
                assert_eq!(self.admin_email_address, approver_email_address);
                assert_eq!(
                    self.create_career_req_mock.create_career_req_id,
                    create_career_req_id
                );
                assert_eq!(self.approved_time, approved_time);
                Ok(Some(user.email_address))
            } else {
                Ok(None)
            }
        }
    }

    #[tokio::test]
    async fn handle_create_career_request_approval_success() {
        let admin_email_address = String::from("admin@test.com");
        let user_account_id = 432;
        let user_email_address = String::from("test@test.com");
        let user_option = Some(User {
            user_account_id,
            email_address: user_email_address.clone(),
        });
        let create_career_req_id = 53215;
        let create_career_req = CreateCareerReqMock {
            create_career_req_id,
            user_account_id,
        };
        let approval_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 4, 1, 21, 0, 40)
            .unwrap();
        let op_mock = CreateCareerReqApprovalOperationMock {
            admin_email_address: admin_email_address.clone(),
            user_option,
            create_career_req_mock: create_career_req,
            approved_time: approval_time,
        };
        let send_mail_mock = SendMailMock::new(
            user_email_address,
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );

        let result = handle_create_career_request_approval(
            admin_email_address,
            create_career_req_id,
            approval_time,
            op_mock,
            send_mail_mock,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(CreateCareerReqApprovalResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_create_career_request_approval_success_no_user_account_found() {
        let admin_email_address = String::from("admin@test.com");
        let user_account_id = 432;
        let user_email_address = String::from("test@test.com");
        let create_career_req_id = 53215;
        let create_career_req = CreateCareerReqMock {
            create_career_req_id,
            user_account_id,
        };
        let approval_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 4, 1, 21, 0, 40)
            .unwrap();
        let op_mock = CreateCareerReqApprovalOperationMock {
            admin_email_address: admin_email_address.clone(),
            user_option: None,
            create_career_req_mock: create_career_req,
            approved_time: approval_time,
        };
        let send_mail_mock = SendMailMock::new(
            user_email_address,
            SYSTEM_EMAIL_ADDRESS.to_string(),
            SUBJECT.to_string(),
            create_text(),
        );

        let result = handle_create_career_request_approval(
            admin_email_address,
            create_career_req_id,
            approval_time,
            op_mock,
            send_mail_mock,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(CreateCareerReqApprovalResult {}, resp.1 .0);
    }

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: i64,
    }

    #[derive(Debug)]
    struct Input {
        from: NaiveDate,
        to: NaiveDate,
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        vec![
            TestCase {
                name: "less 1 year".to_string(),
                input: Input {
                    from: NaiveDate::from_ymd_opt(2009, 4, 1).expect("failed to get NaiveDate"),
                    to: NaiveDate::from_ymd_opt(2010, 3, 31).expect("failed to get NaiveDate"),
                },
                expected: 0,
            },
            TestCase {
                name: "just 1 year".to_string(),
                input: Input {
                    from: NaiveDate::from_ymd_opt(2009, 4, 1).expect("failed to get NaiveDate"),
                    to: NaiveDate::from_ymd_opt(2010, 4, 1).expect("failed to get NaiveDate"),
                },
                expected: 1,
            },
            TestCase {
                name: "less 1 year (leap year)".to_string(),
                input: Input {
                    from: NaiveDate::from_ymd_opt(2011, 4, 1).expect("failed to get NaiveDate"),
                    to: NaiveDate::from_ymd_opt(2012, 3, 30).expect("failed to get NaiveDate"),
                },
                expected: 0,
            },
            TestCase {
                name: "just 1 year (leap year)".to_string(),
                input: Input {
                    from: NaiveDate::from_ymd_opt(2011, 4, 1).expect("failed to get NaiveDate"),
                    to: NaiveDate::from_ymd_opt(2012, 3, 31).expect("failed to get NaiveDate"),
                },
                expected: 1,
            },
            TestCase {
                name: "passed leap year 2 times".to_string(),
                input: Input {
                    from: NaiveDate::from_ymd_opt(2010, 4, 1).expect("failed to get NaiveDate"),
                    to: NaiveDate::from_ymd_opt(2019, 3, 30).expect("failed to get NaiveDate"),
                },
                expected: 9,
            },
            TestCase {
                name: "passed leap year 3 times".to_string(),
                input: Input {
                    from: NaiveDate::from_ymd_opt(2010, 4, 1).expect("failed to get NaiveDate"),
                    to: NaiveDate::from_ymd_opt(2020, 3, 29).expect("failed to get NaiveDate"),
                },
                expected: 10,
            },
        ]
    });

    #[test]
    fn calculate_years_of_service_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let years_of_service =
                calculate_years_of_service(test_case.input.from, test_case.input.to);
            let message = format!("test case \"{}\" failed", test_case.name.clone());
            assert_eq!(test_case.expected, years_of_service, "{}", message);
        }
    }
}
