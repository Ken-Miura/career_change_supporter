// Copyright 2022 Ken Miura

use std::collections::HashSet;

use axum::async_trait;
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::Datelike;
use common::util::{Identity, Ymd};
use common::{ApiError, ErrResp, RespResult};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use once_cell::sync::Lazy;
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

static KATAKANA_LOWER_CASE_UPPER_CASE_SET: Lazy<HashSet<(String, String)>> = Lazy::new(|| {
    let mut set: HashSet<(String, String)> = HashSet::with_capacity(10);
    set.insert(("ァ".to_string(), "ア".to_string()));
    set.insert(("ィ".to_string(), "イ".to_string()));
    set.insert(("ゥ".to_string(), "ウ".to_string()));
    set.insert(("ェ".to_string(), "エ".to_string()));
    set.insert(("ォ".to_string(), "オ".to_string()));
    set.insert(("ッ".to_string(), "ツ".to_string()));
    set.insert(("ャ".to_string(), "ヤ".to_string()));
    set.insert(("ュ".to_string(), "ユ".to_string()));
    set.insert(("ョ".to_string(), "ヨ".to_string()));
    set.insert(("ヮ".to_string(), "ワ".to_string()));
    set
});

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
    let bank_account = trim_space_from_bank_account(bank_account);

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

    let zenkaku_space = "　";
    let full_name =
        identity.last_name_furigana + zenkaku_space + identity.first_name_furigana.as_str();
    if !account_holder_name_matches_full_name(
        bank_account.account_holder_name.as_str(),
        full_name.as_str(),
    ) {
        error!(
            "account_holder_name ({}) does not match full_name ({})",
            bank_account.account_holder_name, full_name
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::AccountHolderNameDoesNotMatchFullName as u32,
            }),
        ));
    }

    let _ = op.submit_bank_account(bank_account).await?;

    Ok((StatusCode::OK, Json(BankAccountResult {})))
}

#[async_trait]
trait SubmitBankAccountOperation {
    async fn find_identity_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<Identity>, ErrResp>;

    async fn submit_bank_account(&self, bank_account: BankAccount) -> Result<(), ErrResp>;
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

    async fn submit_bank_account(&self, bank_account: BankAccount) -> Result<(), ErrResp> {
        // tenantチェック＋tenant作成＋tenant新規or更新
        // documentチェック＋更新
        todo!()
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

fn trim_space_from_bank_account(bank_account: BankAccount) -> BankAccount {
    BankAccount {
        bank_code: bank_account.bank_code.trim().to_string(),
        branch_code: bank_account.branch_code.trim().to_string(),
        account_type: bank_account.account_type.trim().to_string(),
        account_number: bank_account.account_number.trim().to_string(),
        account_holder_name: bank_account.account_holder_name.trim().to_string(),
    }
}

fn account_holder_name_matches_full_name(account_holder_name: &str, full_name: &str) -> bool {
    if account_holder_name == full_name {
        return true;
    }
    // 多くの金融機関が小さなカタカナは、大きなカタカナとして登録する。
    // 従って、小さなカタカナを大きなカタカナに変換した結果と比較して一致する場合も
    // trueとして処理する
    let full_name_upper_case = to_upper_case_of_katakana(full_name);
    if account_holder_name == full_name_upper_case {
        return true;
    }
    false
}

fn to_upper_case_of_katakana(katakana: &str) -> String {
    let mut result = katakana.to_string();
    for l_u in KATAKANA_LOWER_CASE_UPPER_CASE_SET.iter() {
        if result.contains(l_u.0.as_str()) {
            result = result.replace(l_u.0.as_str(), l_u.1.as_str());
        }
    }
    result
}
