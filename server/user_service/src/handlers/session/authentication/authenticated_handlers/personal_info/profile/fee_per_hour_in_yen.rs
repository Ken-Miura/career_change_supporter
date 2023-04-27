// Copyright 2022 Ken Miura

use async_session::serde_json::json;
use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use common::opensearch::{index_document, update_document, INDEX_NAME};
use common::{ApiError, ErrResp, ErrRespStruct, RespResult};
use entity::consulting_fee;
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, QuerySelect, Set, TransactionError,
    TransactionTrait,
};
use opensearch::OpenSearch;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::authentication::authenticated_handlers::authenticated_users::verified_user::VerifiedUser;
use crate::handlers::session::authentication::authenticated_handlers::fee_per_hour_in_yen_range::{
    MAX_FEE_PER_HOUR_IN_YEN, MIN_FEE_PER_HOUR_IN_YEN,
};
use crate::util::document_operation::{
    find_document_model_by_user_account_id_with_shared_lock, insert_document,
};

pub(crate) async fn post_fee_per_hour_in_yen(
    VerifiedUser { user_info }: VerifiedUser,
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
    Json(fee): Json<Fee>,
) -> RespResult<FeePerHourInYenResult> {
    let op = SubmitFeePerHourInYenOperationImpl { pool, index_client };
    handle_fee_per_hour_in_yen_req(user_info.account_id, fee.fee_per_hour_in_yen, op).await
}

#[derive(Deserialize)]
pub(crate) struct Fee {
    #[serde(rename = "fee-per-hour-in-yen")]
    pub(crate) fee_per_hour_in_yen: i32,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct FeePerHourInYenResult {}

async fn handle_fee_per_hour_in_yen_req(
    account_id: i64,
    fee_per_hour_in_yen: i32,
    op: impl SubmitFeePerHourInYenOperation,
) -> RespResult<FeePerHourInYenResult> {
    if !(MIN_FEE_PER_HOUR_IN_YEN..=MAX_FEE_PER_HOUR_IN_YEN).contains(&fee_per_hour_in_yen) {
        error!(
            "illegal fee per hour in yen (account id: {}, fee:{}, min fee: {}, max fee: {})",
            account_id, fee_per_hour_in_yen, MIN_FEE_PER_HOUR_IN_YEN, MAX_FEE_PER_HOUR_IN_YEN
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::IllegalFeePerHourInYen as u32,
            }),
        ));
    }
    op.submit_fee_per_hour_in_yen(account_id, fee_per_hour_in_yen)
        .await?;
    Ok((StatusCode::OK, Json(FeePerHourInYenResult {})))
}

#[async_trait]
trait SubmitFeePerHourInYenOperation {
    async fn submit_fee_per_hour_in_yen(
        &self,
        account_id: i64,
        fee_per_hour_in_yen: i32,
    ) -> Result<(), ErrResp>;
}

struct SubmitFeePerHourInYenOperationImpl {
    pool: DatabaseConnection,
    index_client: OpenSearch,
}

