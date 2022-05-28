// Copyright 2022 Ken Miura

use async_session::serde_json::json;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{
    opensearch::{INDEX_NAME, OPENSEARCH_ENDPOINT_URI},
    smtp::{
        SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SOCKET_FOR_SMTP_SERVER, SYSTEM_EMAIL_ADDRESS,
    },
    ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE, WEB_SITE_NAME,
};

use axum::extract::Extension;
use axum::http::StatusCode;
use entity::{
    admin_account, approved_create_career_req, career, create_career_req, document,
    sea_orm::{
        ActiveModelTrait, ActiveValue::NotSet, DatabaseConnection, DatabaseTransaction,
        EntityTrait, QuerySelect, Set, TransactionError, TransactionTrait,
    },
};
use once_cell::sync::Lazy;
use opensearch::{http::transport::Transport, IndexParts, OpenSearch, UpdateParts};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    err::{unexpected_err_resp, Code},
    util::{find_user_model_by_user_account_id, session::Admin},
};

static SUBJECT: Lazy<String> = Lazy::new(|| format!("[{}] 職務経歴確認完了通知", WEB_SITE_NAME));

pub(crate) async fn post_create_career_request_approval(
    Admin { account_id }: Admin, // 認証されていることを保証するために必須のパラメータ
    Json(create_career_req_approval): Json<CreateCareerReqApproval>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<CreateCareerReqApprovalResult> {
    let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
    let op = CreateCareerReqApprovalOperationImpl { pool };
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
    handle_create_career_request_approval(
        account_id,
        create_career_req_approval.create_career_req_id,
        current_date_time,
        op,
        smtp_client,
    )
    .await
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateCareerReqApproval {
    pub(crate) create_career_req_id: i64,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateCareerReqApprovalResult {}

async fn handle_create_career_request_approval(
    admin_account_id: i64,
    create_career_req_id: i64,
    approved_time: DateTime<FixedOffset>,
    op: impl CreateCareerReqApprovalOperation,
    send_mail: impl SendMail,
) -> RespResult<CreateCareerReqApprovalResult> {
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

    let approved_user = op
        .approve_create_career_req(
            user_account_id,
            create_career_req_id,
            admin_email_address,
            approved_time,
        )
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

    let _ = async move {
        send_mail.send_mail(
            &user_email_address,
            SYSTEM_EMAIL_ADDRESS,
            &SUBJECT,
            create_text().as_str(),
        )
    }
    .await?;

    Ok((StatusCode::OK, Json(CreateCareerReqApprovalResult {})))
}

#[async_trait]
trait CreateCareerReqApprovalOperation {
    async fn get_admin_email_address_by_admin_account_id(
        &self,
        admin_account_id: i64,
    ) -> Result<Option<String>, ErrResp>;

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
}

#[async_trait]
impl CreateCareerReqApprovalOperation for CreateCareerReqApprovalOperationImpl {
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

    async fn approve_create_career_req(
        &self,
        user_account_id: i64,
        create_career_req_id: i64,
        approver_email_address: String,
        approved_time: DateTime<FixedOffset>,
    ) -> Result<Option<String>, ErrResp> {
        let notification_email_address_option = self
            .pool
            .transaction::<_, Option<String>, ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let user_option =
                        find_user_model_by_user_account_id(txn, user_account_id).await?;
                    let user = match user_option {
                        Some(m) => m,
                        None => return Ok(None),
                    };

                    let req = find_create_career_req_model_by_create_career_req_id(
                        txn,
                        create_career_req_id,
                    )
                    .await?;

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
                            "failed to insert approved_create_career_req (user_account_id: {}): {}",
                            user_account_id, e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let _ = create_career_req::Entity::delete_by_id(user_account_id)
                        .exec(txn)
                        .await
                        .map_err(|e| {
                            error!(
                                "failed to delete create_career_req (user_account_id: {}): {}",
                                user_account_id, e
                            );
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;

                    let document_option =
                        find_document_model_by_user_account_id(txn, user_account_id).await?;
                    if let Some(document) = document_option {
                        let _ = insert_new_career_into_document(
                            &OPENSEARCH_ENDPOINT_URI,
                            INDEX_NAME,
                            document.document_id.to_string().as_str(),
                            career_model,
                        )
                        .await?;
                    } else {
                        // document_idとしてuser_account_idを利用
                        let document_id = user_account_id;
                        let _ = insert_document(txn, user_account_id, document_id).await?;
                        let _ = add_new_document_with_career(
                            &OPENSEARCH_ENDPOINT_URI,
                            INDEX_NAME,
                            document_id.to_string().as_str(),
                            career_model,
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

async fn find_create_career_req_model_by_create_career_req_id(
    txn: &DatabaseTransaction,
    create_career_req_id: i64,
) -> Result<create_career_req::Model, ErrRespStruct> {
    let req_option = create_career_req::Entity::find_by_id(create_career_req_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find create_career_req (create_career_req_id: {}): {}",
                create_career_req_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    let req = req_option.ok_or_else(|| {
        error!(
            "no create_career_req (create_career_req_id: {}) found",
            create_career_req_id
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(req)
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

async fn find_document_model_by_user_account_id(
    txn: &DatabaseTransaction,
    user_account_id: i64,
) -> Result<Option<document::Model>, ErrRespStruct> {
    let doc_option = document::Entity::find_by_id(user_account_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find document (user_account_id: {}): {}",
                user_account_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(doc_option)
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
    endpoint_uri: &str,
    index_name: &str,
    document_id: &str,
    career_model: career::Model,
) -> Result<(), ErrRespStruct> {
    let transport = Transport::single_node(endpoint_uri).map_err(|e| {
        error!(
            "failed to struct transport (endpoint_uri: {}): {}",
            endpoint_uri, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    let client = OpenSearch::new(transport);

    let new_document = json!({
        "user_account_id": career_model.user_account_id,
        "careers": [{
            "career_id": career_model.career_id,
            "company_name": career_model.company_name,
            "department_name": career_model.department_name,
            "office": career_model.office,
            "career_start_date": career_model.career_start_date,
            "career_end_date": career_model.career_end_date,
            "contract_type": career_model.contract_type,
            "profession": career_model.profession,
            "annual_income_in_man_yen": career_model.annual_income_in_man_yen,
            "is_manager": career_model.is_manager,
            "position_name": career_model.position_name,
            "is_new_graduate": career_model.is_manager,
            "note": career_model.note,
        }],
        "fee_per_hour_in_yen": null,
        "rating": null,
        "is_bank_account_registered": null
    });

    let response = client
        .index(IndexParts::IndexId(index_name, document_id))
        .body(new_document.clone())
        .send()
        .await
        .map_err(|e| {
            error!(
                "failed to index document (index_name: {}, document_id: {}, document: {}): {}",
                index_name, document_id, new_document, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    let status_code = response.status_code();
    if !status_code.is_success() {
        error!("failed to request index (response: {:?})", response);
        return Err(ErrRespStruct {
            err_resp: unexpected_err_resp(),
        });
    }

    Ok(())
}

async fn insert_new_career_into_document(
    endpoint_uri: &str,
    index_name: &str,
    document_id: &str,
    career_model: career::Model,
) -> Result<(), ErrRespStruct> {
    let transport = Transport::single_node(endpoint_uri).map_err(|e| {
        error!(
            "failed to struct transport (endpoint_uri: {}): {}",
            endpoint_uri, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    let client = OpenSearch::new(transport);

    let script = json!({
        "script": {
            "source": "ctx._source.careers.add(params.career)",
            "params": {
              "career": {
                "career_id": career_model.career_id,
                "company_name": career_model.company_name,
                "department_name": career_model.department_name,
                "office": career_model.office,
                "career_start_date": career_model.career_start_date,
                "career_end_date": career_model.career_end_date,
                "contract_type": career_model.contract_type,
                "profession": career_model.profession,
                "annual_income_in_man_yen": career_model.annual_income_in_man_yen,
                "is_manager": career_model.is_manager,
                "position_name": career_model.position_name,
                "is_new_graduate": career_model.is_manager,
                "note": career_model.note,
              }
            }
        }
    });

    let response = client
        .update(UpdateParts::IndexId(index_name, document_id))
        .body(script.clone())
        .send()
        .await
        .map_err(|e| {
            error!(
                "failed to index document (index_name: {}, document_id: {}, script: {}): {}",
                index_name, document_id, script, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    let status_code = response.status_code();
    if !status_code.is_success() {
        error!("failed to request index (response: {:?})", response);
        return Err(ErrRespStruct {
            err_resp: unexpected_err_resp(),
        });
    }

    Ok(())
}

fn create_text() -> String {
    // TODO: 文面の調整
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
        INQUIRY_EMAIL_ADDRESS
    )
}

#[cfg(test)]
mod tests {}
