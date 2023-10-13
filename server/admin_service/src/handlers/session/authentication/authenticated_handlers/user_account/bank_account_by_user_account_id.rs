// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::util::BankAccount;
use common::{ErrResp, RespResult};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;

use super::super::admin::Admin;
use super::{validate_account_id_is_positive, UserAccountIdQuery};

pub(crate) async fn get_bank_account_by_user_account_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<UserAccountIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<BankAccountResult> {
    let query = query.0;
    let op = BankAccountOperationImpl { pool };
    get_bank_account_by_user_account_id_internal(query.user_account_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub(crate) struct BankAccountResult {
    bank_account: Option<BankAccount>,
}

async fn get_bank_account_by_user_account_id_internal(
    user_account_id: i64,
    op: impl BankAccountOperation,
) -> RespResult<BankAccountResult> {
    validate_account_id_is_positive(user_account_id)?;
    let bank_account = op
        .get_bank_account_by_user_account_id(user_account_id)
        .await?;
    Ok((StatusCode::OK, Json(BankAccountResult { bank_account })))
}

#[async_trait]
trait BankAccountOperation {
    async fn get_bank_account_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<BankAccount>, ErrResp>;
}

struct BankAccountOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl BankAccountOperation for BankAccountOperationImpl {
    async fn get_bank_account_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<BankAccount>, ErrResp> {
        let result = entity::bank_account::Entity::find_by_id(user_account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find bank_account (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(result.map(|m| BankAccount {
            bank_code: m.bank_code,
            branch_code: m.branch_code,
            account_type: m.account_type,
            account_number: m.account_number,
            account_holder_name: m.account_holder_name,
        }))
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::ErrResp;

    use crate::err::Code;

    use super::*;

    struct BankAccountOperationMock {
        user_account_id: i64,
        bank_account: BankAccount,
    }

    #[async_trait]
    impl BankAccountOperation for BankAccountOperationMock {
        async fn get_bank_account_by_user_account_id(
            &self,
            user_account_id: i64,
        ) -> Result<Option<BankAccount>, ErrResp> {
            if self.user_account_id != user_account_id {
                return Ok(None);
            }
            Ok(Some(self.bank_account.clone()))
        }
    }

    #[tokio::test]

    async fn get_bank_account_by_user_account_id_internal_success() {
        let user_account_id = 64431;
        let bank_account = BankAccount {
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "タナカ　タロウ".to_string(),
        };
        let op_mock = BankAccountOperationMock {
            user_account_id,
            bank_account: bank_account.clone(),
        };

        let result = get_bank_account_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            bank_account,
            resp.1 .0.bank_account.expect("failed to get bank_account")
        );
    }

    #[tokio::test]

    async fn get_bank_account_by_user_account_id_internal_success_no_tenant_id_found() {
        let user_account_id = 64431;
        let bank_account = BankAccount {
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "タナカ　タロウ".to_string(),
        };
        let op_mock = BankAccountOperationMock {
            user_account_id,
            bank_account: bank_account.clone(),
        };
        let dummy_id = user_account_id + 451;

        let result = get_bank_account_by_user_account_id_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(None, resp.1 .0.bank_account);
    }

    #[tokio::test]
    async fn get_bank_account_by_user_account_id_internal_fail_user_account_id_is_zero() {
        let user_account_id = 0;
        let bank_account = BankAccount {
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "タナカ　タロウ".to_string(),
        };
        let op_mock = BankAccountOperationMock {
            user_account_id,
            bank_account,
        };

        let result = get_bank_account_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_bank_account_by_user_account_id_internal_fail_user_account_id_is_negative() {
        let user_account_id = -1;
        let bank_account = BankAccount {
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "タナカ　タロウ".to_string(),
        };
        let op_mock = BankAccountOperationMock {
            user_account_id,
            bank_account,
        };

        let result = get_bank_account_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }
}