#[async_trait]
impl SubmitFeePerHourInYenOperation for SubmitFeePerHourInYenOperationImpl {
    async fn submit_fee_per_hour_in_yen(
        &self,
        account_id: i64,
        fee_per_hour_in_yen: i32,
    ) -> Result<(), ErrResp> {
        let index_client = self.index_client.clone();
        let _ = self
            .pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let fee_option = consulting_fee::Entity::find_by_id(account_id)
                        .lock_exclusive()
                        .one(txn)
                        .await
                        .map_err(|e| {
                            error!(
                                "failed to find consulting_fee (account_id: {}): {}",
                                account_id, e
                            );
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;
                    if let Some(fee) = fee_option {
                        let mut active_model: consulting_fee::ActiveModel = fee.into();
                        active_model.fee_per_hour_in_yen = Set(fee_per_hour_in_yen);
                        active_model.update(txn).await.map_err(|e| {
                            error!(
                                "failed to update consulting_fee (account_id: {}): {}",
                                account_id, e
                            );
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;
                    } else {
                        let active_model = consulting_fee::ActiveModel {
                            user_account_id: Set(account_id),
                            fee_per_hour_in_yen: Set(fee_per_hour_in_yen),
                        };
                        active_model.insert(txn).await.map_err(|e| {
                            error!(
                                "failed to insert consulting_fee (account_id: {}): {}",
                                account_id, e
                            );
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;
                    }

                    let document_option =
                        find_document_model_by_user_account_id_with_shared_lock(txn, account_id).await?;
                    if let Some(document) = document_option {
                        let document_id = document.document_id;
                        info!("update document for \"fee_per_hour_in_yen\" (account_id: {}, document_id: {}, fee_per_hour_in_yen: {})", account_id, document_id, fee_per_hour_in_yen);
                        let _ = update_new_fee_per_hour_in_yen_on_document(
                            INDEX_NAME,
                            document_id.to_string().as_str(),
                            fee_per_hour_in_yen,
                            index_client
                        )
                        .await?;
                    } else {
                        // document_idとしてuser_account_idを利用
                        let document_id = account_id;
                        info!("create document for \"fee_per_hour_in_yen\" (account_id: {}, document_id: {}, fee_per_hour_in_yen: {})", account_id, document_id, fee_per_hour_in_yen);
                        let _ = insert_document(txn, account_id, document_id).await?;
                        let _ = add_new_fee_per_hour_in_yen_into_document(
                            INDEX_NAME,
                            document_id.to_string().as_str(),
                            account_id,
                            fee_per_hour_in_yen,
                            index_client
                        )
                        .await?;
                    };

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
                    error!("failed to submit fee_per_hour_in_yen: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

async fn update_new_fee_per_hour_in_yen_on_document(
    index_name: &str,
    document_id: &str,
    fee_per_hour_in_yen: i32,
    index_client: OpenSearch,
) -> Result<(), ErrRespStruct> {
    let value = format!("ctx._source.fee_per_hour_in_yen = {}", fee_per_hour_in_yen);
    let script = json!({
        "script": {
            "source": value
        }
    });
    update_document(index_name, document_id, &script, &index_client)
        .await
        .map_err(|e| {
            error!(
                "failed to update fee_per_hour_in_yen on document (document_id: {}, fee_per_hour_in_yen: {})",
                document_id, fee_per_hour_in_yen
            );
            ErrRespStruct { err_resp: e }
        })?;
    Ok(())
}

async fn add_new_fee_per_hour_in_yen_into_document(
    index_name: &str,
    document_id: &str,
    account_id: i64,
    fee_per_hour_in_yen: i32,
    index_client: OpenSearch,
) -> Result<(), ErrRespStruct> {
    let new_document = json!({
        "user_account_id": account_id,
        "careers": [],
        "fee_per_hour_in_yen": fee_per_hour_in_yen,
        "rating": null,
        "is_bank_account_registered": null
    });
    index_document(index_name, document_id, &new_document, &index_client)
        .await
        .map_err(|e| {
            error!(
                "failed to index new document with fee_per_hour_in_yen (document_id: {}, fee_per_hour_in_yen: {})",
                document_id, fee_per_hour_in_yen
            );
            ErrRespStruct { err_resp: e }
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use common::ErrResp;
    use hyper::StatusCode;

    use crate::{
        err::Code,
        handlers::session::authentication::authenticated_handlers::{
            fee_per_hour_in_yen_range::{MAX_FEE_PER_HOUR_IN_YEN, MIN_FEE_PER_HOUR_IN_YEN},
            personal_info::profile::fee_per_hour_in_yen::FeePerHourInYenResult,
        },
    };

    use super::{handle_fee_per_hour_in_yen_req, SubmitFeePerHourInYenOperation};

    struct SubmitFeePerHourInYenOperationMock {
        account_id: i64,
        fee_per_hour_in_yen: i32,
    }

    #[async_trait]
    impl SubmitFeePerHourInYenOperation for SubmitFeePerHourInYenOperationMock {
        async fn submit_fee_per_hour_in_yen(
            &self,
            account_id: i64,
            fee_per_hour_in_yen: i32,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.account_id, account_id);
            assert_eq!(self.fee_per_hour_in_yen, fee_per_hour_in_yen);
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_fee_per_hour_in_yen_req_success1() {
        let account_id = 4151;
        let fee_per_hour_in_yen = 4500;
        let op = SubmitFeePerHourInYenOperationMock {
            account_id,
            fee_per_hour_in_yen,
        };

        let result = handle_fee_per_hour_in_yen_req(account_id, fee_per_hour_in_yen, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(FeePerHourInYenResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_fee_per_hour_in_yen_req_success2() {
        let account_id = 4151;
        let fee_per_hour_in_yen = MIN_FEE_PER_HOUR_IN_YEN;
        let op = SubmitFeePerHourInYenOperationMock {
            account_id,
            fee_per_hour_in_yen,
        };

        let result = handle_fee_per_hour_in_yen_req(account_id, fee_per_hour_in_yen, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(FeePerHourInYenResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_fee_per_hour_in_yen_req_success3() {
        let account_id = 4151;
        let fee_per_hour_in_yen = MAX_FEE_PER_HOUR_IN_YEN;
        let op = SubmitFeePerHourInYenOperationMock {
            account_id,
            fee_per_hour_in_yen,
        };

        let result = handle_fee_per_hour_in_yen_req(account_id, fee_per_hour_in_yen, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(FeePerHourInYenResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_fee_per_hour_in_yen_req_fail_illegal_fee_per_hour_in_yen1() {
        let account_id = 4151;
        let fee_per_hour_in_yen = MIN_FEE_PER_HOUR_IN_YEN - 1;
        let op = SubmitFeePerHourInYenOperationMock {
            account_id,
            fee_per_hour_in_yen,
        };

        let result = handle_fee_per_hour_in_yen_req(account_id, fee_per_hour_in_yen, op).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::IllegalFeePerHourInYen as u32, resp.1 .0.code);
    }

    #[tokio::test]
    async fn handle_fee_per_hour_in_yen_req_fail_illegal_fee_per_hour_in_yen2() {
        let account_id = 4151;
        let fee_per_hour_in_yen = MAX_FEE_PER_HOUR_IN_YEN + 1;
        let op = SubmitFeePerHourInYenOperationMock {
            account_id,
            fee_per_hour_in_yen,
        };

        let result = handle_fee_per_hour_in_yen_req(account_id, fee_per_hour_in_yen, op).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::IllegalFeePerHourInYen as u32, resp.1 .0.code);
    }
}
