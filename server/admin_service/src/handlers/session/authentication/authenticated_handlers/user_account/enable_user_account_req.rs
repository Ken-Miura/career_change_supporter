// Copyright 2023 Ken Miura

use async_session::serde_json::{json, Value};
use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::opensearch::{index_document, INDEX_NAME};
use common::rating::calculate_average_rating;
use common::{ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, Set, TransactionError,
    TransactionTrait,
};
use opensearch::OpenSearch;
use serde::Deserialize;
use tracing::{error, info};

use crate::err::unexpected_err_resp;
use crate::handlers::session::authentication::authenticated_handlers::calculate_years_of_service;
use crate::handlers::session::authentication::authenticated_handlers::user_account_operation::find_user_account_model_by_user_account_id_with_exclusive_lock;

use super::super::admin::Admin;
use super::{validate_account_id_is_positive, Career, UserAccount, UserAccountRetrievalResult};

pub(crate) async fn post_enable_user_account_req(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
    Json(req): Json<EnableUserAccountReq>,
) -> RespResult<UserAccountRetrievalResult> {
    let op = EnableUserAccountReqOperationImpl { pool, index_client };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    handle_enable_user_account_req(req.user_account_id, current_date_time, &op).await
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct EnableUserAccountReq {
    user_account_id: i64,
}

async fn handle_enable_user_account_req(
    user_account_id: i64,
    current_date_time: DateTime<FixedOffset>,
    op: &impl EnableUserAccountReqOperation,
) -> RespResult<UserAccountRetrievalResult> {
    validate_account_id_is_positive(user_account_id)?;
    let careers = op.get_careers(user_account_id).await?;
    let fee_per_hour_in_yen = op.get_fee_per_hour_in_yen(user_account_id).await?;
    let tenant_id = op.get_tenant_id(user_account_id).await?;
    let ratings = op.get_consultant_rating_info(user_account_id).await?; // user_account_id == consultant_id

    let doc_value = generate_document_value(
        user_account_id,
        careers,
        fee_per_hour_in_yen,
        tenant_id,
        ratings,
        current_date_time,
    );

    let ua = op
        .enable_user_account_req(user_account_id, INDEX_NAME.to_string(), doc_value)
        .await?;
    Ok((
        StatusCode::OK,
        Json(UserAccountRetrievalResult {
            user_account: Some(ua),
        }),
    ))
}

#[async_trait]
trait EnableUserAccountReqOperation {
    async fn get_careers(&self, user_account_id: i64) -> Result<Vec<Career>, ErrResp>;

    async fn get_fee_per_hour_in_yen(&self, user_account_id: i64) -> Result<Option<i32>, ErrResp>;

    async fn get_tenant_id(&self, user_account_id: i64) -> Result<Option<String>, ErrResp>;

    async fn get_consultant_rating_info(&self, consultant_id: i64) -> Result<Vec<i16>, ErrResp>;

    async fn enable_user_account_req(
        &self,
        user_account_id: i64,
        index_name: String,
        doc_value: Option<Value>,
    ) -> Result<UserAccount, ErrResp>;
}

struct EnableUserAccountReqOperationImpl {
    pool: DatabaseConnection,
    index_client: OpenSearch,
}

#[async_trait]
impl EnableUserAccountReqOperation for EnableUserAccountReqOperationImpl {
    async fn get_careers(&self, user_account_id: i64) -> Result<Vec<Career>, ErrResp> {
        super::get_careers(user_account_id, &self.pool).await
    }

    async fn get_fee_per_hour_in_yen(&self, user_account_id: i64) -> Result<Option<i32>, ErrResp> {
        super::get_fee_per_hour_in_yen(user_account_id, &self.pool).await
    }

    async fn get_tenant_id(&self, user_account_id: i64) -> Result<Option<String>, ErrResp> {
        super::get_tenant_id(user_account_id, &self.pool).await
    }

    async fn get_consultant_rating_info(&self, consultant_id: i64) -> Result<Vec<i16>, ErrResp> {
        super::get_consultant_rating_info(consultant_id, &self.pool).await
    }

    async fn enable_user_account_req(
        &self,
        user_account_id: i64,
        index_name: String,
        doc_value: Option<Value>,
    ) -> Result<UserAccount, ErrResp> {
        let index_client = self.index_client.clone();
        let result = self.pool
            .transaction::<_, entity::user_account::Model, ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let user_model = find_user_model_with_exclusive_lock(user_account_id, txn).await?;

                    let mut user_active_model: entity::user_account::ActiveModel = user_model.into();
                    user_active_model.disabled_at = Set(None);
                    let result = user_active_model.update(txn).await.map_err(|e| {
                        error!("failed to update disabled_at in user_account (user_account_id: {}): {}", user_account_id, e);
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    if let Some(dv) = doc_value {
                      let document_id = user_account_id; // document_idとしてuser_account_idを利用
                      info!("create document (user_account_id: {}, document_id: {}) on enabling user account", user_account_id, document_id);
                      let _ = insert_document(txn, user_account_id, document_id).await?;
                      index_document(index_name.as_str(), document_id.to_string().as_str(), &dv, &index_client)
                        .await
                        .map_err(|e| {
                          error!(
                              "failed to index new document (user_account_id: {}, document_id: {})",
                              user_account_id, document_id
                          );
                          ErrRespStruct { err_resp: e }
                        })?;
                    }

                    Ok(result)
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(db_err) => {
                    error!("connection error: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    error!("failed to enable_user_account_req: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;

        Ok(UserAccount {
            user_account_id: result.user_account_id,
            email_address: result.email_address,
            last_login_time: result
                .last_login_time
                .map(|m| m.with_timezone(&(*JAPANESE_TIME_ZONE)).to_rfc3339()),
            created_at: result
                .created_at
                .with_timezone(&(*JAPANESE_TIME_ZONE))
                .to_rfc3339(),
            mfa_enabled_at: result
                .mfa_enabled_at
                .map(|m| m.with_timezone(&(*JAPANESE_TIME_ZONE)).to_rfc3339()),
            disabled_at: result
                .disabled_at
                .map(|m| m.with_timezone(&(*JAPANESE_TIME_ZONE)).to_rfc3339()),
        })
    }
}

async fn find_user_model_with_exclusive_lock(
    user_account_id: i64,
    txn: &DatabaseTransaction,
) -> Result<entity::user_account::Model, ErrRespStruct> {
    let user_model =
        find_user_account_model_by_user_account_id_with_exclusive_lock(txn, user_account_id)
            .await?;
    let user_model = user_model.ok_or_else(|| {
        error!(
            "failed to find user_account (user_account_id: {})",
            user_account_id
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(user_model)
}

fn generate_document_value(
    user_account_id: i64,
    careers: Vec<Career>,
    fee_per_hour_in_yen: Option<i32>,
    tenant_id: Option<String>,
    ratings: Vec<i16>,
    current_date_time: DateTime<FixedOffset>,
) -> Option<Value> {
    if careers.is_empty()
        && fee_per_hour_in_yen.is_none()
        && tenant_id.is_none()
        && ratings.is_empty()
    {
        return None;
    }
    let num_of_careers = careers.len();
    let career_values = generate_career_values(careers, current_date_time);
    let is_bank_account_registered = tenant_id.is_some();
    let num_of_rated = ratings.len();
    let average_rating = calculate_average_rating(ratings);
    Some(json!({
        "user_account_id": user_account_id,
        "careers": career_values,
        "num_of_careers": num_of_careers,
        "fee_per_hour_in_yen": fee_per_hour_in_yen,
        "is_bank_account_registered": is_bank_account_registered,
        "rating": average_rating,
        "num_of_rated": num_of_rated
    }))
}

fn generate_career_values(
    careers: Vec<Career>,
    current_date_time: DateTime<FixedOffset>,
) -> Vec<Value> {
    let mut values = Vec::with_capacity(careers.len());
    for c in careers {
        let years_of_service = if let Some(career_end_date) = c.career_end_date {
            calculate_years_of_service(c.career_start_date, career_end_date)
        } else {
            calculate_years_of_service(c.career_start_date, current_date_time.naive_local().date())
        };
        let employed = c.career_end_date.is_none();
        let value = json!({
            "career_id": c.career_id,
            "company_name": c.company_name,
            "department_name": c.department_name,
            "office": c.office,
            "years_of_service": years_of_service,
            "employed": employed,
            "contract_type": c.contract_type,
            "profession": c.profession,
            "annual_income_in_man_yen": c.annual_income_in_man_yen,
            "is_manager": c.is_manager,
            "position_name": c.position_name,
            "is_new_graduate": c.is_new_graduate,
            "note": c.note,
        });
        values.push(value);
    }
    values
}

async fn insert_document(
    txn: &DatabaseTransaction,
    user_account_id: i64,
    document_id: i64,
) -> Result<(), ErrRespStruct> {
    let document = entity::document::ActiveModel {
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

#[cfg(test)]
mod tests {

    // use chrono::TimeZone;

    // use crate::err::Code;

    // use super::*;

    // struct EnableUserAccountReqOperationMock {
    //     user_account_id: i64,
    //     current_date_time: DateTime<FixedOffset>,
    //     user_account: UserAccount,
    // }

    // #[async_trait]
    // impl EnableUserAccountReqOperation for EnableUserAccountReqOperationMock {
    //     async fn enable_user_account_req(
    //         &self,
    //         user_account_id: i64,
    //         index_name: String,
    //         current_date_time: DateTime<FixedOffset>,
    //     ) -> Result<UserAccount, ErrResp> {
    //         assert_eq!(self.user_account_id, user_account_id);
    //         assert_eq!(INDEX_NAME.to_string(), index_name);
    //         assert_eq!(self.current_date_time, current_date_time);
    //         Ok(self.user_account.clone())
    //     }
    // }

    // fn create_dummy_user_account(user_account_id: i64) -> UserAccount {
    //     UserAccount {
    //         user_account_id,
    //         email_address: "test0@test.com".to_string(),
    //         last_login_time: Some("2023-04-15T14:12:53.4242+09:00 ".to_string()),
    //         created_at: "2023-04-13T14:12:53.4242+09:00 ".to_string(),
    //         mfa_enabled_at: None,
    //         disabled_at: Some("2023-05-15T14:12:53.4242+09:00 ".to_string()),
    //     }
    // }

    // #[tokio::test]
    // async fn handle_enable_user_account_req_success() {
    //     let user_account_id = 57301;
    //     let current_date_time = JAPANESE_TIME_ZONE
    //         .with_ymd_and_hms(2022, 4, 5, 21, 0, 40)
    //         .unwrap();
    //     let user_account = create_dummy_user_account(user_account_id);
    //     let op_mock = EnableUserAccountReqOperationMock {
    //         user_account_id,
    //         current_date_time,
    //         user_account: user_account.clone(),
    //     };

    //     let result =
    //         handle_enable_user_account_req(user_account_id, current_date_time, &op_mock).await;

    //     let resp = result.expect("failed to get Ok");
    //     assert_eq!(resp.0, StatusCode::OK);
    //     assert_eq!(resp.1 .0.user_account, Some(user_account))
    // }

    // #[tokio::test]
    // async fn handle_enable_user_account_req_fail_user_account_id_is_zero() {
    //     let user_account_id = 0;
    //     let current_date_time = JAPANESE_TIME_ZONE
    //         .with_ymd_and_hms(2022, 4, 5, 21, 0, 40)
    //         .unwrap();
    //     let op_mock = EnableUserAccountReqOperationMock {
    //         user_account_id,
    //         current_date_time,
    //         user_account: create_dummy_user_account(user_account_id),
    //     };

    //     let result =
    //         handle_enable_user_account_req(user_account_id, current_date_time, &op_mock).await;

    //     let resp = result.expect_err("failed to get Err");
    //     assert_eq!(resp.0, StatusCode::BAD_REQUEST);
    //     assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    // }

    // #[tokio::test]
    // async fn handle_enable_user_account_req_fail_user_account_id_is_negative() {
    //     let user_account_id = -1;
    //     let current_date_time = JAPANESE_TIME_ZONE
    //         .with_ymd_and_hms(2022, 4, 5, 21, 0, 40)
    //         .unwrap();
    //     let op_mock = EnableUserAccountReqOperationMock {
    //         user_account_id,
    //         current_date_time,
    //         user_account: create_dummy_user_account(user_account_id),
    //     };

    //     let result =
    //         handle_enable_user_account_req(user_account_id, current_date_time, &op_mock).await;

    //     let resp = result.expect_err("failed to get Err");
    //     assert_eq!(resp.0, StatusCode::BAD_REQUEST);
    //     assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    // }
}
