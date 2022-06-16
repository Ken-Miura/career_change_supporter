// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::Datelike;
use common::util::{Identity, Ymd};
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

    let identity_option = op.find_identity_by_account_id(account_id).await?;
    let identity = identity_option.ok_or_else(|| {
        error!("identity is not registered (account id: {})", account_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoIdentityRegistered as u32,
            }),
        )
    })?;
    // 口座名義チェック
    // tenantチェック＋tenant作成＋tenant新規or更新
    // documentチェック＋更新
    todo!()
}

#[async_trait]
trait SubmitBankAccountOperation {
    async fn find_identity_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<Identity>, ErrResp>;
}

struct SubmitBankAccountOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl SubmitBankAccountOperation for SubmitBankAccountOperationImpl {
    async fn find_identity_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<Identity>, ErrResp> {
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
        Ok(model.map(|m| Identity {
            last_name: m.last_name,
            first_name: m.first_name,
            last_name_furigana: m.last_name_furigana,
            first_name_furigana: m.first_name_furigana,
            date_of_birth: Ymd {
                year: m.date_of_birth.year(),
                month: m.date_of_birth.month(),
                day: m.date_of_birth.day(),
            },
            prefecture: m.prefecture,
            city: m.city,
            address_line1: m.address_line1,
            address_line2: m.address_line2,
            telephone_number: m.telephone_number,
        }))
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
