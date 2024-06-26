// Copyright 2022 Ken Miura

use std::collections::HashSet;

use async_session::serde_json::json;
use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::Datelike;
use common::opensearch::{index_document, update_document, INDEX_NAME};
use common::util::{Identity, Ymd};
use common::{ApiError, ErrResp, ErrRespStruct, RespResult};
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, QuerySelect, Set, TransactionError, TransactionTrait,
};
use entity::{career, consulting_fee};
use once_cell::sync::Lazy;
use opensearch::OpenSearch;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::err::unexpected_err_resp;
use crate::err::Code;
use crate::handlers::session::authentication::authenticated_handlers::authenticated_users::user::User;
use crate::handlers::session::authentication::authenticated_handlers::document_operation::{
    find_document_model_by_user_account_id_with_exclusive_lock, insert_document,
};

use super::bank_account_validator::{validate_bank_account, BankAccountValidationError};
use super::BankAccount;

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
    User { user_info }: User,
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
    Json(bank_account_register_req): Json<BankAccountRegisterReq>,
) -> RespResult<BankAccountResult> {
    let bank_account = bank_account_register_req.bank_account;
    let non_profit_objective = bank_account_register_req.non_profit_objective;
    let op = SubmitBankAccountOperationImpl { pool, index_client };
    handle_bank_account_req(user_info.account_id, bank_account, non_profit_objective, op).await
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct BankAccountRegisterReq {
    bank_account: BankAccount,
    non_profit_objective: bool,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct BankAccountResult {}

async fn handle_bank_account_req(
    account_id: i64,
    bank_account: BankAccount,
    non_profit_objective: bool,
    op: impl SubmitBankAccountOperation,
) -> RespResult<BankAccountResult> {
    if !non_profit_objective {
        error!(
            "did not agree non profit objective use (account id: {})",
            account_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ProfitObjectiveUseIsNotAllowd as u32,
            }),
        ));
    }
    validate_bank_account(&bank_account).map_err(|e| {
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

    let bank_account_exists = op.check_if_bank_account_exists(account_id).await?;
    if !bank_account_exists {
        is_eligible_to_create_bank_account(account_id, &op).await?;
    }

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

    op.submit_bank_account(account_id, bank_account).await?;

    Ok((StatusCode::OK, Json(BankAccountResult {})))
}

// 相談を受け付けるためには、(身分証明に加えて) 職務経歴、相談料、銀行口座の登録が必要となる。
// 銀行口座情報は必要がない限り保持したくないので、既に他に必要な情報が準備されていることを確認する。
async fn is_eligible_to_create_bank_account(
    account_id: i64,
    op: &impl SubmitBankAccountOperation,
) -> Result<(), ErrResp> {
    let career_exists = op.check_if_careers_exist(account_id).await?;
    if !career_exists {
        error!("no careers found (account id: {})", account_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoCareersFound as u32,
            }),
        ));
    }
    let fee_per_hour_in_yen_exists = op.check_if_fee_per_hour_in_yen_exists(account_id).await?;
    if !fee_per_hour_in_yen_exists {
        error!("no fee_per_hour_in_yen found (account id: {})", account_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoFeePerHourInYenFound as u32,
            }),
        ));
    }
    Ok(())
}

#[async_trait]
trait SubmitBankAccountOperation {
    async fn find_identity_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<Identity>, ErrResp>;

    async fn check_if_bank_account_exists(&self, account_id: i64) -> Result<bool, ErrResp>;

    async fn check_if_careers_exist(&self, account_id: i64) -> Result<bool, ErrResp>;

    async fn check_if_fee_per_hour_in_yen_exists(&self, account_id: i64) -> Result<bool, ErrResp>;

    async fn submit_bank_account(
        &self,
        account_id: i64,
        bank_account: BankAccount,
    ) -> Result<(), ErrResp>;
}

struct SubmitBankAccountOperationImpl {
    pool: DatabaseConnection,
    index_client: OpenSearch,
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

    async fn check_if_bank_account_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        let ba_option = entity::bank_account::Entity::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find bank_account (account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(ba_option.is_some())
    }

    async fn check_if_careers_exist(&self, account_id: i64) -> Result<bool, ErrResp> {
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
        Ok(!models.is_empty())
    }

