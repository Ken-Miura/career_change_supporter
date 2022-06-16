// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{Extension, Json};
use common::{ApiError, ErrResp, RespResult};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;
use crate::{
    err::Code,
    util::{
        session::User,
        validator::bank_account_validator::{validate_bank_account, BankAccountValidationError},
        BankAccount,
    },
};

pub(crate) async fn post_bank_account(
    User { account_id }: User,
    Json(bank_account): Json<BankAccount>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<BankAccountResult> {
    let op = SubmitBankAccountOperationImpl { pool };
    handle_bank_account_req(account_id, bank_account, op).await
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct BankAccountResult {}

async fn handle_bank_account_req(
    account_id: i64,
    bank_account: BankAccount,
    op: impl SubmitBankAccountOperation,
) -> RespResult<BankAccountResult> {
    let _ = validate_bank_account(&bank_account).map_err(|e| {
        error!("invalid bank account: {}", e);
        create_invalid_bank_account_err(&e)
    })?;

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

    todo!()
}

#[async_trait]
trait SubmitBankAccountOperation {
    /// Identityが存在するか確認する。存在する場合、trueを返す。そうでない場合、falseを返す。
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
}

struct SubmitBankAccountOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl SubmitBankAccountOperation for SubmitBankAccountOperationImpl {
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
}

fn create_invalid_bank_account_err(e: &BankAccountValidationError) -> ErrResp {
    let code;
    match e {
        BankAccountValidationError::InvalidBankCodeFormat(_) => code = Code::InvalidBankCodeFormat,
        BankAccountValidationError::InvalidBranchCodeFormat(_) => {
            code = Code::InvalidBranchCodeFormat
        }
        BankAccountValidationError::InvalidAccountType(_) => code = Code::InvalidAccountType,
        BankAccountValidationError::InvalidAccountNumberFormat(_) => {
            code = Code::InvalidAccountNumberFormat
        }
        BankAccountValidationError::InvalidAccountHolderNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = Code::InvalidAccountHolderNameLength,
        BankAccountValidationError::IllegalCharInAccountHolderName(_) => {
            code = Code::IllegalCharInAccountHolderName
        }
    }
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError { code: code as u32 }),
    )
}
