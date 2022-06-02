// Copyright 2022 Ken Miura

use async_session::serde_json::json;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use axum::{extract::Query, Extension};
use common::opensearch::{update_document, INDEX_NAME, OPENSEARCH_ENDPOINT_URI};
use common::{ApiError, ErrResp};
use common::{ErrRespStruct, RespResult};
use entity::sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, TransactionError,
    TransactionTrait,
};
use entity::{career, document};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::util::session::User;

pub(crate) async fn career(
    User { account_id }: User,
    param: Query<DeleteCareerQueryParam>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<DeleteCareerResult> {
    let param = param.0;
    let op = DeleteCareerOperationImpl::new(pool);
    handle_career_req(account_id, param.career_id, op).await
}

#[derive(Deserialize)]
pub(crate) struct DeleteCareerQueryParam {
    pub(crate) career_id: i64,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct DeleteCareerResult {}

async fn handle_career_req(
    account_id: i64,
    career_id: i64,
    op: impl DeleteCareerOperation,
) -> RespResult<DeleteCareerResult> {
    // 任意の職務経歴の削除を防ぐため、必ずログインユーザーのアカウントIDに紐付いた職務経歴かチェック
    let career_ids = op.filter_career_ids_by_account_id(account_id).await?;
    if !career_ids.contains(&career_id) {
        error!(
            "No career associated with user account found (account_id: {}, career_id: {})",
            account_id, career_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoCareerToHandleFound as u32,
            }),
        ));
    }
    let _ = op.delete_career(account_id, career_id).await?;
    Ok((StatusCode::OK, Json(DeleteCareerResult {})))
}

#[async_trait]
trait DeleteCareerOperation {
    async fn filter_career_ids_by_account_id(&self, account_id: i64) -> Result<Vec<i64>, ErrResp>;
    async fn delete_career(&self, account_id: i64, career_id: i64) -> Result<(), ErrResp>;
}

struct DeleteCareerOperationImpl {
    pool: DatabaseConnection,
}

impl DeleteCareerOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeleteCareerOperation for DeleteCareerOperationImpl {
    async fn filter_career_ids_by_account_id(&self, account_id: i64) -> Result<Vec<i64>, ErrResp> {
        let models = career::Entity::find()
            .filter(career::Column::UserAccountId.eq(account_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter career (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| m.career_id)
            .collect::<Vec<i64>>())
    }

    async fn delete_career(&self, account_id: i64, career_id: i64) -> Result<(), ErrResp> {
        let _ = self
            .pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let document_option = document::Entity::find_by_id(account_id)
                        .lock_exclusive()
                        .one(txn)
                        .await
                        .map_err(|e| {
                            error!(
                                "failed to find document (user_account_id: {}): {}",
                                account_id, e
                            );
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;
                    let document = document_option.ok_or_else(|| {
                        error!("no document found (user_account_id: {})", account_id);
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let _ = career::Entity::delete_by_id(career_id)
                        .exec(txn)
                        .await
                        .map_err(|e| {
                            error!("failed to delete career (career_id: {}): {}", career_id, e);
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;

                    let document_id = document.document_id.to_string();
                    let _ = remove_career_from_document(
                        &OPENSEARCH_ENDPOINT_URI,
                        INDEX_NAME,
                        document_id.as_str(),
                        career_id,
                    )
                    .await?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(db_err) => {
                    error!("connection error: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    error!("failed to delete career: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;

        Ok(())
    }
}

async fn remove_career_from_document(
    endpoint_uri: &str,
    index_name: &str,
    document_id: &str,
    career_id: i64,
) -> Result<(), ErrRespStruct> {
    let script = json!({
        "script": {
            "source": "ctx._source.careers.removeIf(career -> career.career_id == params.career_id)",
            "params": {
                "career_id": career_id
            }
        }
    });
    let _ = update_document(endpoint_uri, index_name, document_id, &script)
        .await
        .map_err(|e| {
            error!(
                "failed to remove career from document (document_id: {}, career_id: {})",
                document_id, career_id
            );
            ErrRespStruct { err_resp: e }
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::ErrResp;

    use crate::err::Code;

    use super::{handle_career_req, DeleteCareerOperation, DeleteCareerResult};

    struct DeleteCareerOperationMock {
        account_id: i64,
        career_ids: Vec<i64>,
    }

    #[async_trait]
    impl DeleteCareerOperation for DeleteCareerOperationMock {
        async fn filter_career_ids_by_account_id(
            &self,
            account_id: i64,
        ) -> Result<Vec<i64>, ErrResp> {
            assert_eq!(self.account_id, account_id);
            Ok(self.career_ids.clone())
        }

        async fn delete_career(&self, account_id: i64, career_id: i64) -> Result<(), ErrResp> {
            assert_eq!(self.account_id, account_id);
            assert!(self.career_ids.contains(&career_id));
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_career_req_success() {
        let account_id = 432;
        let career1_id = 5124;
        let career2_id = 5125;
        let career3_id = 5126;
        let career_ids = vec![career1_id, career2_id, career3_id];
        let op = DeleteCareerOperationMock {
            account_id,
            career_ids,
        };

        let result = handle_career_req(account_id, career1_id, op).await;

        let resp = result.expect("faile to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(DeleteCareerResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_career_req_fail_no_career_to_handle_found() {
        let account_id = 432;
        let career1_id = 5124;
        let career2_id = 5125;
        let career3_id = 5126;
        let career_ids = vec![career1_id, career2_id, career3_id];
        let op = DeleteCareerOperationMock {
            account_id,
            career_ids,
        };
        let dummy_career_id = 41;
        assert_ne!(dummy_career_id, career1_id);
        assert_ne!(dummy_career_id, career2_id);
        assert_ne!(dummy_career_id, career3_id);

        let result = handle_career_req(account_id, dummy_career_id, op).await;

        let resp = result.expect_err("faile to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::NoCareerToHandleFound as u32, resp.1 .0.code);
    }
}
