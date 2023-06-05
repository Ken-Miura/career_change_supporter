// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::opensearch::{delete_document, INDEX_NAME};
use common::{ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, ModelTrait, Set, TransactionError,
    TransactionTrait,
};
use opensearch::OpenSearch;
use serde::Deserialize;
use tracing::{error, info};

use crate::err::unexpected_err_resp;
use crate::handlers::session::authentication::authenticated_handlers::document_operation::find_document_model_by_user_account_id_with_exclusive_lock;
use crate::handlers::session::authentication::authenticated_handlers::user_account_operation::find_user_account_model_by_user_account_id_with_exclusive_lock;

use super::super::admin::Admin;
use super::{validate_account_id_is_positive, UserAccount, UserAccountRetrievalResult};

pub(crate) async fn post_disable_user_account_req(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
    Json(req): Json<DisableUserAccountReq>,
) -> RespResult<UserAccountRetrievalResult> {
    let op = DisableUserAccountReqOperationImpl { pool, index_client };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    handle_disable_user_account_req(req.user_account_id, current_date_time, &op).await
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct DisableUserAccountReq {
    user_account_id: i64,
}

async fn handle_disable_user_account_req(
    user_account_id: i64,
    current_date_time: DateTime<FixedOffset>,
    op: &impl DisableUserAccountReqOperation,
) -> RespResult<UserAccountRetrievalResult> {
    validate_account_id_is_positive(user_account_id)?;
    let ua = op
        .disable_user_account_req(user_account_id, INDEX_NAME.to_string(), current_date_time)
        .await?;
    Ok((
        StatusCode::OK,
        Json(UserAccountRetrievalResult {
            user_account: Some(ua),
        }),
    ))
}

#[async_trait]
trait DisableUserAccountReqOperation {
    async fn disable_user_account_req(
        &self,
        user_account_id: i64,
        index_name: String,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<UserAccount, ErrResp>;
}

struct DisableUserAccountReqOperationImpl {
    pool: DatabaseConnection,
    index_client: OpenSearch,
}

#[async_trait]
impl DisableUserAccountReqOperation for DisableUserAccountReqOperationImpl {
    async fn disable_user_account_req(
        &self,
        user_account_id: i64,
        index_name: String,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<UserAccount, ErrResp> {
        let index_client = self.index_client.clone();
        let result = self.pool
            .transaction::<_, entity::user_account::Model, ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let user_model = find_user_model_with_exclusive_lock(user_account_id, txn).await?;

                    let mut user_active_model: entity::user_account::ActiveModel = user_model.into();
                    user_active_model.disabled_at = Set(Some(current_date_time));
                    let result = user_active_model.update(txn).await.map_err(|e| {
                        error!("failed to update disabled_at in user_account (user_account_id: {}): {}", user_account_id, e);
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let doc_option = find_document_model_by_user_account_id_with_exclusive_lock(txn, user_account_id).await?;
                    if let Some(doc) = doc_option {
                        info!("document (user_account_id: {}, document_id: {}) exists and will be deleted", user_account_id, doc.document_id);
                        let document_id = doc.document_id.to_string();
                        let _ = doc.delete(txn).await.map_err(|e| {
                            error!("failed to delete document (user_account_id: {}): {}", user_account_id, e);
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;
                        delete_document(index_name.as_str(), document_id.as_str(), &index_client).await.map_err(|e|{
                            error!(
                              "failed to delete document (user_account_id: {}, index_name: {}, document_id: {}) from Opensearch",
                              user_account_id, index_name, document_id
                            );
                            ErrRespStruct {
                              err_resp: e,
                            }
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
                    error!("failed to disable_user_account_req: {}", err_resp_struct);
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

    use chrono::TimeZone;

    use crate::err::Code;

    use super::*;

    struct DisableUserAccountReqOperationMock {
        user_account_id: i64,
        current_date_time: DateTime<FixedOffset>,
        user_account: UserAccount,
    }

    #[async_trait]
    impl DisableUserAccountReqOperation for DisableUserAccountReqOperationMock {
        async fn disable_user_account_req(
            &self,
            user_account_id: i64,
            index_name: String,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<UserAccount, ErrResp> {
            assert_eq!(self.user_account_id, user_account_id);
            assert_eq!(INDEX_NAME.to_string(), index_name);
            assert_eq!(self.current_date_time, current_date_time);
            Ok(self.user_account.clone())
        }
    }

    fn create_dummy_user_account(user_account_id: i64) -> UserAccount {
        UserAccount {
            user_account_id,
            email_address: "test0@test.com".to_string(),
            last_login_time: Some("2023-04-15T14:12:53.4242+09:00 ".to_string()),
            created_at: "2023-04-13T14:12:53.4242+09:00 ".to_string(),
            mfa_enabled_at: None,
            disabled_at: Some("2023-05-15T14:12:53.4242+09:00 ".to_string()),
        }
    }

    #[tokio::test]
    async fn handle_disable_user_account_req_success() {
        let user_account_id = 57301;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 4, 5, 21, 0, 40)
            .unwrap();
        let user_account = create_dummy_user_account(user_account_id);
        let op_mock = DisableUserAccountReqOperationMock {
            user_account_id,
            current_date_time,
            user_account: user_account.clone(),
        };

        let result =
            handle_disable_user_account_req(user_account_id, current_date_time, &op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(resp.0, StatusCode::OK);
        assert_eq!(resp.1 .0.user_account, Some(user_account))
    }

    #[tokio::test]
    async fn handle_disable_user_account_req_fail_user_account_id_is_zero() {
        let user_account_id = 0;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 4, 5, 21, 0, 40)
            .unwrap();
        let op_mock = DisableUserAccountReqOperationMock {
            user_account_id,
            current_date_time,
            user_account: create_dummy_user_account(user_account_id),
        };

        let result =
            handle_disable_user_account_req(user_account_id, current_date_time, &op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn handle_disable_user_account_req_fail_user_account_id_is_negative() {
        let user_account_id = -1;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 4, 5, 21, 0, 40)
            .unwrap();
        let op_mock = DisableUserAccountReqOperationMock {
            user_account_id,
            current_date_time,
            user_account: create_dummy_user_account(user_account_id),
        };

        let result =
            handle_disable_user_account_req(user_account_id, current_date_time, &op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }
}
