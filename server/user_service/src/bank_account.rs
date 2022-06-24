// Copyright 2022 Ken Miura

use std::collections::HashSet;

use async_session::serde_json::json;
use axum::async_trait;
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::Datelike;
use common::opensearch::{index_document, update_document, INDEX_NAME, OPENSEARCH_ENDPOINT_URI};
use common::payment_platform::tenant::{
    CreateTenant, TenantOperation, TenantOperationImpl, UpdateTenant,
};
use common::util::{Identity, Ymd};
use common::{ApiError, ErrResp, ErrRespStruct, RespResult};
use entity::prelude::Tenant as TenantEntity;
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, QuerySelect, Set, TransactionError,
    TransactionTrait,
};
use once_cell::sync::Lazy;
use serde::Serialize;
use tracing::{error, info};
use uuid::Uuid;

use crate::err::unexpected_err_resp;
use crate::util::{
    find_document_model_by_user_account_id_with_shared_lock, insert_document, ACCESS_INFO,
};
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

const PLATFORM_FEE_RATE: &str = "30.00";
const PAYJP_FEE_INCLUDED: bool = true;
const MINIMUM_TRANSFER_AMOUNT: i32 = 1000;

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

    let _ = op.submit_bank_account(account_id, bank_account).await?;

    Ok((StatusCode::OK, Json(BankAccountResult {})))
}

#[async_trait]
trait SubmitBankAccountOperation {
    async fn find_identity_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<Identity>, ErrResp>;

    async fn submit_bank_account(
        &self,
        account_id: i64,
        bank_account: BankAccount,
    ) -> Result<(), ErrResp>;
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

    async fn submit_bank_account(
        &self,
        account_id: i64,
        bank_account: BankAccount,
    ) -> Result<(), ErrResp> {
        // pay.jp上のテナントの作成（更新）とopensearch上のインデックスの作成（更新）は
        // まとめて一つのトランザクションで実施したい。
        // しかし、片方が失敗し、もう片方が成功するケースのハンドリングが複雑になるため、それぞれ独立したトランザクションで対応する
        let _ = self.submit_tenant(account_id, bank_account).await?;
        let _ = self
            .set_bank_account_registered_on_index(account_id)
            .await?;
        Ok(())
    }
}

