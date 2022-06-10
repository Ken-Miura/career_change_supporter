// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{Extension, Json};
use common::{ApiError, ErrResp, ErrRespStruct, RespResult};
use entity::consulting_fee;
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, QuerySelect, Set, TransactionError,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::util::session::User;

const MIN_FEE_PER_HOUR_IN_YEN: i32 = 3000;
const MAX_FEE_PER_HOUR_IN_YEN: i32 = 50000;

pub(crate) async fn post_fee_per_hour_in_yen(
    User { account_id }: User,
    Json(fee): Json<Fee>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<FeePerHourInYenResult> {
    let op = SubmitFeePerHourYenOperationImpl { pool };
    handle_fee_per_hour_yen_req(account_id, fee.fee_per_hour_in_yen, op).await
}

#[derive(Deserialize)]
pub(crate) struct Fee {
    #[serde(rename = "fee-per-hour-in-yen")]
    fee_per_hour_in_yen: i32,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct FeePerHourInYenResult {}

async fn handle_fee_per_hour_yen_req(
    account_id: i64,
    fee_per_hour_in_yen: i32,
    op: impl SubmitFeePerHourYenOperation,
) -> RespResult<FeePerHourInYenResult> {
    let identity_exists = op.check_if_identity_exists(account_id).await?;
    if !identity_exists {
        error!("identity is not registered (account id: {})", account_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoIdentityRegistered as u32,
            }),
        ));
    }
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
    let _ = op
        .submit_fee_per_hour_in_yen(account_id, fee_per_hour_in_yen)
        .await?;
    Ok((StatusCode::OK, Json(FeePerHourInYenResult {})))
}

#[async_trait]
trait SubmitFeePerHourYenOperation {
    /// Identityが存在するか確認する。存在する場合、trueを返す。そうでない場合、falseを返す。
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
    async fn submit_fee_per_hour_in_yen(
        &self,
        account_id: i64,
        fee_per_hour_in_yen: i32,
    ) -> Result<(), ErrResp>;
}

struct SubmitFeePerHourYenOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl SubmitFeePerHourYenOperation for SubmitFeePerHourYenOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        let model = entity::prelude::Identity::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find identity (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.is_some())
    }

    async fn submit_fee_per_hour_in_yen(
        &self,
        account_id: i64,
        fee_per_hour_in_yen: i32,
    ) -> Result<(), ErrResp> {
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
                        // update
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
                        // insert
                    }
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
