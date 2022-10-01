// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{DateTime, FixedOffset};
use common::payment_platform::charge::{Charge, ChargeOperation, ChargeOperationImpl};
use common::smtp::{SendMail, SmtpClient, SMTP_HOST, SMTP_PASSWORD, SMTP_PORT, SMTP_USERNAME};
use common::ApiError;
use common::{ErrResp, ErrRespStruct, RespResult};
use entity::sea_orm::ActiveValue::NotSet;
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, Set, TransactionError, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::util::session::User;
use crate::util::{self, ACCESS_INFO, KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ};

pub(crate) async fn post_finish_request_consultation(
    User { account_id }: User,
    Json(param): Json<FinishRequestConsultationParam>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<FinishRequestConsultationResult> {
    let charge_id = param.charge_id;
    let op = FinishRequestConsultationOperationImpl { pool };
    let smtp_client = SmtpClient::new(
        SMTP_HOST.to_string(),
        *SMTP_PORT,
        SMTP_USERNAME.to_string(),
        SMTP_PASSWORD.to_string(),
    );
    handle_finish_request_consultation(account_id, charge_id, op, smtp_client).await
}

#[derive(Deserialize)]
pub(crate) struct FinishRequestConsultationParam {
    pub charge_id: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct FinishRequestConsultationResult {}

async fn handle_finish_request_consultation(
    account_id: i64,
    charge_id: String,
    op: impl FinishRequestConsultationOperation,
    send_mail: impl SendMail,
) -> RespResult<FinishRequestConsultationResult> {
    let _ = validate_identity_exists(account_id, &op).await?;
    let charge = op.get_charge_by_charge_id(charge_id.clone()).await?;
    let consultant_id = extract_consultant_id(&charge)?;
    let _ = validate_consultant_is_available(consultant_id, &op).await?;
    // chargeのステータスチェック
    // chargeからexpireの時間を取得
    // 3Dセキュアフロー完了
    // メール送信
    todo!()
}

async fn validate_identity_exists(
    account_id: i64,
    op: &impl FinishRequestConsultationOperation,
) -> Result<(), ErrResp> {
    let identity_exists = op.check_if_identity_exists(account_id).await?;
    if !identity_exists {
        error!("identity is not registered (account_id: {})", account_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoIdentityRegistered as u32,
            }),
        ));
    }
    Ok(())
}

fn extract_consultant_id(charge: &Charge) -> Result<i64, ErrResp> {
    let metadata = match charge.metadata.clone() {
        Some(metadata) => metadata,
        None => {
            error!("no metadata found on charge (id: {})", charge.id);
            return Err(unexpected_err_resp());
        }
    };
    let consultant_id = match metadata.get(KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ) {
        Some(c_id) => c_id,
        None => {
            error!(
                "no consultant_id found in metadata on charge (id: {})",
                charge.id
            );
            return Err(unexpected_err_resp());
        }
    };
    let consultant_id = match consultant_id.parse::<i64>() {
        Ok(c_id) => c_id,
        Err(e) => {
            error!("failed to parse consultant_id in metadata on charge (id: {}, consultant_id: {}): {}", charge.id, consultant_id, e);
            return Err(unexpected_err_resp());
        }
    };
    Ok(consultant_id)
}

async fn validate_consultant_is_available(
    consultant_id: i64,
    op: &impl FinishRequestConsultationOperation,
) -> Result<(), ErrResp> {
    let consultant_available = op.check_if_consultant_is_available(consultant_id).await?;
    if !consultant_available {
        error!(
            "consultant is not available (consultant_id: {})",
            consultant_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultantIsNotAvailable as u32,
            }),
        ));
    }
    Ok(())
}

#[async_trait]
trait FinishRequestConsultationOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
    async fn get_charge_by_charge_id(&self, charge_id: String) -> Result<Charge, ErrResp>;
    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp>;
    async fn create_request_consultation(
        &self,
        account_id: i64,
        consultant_id: i64,
        charge_id: String,
        expiry_date_time: DateTime<FixedOffset>,
    ) -> Result<Charge, ErrResp>;
}

struct FinishRequestConsultationOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl FinishRequestConsultationOperation for FinishRequestConsultationOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn get_charge_by_charge_id(&self, charge_id: String) -> Result<Charge, ErrResp> {
        let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
        let charge = charge_op
            .ge_charge_by_charge_id(charge_id.as_str())
            .await
            .map_err(|e| {
                // TODO: https://pay.jp/docs/api/#error に基づいてハンドリングする
                error!("failed to get charge by charge id ({}): {}", charge_id, e);
                unexpected_err_resp()
            })?;
        Ok(charge)
    }

    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp> {
        util::check_if_consultant_is_available(&self.pool, consultant_id).await
    }

    async fn create_request_consultation(
        &self,
        account_id: i64,
        consultant_id: i64,
        charge_id: String,
        expiry_date_time: DateTime<FixedOffset>,
    ) -> Result<Charge, ErrResp> {
        let charge = self.pool.transaction::<_, Charge, ErrRespStruct>(|txn| {
            Box::pin(async move {
                let active_model = entity::consultation_req::ActiveModel {
                    consultation_req_id: NotSet,
                    user_account_id: Set(account_id),
                    consultant_id: Set(consultant_id),
                    charge_id: Set(charge_id.clone()),
                    expiry_date_time: Set(expiry_date_time),
                };
                active_model.insert(txn).await.map_err(|e| {
                    error!(
                        "failed to insert consultation_req (account_id: {}, consultant_id: {}, charge_id: {}, expiry_date_time: {}): {}",
                        account_id, consultant_id, charge_id.clone(), expiry_date_time, e
                    );
                    ErrRespStruct {
                        err_resp: unexpected_err_resp(),
                    }
                })?;

                let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
                let charge = charge_op.finish_three_d_secure_flow(charge_id.as_str())
                    .await.map_err(|e| {
                        // TODO: https://pay.jp/docs/api/#error に基づいてハンドリングする
                        error!("failed to finish 3D secure flow (charge_id: {}): {}", charge_id, e);
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                Ok(charge)
            })
        }).await.map_err(|e| match e {
            TransactionError::Connection(db_err) => {
                error!("connection error: {}", db_err);
                unexpected_err_resp()
            }
            TransactionError::Transaction(err_resp_struct) => {
                error!("failed to create_request_consultation: {}", err_resp_struct);
                err_resp_struct.err_resp
            }
        })?;
        Ok(charge)
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