impl SubmitBankAccountOperationImpl {
    async fn submit_tenant(
        &self,
        account_id: i64,
        bank_account: BankAccount,
    ) -> Result<(), ErrResp> {
        let _ = self
            .pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let tenant_option = TenantEntity::find_by_id(account_id)
                        .lock_shared()
                        .one(txn)
                        .await
                        .map_err(|e| {
                            error!("failed to find tenant (account_id: {}): {}", account_id, e);
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;

                    let tenant_op = TenantOperationImpl::new(&ACCESS_INFO);
                    if let Some(tenant) = tenant_option {
                        let update_tenant = UpdateTenant {
                            name: bank_account.account_holder_name.clone(),
                            platform_fee_rate: PLATFORM_FEE_RATE.to_string(),
                            minimum_transfer_amount: MINIMUM_TRANSFER_AMOUNT,
                            bank_code: bank_account.bank_code,
                            bank_branch_code: bank_account.branch_code,
                            bank_account_type: bank_account.account_type,
                            bank_account_number: bank_account.account_number,
                            bank_account_holder_name: bank_account.account_holder_name,
                            metadata: None,
                        };
                        info!(
                            "update tenant (account_id: {}, tenant_id: {}, update_tenant: {:?})",
                            account_id, tenant.tenant_id, update_tenant
                        );
                        let _ = tenant_op
                            .update_tenant(tenant.tenant_id.as_str(), &update_tenant)
                            .await
                            .map_err(|e| {
                                error!(
                                "failed to update tenant (account_id: {}, update_tenant: {:?}): {}",
                                account_id, update_tenant, e
                            );
                                ErrRespStruct {
                                    err_resp: create_err_resp(&e),
                                }
                            })?;
                    } else {
                        let uuid = Uuid::new_v4().simple().to_string();
                        let active_model = entity::tenant::ActiveModel {
                            user_account_id: Set(account_id),
                            tenant_id: Set(uuid.clone()),
                        };
                        active_model.insert(txn).await.map_err(|e| {
                            error!(
                                "failed to insert tenant (account_id: {}, uuid: {}): {}",
                                account_id, uuid, e
                            );
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;
                        let create_tenant = CreateTenant {
                            name: bank_account.account_holder_name.clone(),
                            id: uuid.clone(),
                            platform_fee_rate: PLATFORM_FEE_RATE.to_string(),
                            payjp_fee_included: PAYJP_FEE_INCLUDED,
                            minimum_transfer_amount: MINIMUM_TRANSFER_AMOUNT,
                            bank_code: bank_account.bank_code,
                            bank_branch_code: bank_account.branch_code,
                            bank_account_type: bank_account.account_type,
                            bank_account_number: bank_account.account_number,
                            bank_account_holder_name: bank_account.account_holder_name,
                            metadata: None,
                        };
                        info!(
                            "create tenant (account_id: {}, tenant_id: {}, create_tenant: {:?})",
                            account_id, uuid, create_tenant
                        );
                        let _ = tenant_op.create_tenant(&create_tenant).await.map_err(|e| {
                            error!(
                                "failed to create tenant (account_id: {}, create_tenant: {:?}): {}",
                                account_id, create_tenant, e
                            );
                            ErrRespStruct {
                                err_resp: create_err_resp(&e),
                            }
                        })?;
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
                    error!("failed to submit tenant: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }

    async fn set_bank_account_registered_on_index(&self, account_id: i64) -> Result<(), ErrResp> {
        let _ = self
            .pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let document_option =
                        find_document_model_by_user_account_id_with_shared_lock(txn, account_id).await?;
                    if let Some(document) = document_option {
                        let document_id = document.document_id;
                        info!("update document for \"is_bank_account_registered\" (account_id: {}, document_id: {})", account_id, document_id);
                        let _ = update_is_bank_account_registered_on_document(
                            &OPENSEARCH_ENDPOINT_URI,
                            INDEX_NAME,
                            document_id.to_string().as_str(),
                        )
                        .await?;
                    } else {
                        // document_idとしてuser_account_idを利用
                        let document_id = account_id;
                        info!("create document for \"is_bank_account_registered\" (account_id: {}, document_id: {})", account_id, document_id);
                        let _ = insert_document(txn, account_id, document_id).await?;
                        let _ = add_new_document_with_is_bank_account_registered(
                            &OPENSEARCH_ENDPOINT_URI,
                            INDEX_NAME,
                            document_id.to_string().as_str(),
                            account_id,
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
                    error!(
                        "failed to index document with is_bank_account_registered: {}",
                        err_resp_struct
                    );
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

async fn update_is_bank_account_registered_on_document(
    endpoint_uri: &str,
    index_name: &str,
    document_id: &str,
) -> Result<(), ErrRespStruct> {
    let value = format!("ctx._source.is_bank_account_registered = {}", true);
    let script = json!({
        "script": {
            "source": value
        }
    });
    let _ = update_document(endpoint_uri, index_name, document_id, &script)
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
    endpoint_uri: &str,
    index_name: &str,
    document_id: &str,
    account_id: i64,
) -> Result<(), ErrRespStruct> {
    let new_document = json!({
        "user_account_id": account_id,
        "careers": [],
        "fee_per_hour_in_yen": null,
        "rating": null,
        "is_bank_account_registered": true
    });
    let _ = index_document(endpoint_uri, index_name, document_id, &new_document)
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

fn create_err_resp(e: &common::payment_platform::Error) -> ErrResp {
    match e {
        common::payment_platform::Error::RequestProcessingError(_) => unexpected_err_resp(),
        common::payment_platform::Error::ApiError(e) => {
            let status_code = e.error.status;
            if status_code == StatusCode::TOO_MANY_REQUESTS.as_u16() as u32 {
                return (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ApiError {
                        code: Code::ReachPaymentPlatformRateLimit as u32,
                    }),
                );
            }
            let code = &e.error.code;
            if let Some(code) = code {
                create_err_resp_from_code(code.as_str())
            } else {
                unexpected_err_resp()
            }
        }
    }
}

fn create_err_resp_from_code(code: &str) -> ErrResp {
    if code == "invalid_bank_code" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidBank as u32,
            }),
        )
    } else if code == "invalid_bank_branch_code" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidBankBranch as u32,
            }),
        )
    } else if code == "invalid_bank_account_number" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidBankAccountNumber as u32,
            }),
        )
    } else {
        unexpected_err_resp()
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use common::{
        util::{Identity, Ymd},
        ApiError, ErrResp,
    };

    use crate::bank_account::BankAccountResult;
    use crate::util::BankAccount;

    use super::{handle_bank_account_req, SubmitBankAccountOperation};

    struct SubmitBankAccountOperationMock {
        identity: Option<Identity>,
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

        async fn submit_bank_account(
            &self,
            _account_id: i64,
            _bank_account: BankAccount,
        ) -> Result<(), ErrResp> {
            if let Some(err) = &self.submit_bank_account_err {
                Err((
                    err.0,
                    Json(ApiError {
                        code: err.1.code as u32,
                    }),
                ))
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn handle_bank_account_req_test() {
        let account_id = 5135;
        let identity = Identity {
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
        let bank_account = BankAccount {
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: identity.last_name_furigana.clone()
                + "　"
                + identity.first_name_furigana.as_str(),
        };
        let op = SubmitBankAccountOperationMock {
            identity: Some(identity),
            submit_bank_account_err: None,
        };

        let resp = handle_bank_account_req(account_id, bank_account, op).await;

        let result = resp.expect("failed to get Ok");
        assert_eq!(result.0, StatusCode::OK);
        assert_eq!(result.1 .0, BankAccountResult {});
    }
}