    async fn check_if_fee_per_hour_in_yen_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        let fee_option = consulting_fee::Entity::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find consulting_fee (account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(fee_option.is_some())
    }

    async fn submit_bank_account(
        &self,
        account_id: i64,
        bank_account: BankAccount,
    ) -> Result<(), ErrResp> {
        let index_client = self.index_client.clone();
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let model_option = find_bank_account_with_exclusive_lock(txn, account_id).await?;
                    if let Some(model) = model_option {
                        // 処理が失敗して解析が必要になる場合以外、口座情報という個人情報はログに残さないようにする。
                        info!("update banck account (account_id: {})", account_id);
                        update_bank_account(txn, model, bank_account).await?;
                    } else {
                        // 処理が失敗して解析が必要になる場合以外、口座情報という個人情報はログに残さないようにする。
                        info!("register banck account (account_id: {})", account_id);
                        create_bank_account(txn, account_id, bank_account).await?;

                        // 口座情報を登録済かどうかでコンサルタント検索に出てくるかどうか変えるため、
                        // 以下で口座情報を登録済かどうかインデックスを更新する。
                        let document_option =
                        find_document_model_by_user_account_id_with_exclusive_lock(txn, account_id).await?;
                        if let Some(document) = document_option {
                            let document_id = document.document_id;
                            info!(
                                "update document for \"is_bank_account_registered\" (account_id: {}, document_id: {})",
                                account_id, document_id
                            );
                            update_is_bank_account_registered_on_document(INDEX_NAME, document_id.to_string().as_str(), index_client).await?;
                        } else {
                            // document_idとしてuser_account_idを利用
                            let document_id = account_id;
                            info!(
                                "create document for \"is_bank_account_registered\" (account_id: {}, document_id: {})",
                                account_id, document_id
                            );
                            insert_document(txn, account_id, document_id).await?;
                            add_new_document_with_is_bank_account_registered(
                                INDEX_NAME,
                                document_id.to_string().as_str(),
                                account_id,
                                index_client,
                            ).await?;
                        };
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
                    error!("failed to submit_bank_account: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;

        Ok(())
    }
}

async fn find_bank_account_with_exclusive_lock(
    txn: &DatabaseTransaction,
    account_id: i64,
) -> Result<Option<entity::bank_account::Model>, ErrRespStruct> {
    let ba = entity::bank_account::Entity::find_by_id(account_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find bank_account (account_id: {}): {}",
                account_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(ba)
}

async fn update_bank_account(
    txn: &DatabaseTransaction,
    model: entity::bank_account::Model,
    bank_account: BankAccount,
) -> Result<(), ErrRespStruct> {
    let account_id = model.user_account_id;
    let ba = bank_account.clone();
    let mut active_model: entity::bank_account::ActiveModel = model.into();
    active_model.bank_code = Set(bank_account.bank_code);
    active_model.branch_code = Set(bank_account.branch_code);
    active_model.account_type = Set(bank_account.account_type);
    active_model.account_number = Set(bank_account.account_number);
    active_model.account_holder_name = Set(bank_account.account_holder_name);
    let _ = active_model.update(txn).await.map_err(|e| {
        error!(
            "failed to update banck_account (account_id: {}, bank_account: {:?}): {}",
            account_id, ba, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

async fn create_bank_account(
    txn: &DatabaseTransaction,
    account_id: i64,
    bank_account: BankAccount,
) -> Result<(), ErrRespStruct> {
    let ba = bank_account.clone();
    let active_model = entity::bank_account::ActiveModel {
        user_account_id: Set(account_id),
        bank_code: Set(bank_account.bank_code),
        branch_code: Set(bank_account.branch_code),
        account_type: Set(bank_account.account_type),
        account_number: Set(bank_account.account_number),
        account_holder_name: Set(bank_account.account_holder_name),
    };
    let _ = active_model.insert(txn).await.map_err(|e| {
        error!(
            "failed to insert banck_account (account_id: {}, bank_account: {:?}): {}",
            account_id, ba, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

async fn update_is_bank_account_registered_on_document(
    index_name: &str,
    document_id: &str,
    index_client: OpenSearch,
) -> Result<(), ErrRespStruct> {
    let value = format!("ctx._source.is_bank_account_registered = {}", true);
    let script = json!({
        "script": {
            "source": value
        }
    });
    update_document(index_name, document_id, &script, &index_client)
        .await
        .map_err(|e| {
            error!(
                "failed to update is_bank_account_registered on document (document_id: {})",
                document_id
            );
            ErrRespStruct { err_resp: e }
        })?;
    Ok(())
}

async fn add_new_document_with_is_bank_account_registered(
    index_name: &str,
    document_id: &str,
    account_id: i64,
    index_client: OpenSearch,
) -> Result<(), ErrRespStruct> {
    let new_document = json!({
        "user_account_id": account_id,
        "careers": [],
        "fee_per_hour_in_yen": null,
        "is_bank_account_registered": true,
        "rating": null,
        "num_of_rated": 0,
        "disabled": false
    });
    index_document(index_name, document_id, &new_document, &index_client)
        .await
        .map_err(|e| {
            error!(
                "failed to index new document with is_bank_account_registered (document_id: {})",
                document_id
            );
            ErrRespStruct { err_resp: e }
        })?;
    Ok(())
}

fn create_invalid_bank_account_err(e: &BankAccountValidationError) -> ErrResp {
    let code = match e {
        BankAccountValidationError::InvalidBankCodeFormat(_) => Code::InvalidBankCodeFormat,
        BankAccountValidationError::InvalidBranchCodeFormat(_) => Code::InvalidBranchCodeFormat,
        BankAccountValidationError::InvalidAccountType(_) => Code::InvalidAccountType,
        BankAccountValidationError::InvalidAccountNumberFormat(_) => {
            Code::InvalidAccountNumberFormat
        }
        BankAccountValidationError::InvalidAccountHolderNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => Code::InvalidAccountHolderNameLength,
        BankAccountValidationError::IllegalCharInAccountHolderName(_) => {
            Code::IllegalCharInAccountHolderName
        }
    };
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

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Debug, Clone)]
    struct SubmitBankAccountOperationMock {
        identity: Option<Identity>,
        bank_account_exists: bool,
        careers_exist: bool,
        fee_per_hour_in_yen_exists: bool,
        submit_bank_account_err: Option<ErrResp>,
    }

    #[async_trait]
    impl SubmitBankAccountOperation for SubmitBankAccountOperationMock {
        async fn find_identity_by_account_id(
            &self,
            _account_id: i64,
        ) -> Result<Option<Identity>, ErrResp> {
            Ok(self.identity.clone())
        }

        async fn check_if_bank_account_exists(&self, _account_id: i64) -> Result<bool, ErrResp> {
            Ok(self.bank_account_exists)
        }

        async fn check_if_careers_exist(&self, _account_id: i64) -> Result<bool, ErrResp> {
            Ok(self.careers_exist)
        }

        async fn check_if_fee_per_hour_in_yen_exists(
            &self,
            _account_id: i64,
        ) -> Result<bool, ErrResp> {
            Ok(self.fee_per_hour_in_yen_exists)
        }

        async fn submit_bank_account(
            &self,
            _account_id: i64,
            _bank_account: BankAccount,
        ) -> Result<(), ErrResp> {
            if let Some(err) = &self.submit_bank_account_err {
                Err((err.0, Json(ApiError { code: err.1.code })))
            } else {
                Ok(())
            }
        }
    }

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<BankAccountResult>,
    }

    #[derive(Debug)]
    struct Input {
        account_id: i64,
        bank_account_register_req: BankAccountRegisterReq,
        op: SubmitBankAccountOperationMock,
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        let identity1 = Identity {
            last_name: "田中".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "タナカ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: Ymd {
                year: 1999,
                month: 12,
                day: 5,
            },
            prefecture: "北海道".to_string(),
            city: "札幌市".to_string(),
            address_line1: "北区２−１".to_string(),
            address_line2: None,
            telephone_number: "09012345678".to_string(),
        };

        let identity2 = Identity {
            last_name: "庄司".to_string(),
            first_name: "ジロウ".to_string(),
            last_name_furigana: "ショウジ".to_string(),
            first_name_furigana: "ジロウ".to_string(),
            date_of_birth: Ymd {
                year: 1999,
                month: 12,
                day: 5,
            },
            prefecture: "北海道".to_string(),
            city: "札幌市".to_string(),
            address_line1: "北区２−１".to_string(),
            address_line2: None,
            telephone_number: "09012345678".to_string(),
        };

        let identity3 = Identity {
            last_name: "ウィンストン".to_string(),
            first_name: "チャーチル".to_string(),
            last_name_furigana: "ウィンストン".to_string(),
            first_name_furigana: "チャーチル".to_string(),
            date_of_birth: Ymd {
                year: 1999,
                month: 12,
                day: 5,
            },
            prefecture: "北海道".to_string(),
            city: "札幌市".to_string(),
            address_line1: "北区２−１".to_string(),
            address_line2: None,
            telephone_number: "09012345678".to_string(),
        };

        vec![
            TestCase {
                name: "success case1".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: identity1.last_name_furigana.clone()
                                + "　"
                                + identity1.first_name_furigana.as_str(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity1.clone()),
                        bank_account_exists: false,
                        careers_exist: true,
                        fee_per_hour_in_yen_exists: true,
                        submit_bank_account_err: None,
                    },
                },
                expected: Ok((StatusCode::OK, Json(BankAccountResult {}))),
            },
            TestCase {
                name: "success case2".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: "ショウジ　ジロウ".to_string(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity2.clone()),
                        bank_account_exists: true,
                        careers_exist: true,
                        fee_per_hour_in_yen_exists: true,
                        submit_bank_account_err: None,
                    },
                },
                expected: Ok((StatusCode::OK, Json(BankAccountResult {}))),
            },
            TestCase {
                name: "success case3".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: "シヨウジ　ジロウ".to_string(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity2),
                        bank_account_exists: false,
                        careers_exist: true,
                        fee_per_hour_in_yen_exists: true,
                        submit_bank_account_err: None,
                    },
                },
                expected: Ok((StatusCode::OK, Json(BankAccountResult {}))),
            },
            TestCase {
                name: "success case4".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: "ウィンストン　チャーチル".to_string(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity3.clone()),
                        bank_account_exists: false,
                        careers_exist: true,
                        fee_per_hour_in_yen_exists: true,
                        submit_bank_account_err: None,
                    },
                },
                expected: Ok((StatusCode::OK, Json(BankAccountResult {}))),
            },
            TestCase {
                name: "success case5".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: "ウインストン　チヤーチル".to_string(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity3.clone()),
                        bank_account_exists: false,
                        careers_exist: true,
                        fee_per_hour_in_yen_exists: true,
                        submit_bank_account_err: None,
                    },
                },
                expected: Ok((StatusCode::OK, Json(BankAccountResult {}))),
            },
            TestCase {
                name: "fail invalid bank code".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "不当な形式の銀行コード".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: identity1.last_name_furigana.clone()
                                + "　"
                                + identity1.first_name_furigana.as_str(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity1.clone()),
                        bank_account_exists: true,
                        careers_exist: true,
                        fee_per_hour_in_yen_exists: true,
                        submit_bank_account_err: None,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidBankCodeFormat as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail invalid branch code".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "不当な形式の支店コード".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: identity1.last_name_furigana.clone()
                                + "　"
                                + identity1.first_name_furigana.as_str(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity1.clone()),
                        bank_account_exists: false,
                        careers_exist: true,
                        fee_per_hour_in_yen_exists: true,
                        submit_bank_account_err: None,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidBranchCodeFormat as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail invalid account type".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "当座".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: identity1.last_name_furigana.clone()
                                + "　"
                                + identity1.first_name_furigana.as_str(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity1.clone()),
                        bank_account_exists: true,
                        careers_exist: true,
                        fee_per_hour_in_yen_exists: true,
                        submit_bank_account_err: None,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidAccountType as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail invalid account number".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "不当な形式の口座番号".to_string(),
                            account_holder_name: identity1.last_name_furigana.clone()
                                + "　"
                                + identity1.first_name_furigana.as_str(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity1.clone()),
                        bank_account_exists: false,
                        careers_exist: true,
                        fee_per_hour_in_yen_exists: true,
                        submit_bank_account_err: None,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidAccountNumberFormat as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail invalid account holder name".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: "田中　太郎".to_string(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity1.clone()),
                        bank_account_exists: true,
                        careers_exist: true,
                        fee_per_hour_in_yen_exists: true,
                        submit_bank_account_err: None,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalCharInAccountHolderName as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail invalid account holder name length".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: "ア".to_string(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity1.clone()),
                        bank_account_exists: false,
                        careers_exist: true,
                        fee_per_hour_in_yen_exists: true,
                        submit_bank_account_err: None,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidAccountHolderNameLength as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail no identity registered".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: "タナカ　タロウ".to_string(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: None,
                        bank_account_exists: false,
                        careers_exist: false,
                        fee_per_hour_in_yen_exists: false,
                        submit_bank_account_err: None,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoIdentityRegistered as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail account holder name does not match full name1".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: "タナカ　ジロウ".to_string(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity1.clone()),
                        bank_account_exists: true,
                        careers_exist: true,
                        fee_per_hour_in_yen_exists: true,
                        submit_bank_account_err: None,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::AccountHolderNameDoesNotMatchFullName as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail account holder name does not match full name2".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: "ウィンストン　チヤーチル".to_string(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity3),
                        bank_account_exists: false,
                        careers_exist: true,
                        fee_per_hour_in_yen_exists: true,
                        submit_bank_account_err: None,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::AccountHolderNameDoesNotMatchFullName as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail no careers found".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: identity1.last_name_furigana.clone()
                                + "　"
                                + identity1.first_name_furigana.as_str(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity1.clone()),
                        bank_account_exists: false,
                        careers_exist: false,
                        fee_per_hour_in_yen_exists: true,
                        submit_bank_account_err: None,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoCareersFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail no fee_per_hour_in_yen found".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: identity1.last_name_furigana.clone()
                                + "　"
                                + identity1.first_name_furigana.as_str(),
                        },
                        non_profit_objective: true,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity1.clone()),
                        bank_account_exists: false,
                        careers_exist: true,
                        fee_per_hour_in_yen_exists: false,
                        submit_bank_account_err: None,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoFeePerHourInYenFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail profit objective use is not allowed".to_string(),
                input: Input {
                    account_id: 514,
                    bank_account_register_req: BankAccountRegisterReq {
                        bank_account: BankAccount {
                            bank_code: "0001".to_string(),
                            branch_code: "001".to_string(),
                            account_type: "普通".to_string(),
                            account_number: "1234567".to_string(),
                            account_holder_name: identity1.last_name_furigana.clone()
                                + "　"
                                + identity1.first_name_furigana.as_str(),
                        },
                        non_profit_objective: false,
                    },
                    op: SubmitBankAccountOperationMock {
                        identity: Some(identity1.clone()),
                        bank_account_exists: false,
                        careers_exist: true,
                        fee_per_hour_in_yen_exists: true,
                        submit_bank_account_err: None,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::ProfitObjectiveUseIsNotAllowd as u32,
                    }),
                )),
            },
        ]
    });

    #[tokio::test]
    async fn handle_bank_account_req_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let account_id = test_case.input.account_id;
            let bank_account = test_case
                .input
                .bank_account_register_req
                .bank_account
                .clone();
            let non_profit_objective = test_case
                .input
                .bank_account_register_req
                .non_profit_objective;
            let op = test_case.input.op.clone();
            let resp =
                handle_bank_account_req(account_id, bank_account, non_profit_objective, op).await;

            let message = format!("test case \"{}\" failed", test_case.name.clone());
            if test_case.expected.is_ok() {
                let result = resp.expect("failed to get Ok");
                let expected_result = test_case.expected.as_ref().expect("failed to get Ok");
                assert_eq!(expected_result.0, result.0, "{}", message);
                assert_eq!(expected_result.1 .0, result.1 .0, "{}", message);
            } else {
                let result = resp.expect_err("failed to get Err");
                let expected_result = test_case.expected.as_ref().expect_err("failed to get Err");
                assert_eq!(expected_result.0, result.0, "{}", message);
                assert_eq!(expected_result.1 .0, result.1 .0, "{}", message);
            }
        }
    }
}
