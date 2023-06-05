// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::opensearch::INDEX_NAME;
use common::{ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, Set, TransactionError,
    TransactionTrait,
};
use opensearch::OpenSearch;
use serde::Deserialize;
use tracing::{error, info};

use crate::err::unexpected_err_resp;
use crate::handlers::session::authentication::authenticated_handlers::document_operation::find_document_model_by_user_account_id_with_exclusive_lock;
use crate::handlers::session::authentication::authenticated_handlers::user_account::update_disabled_on_document;
use crate::handlers::session::authentication::authenticated_handlers::user_account_operation::find_user_account_model_by_user_account_id_with_exclusive_lock;

use super::super::admin::Admin;
use super::{validate_account_id_is_positive, UserAccount, UserAccountRetrievalResult};

pub(crate) async fn post_enable_user_account_req(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
    Json(req): Json<EnableUserAccountReq>,
) -> RespResult<UserAccountRetrievalResult> {
    let op = EnableUserAccountReqOperationImpl { pool, index_client };
    handle_enable_user_account_req(req.user_account_id, &op).await
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct EnableUserAccountReq {
    user_account_id: i64,
}

async fn handle_enable_user_account_req(
    user_account_id: i64,
    op: &impl EnableUserAccountReqOperation,
) -> RespResult<UserAccountRetrievalResult> {
    validate_account_id_is_positive(user_account_id)?;

    let ua = op
        .enable_user_account_req(user_account_id, INDEX_NAME.to_string())
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
    async fn enable_user_account_req(
        &self,
        user_account_id: i64,
        index_name: String,
    ) -> Result<UserAccount, ErrResp>;
}

struct EnableUserAccountReqOperationImpl {
    pool: DatabaseConnection,
    index_client: OpenSearch,
}

#[async_trait]
impl EnableUserAccountReqOperation for EnableUserAccountReqOperationImpl {
    async fn enable_user_account_req(
        &self,
        user_account_id: i64,
        index_name: String,
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

                    let doc_option = find_document_model_by_user_account_id_with_exclusive_lock(txn, user_account_id).await?;
                    if let Some(doc) = doc_option {
                        info!("document (user_account_id: {}, document_id: {}) exists and set disabled to false on document", user_account_id, doc.document_id);
                        let document_id = doc.document_id.to_string();
                        update_disabled_on_document(&index_name, &document_id, false, index_client).await?;
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
