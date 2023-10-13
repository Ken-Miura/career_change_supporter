// Copyright 2023 Ken Miura

pub(crate) mod bank_account;
mod bank_account_validator;

use axum::async_trait;
use axum::{extract::State, http::StatusCode, Json};
use common::util::BankAccount;
use common::{ErrResp, RespResult};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;
use crate::handlers::session::authentication::authenticated_handlers::authenticated_users::user::User;

pub(crate) async fn get_reward(
    User { user_info }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<RewardResult> {
    let reward_op = RewardOperationImpl::new(pool);
    handle_reward_req(user_info.account_id, reward_op).await
}

async fn handle_reward_req(
    account_id: i64,
    reward_op: impl RewardOperation,
) -> RespResult<RewardResult> {
    let bank_account = reward_op
        .find_bank_account_by_account_id(account_id)
        .await?;
    Ok((StatusCode::OK, Json(RewardResult { bank_account })))
}

#[derive(Serialize, Debug)]
pub(crate) struct RewardResult {
    bank_account: Option<BankAccount>,
}

#[async_trait]
trait RewardOperation {
    async fn find_bank_account_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<BankAccount>, ErrResp>;
}

struct RewardOperationImpl {
    pool: DatabaseConnection,
}

impl RewardOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RewardOperation for RewardOperationImpl {
    async fn find_bank_account_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<BankAccount>, ErrResp> {
        let model = entity::bank_account::Entity::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find bank_account (account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| BankAccount {
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

    use super::*;

    struct RewardOperationMock {
        account_id: i64,
        bank_account: Option<BankAccount>,
    }

    #[async_trait]
    impl RewardOperation for RewardOperationMock {
        async fn find_bank_account_by_account_id(
            &self,
            account_id: i64,
        ) -> Result<Option<BankAccount>, ErrResp> {
            assert_eq!(self.account_id, account_id);
            Ok(self.bank_account.clone())
        }
    }

    #[tokio::test]
    async fn handle_reward_req_returns_empty_rewards() {
        let account_id = 9853;
        let bank_account = None;
        let reward_op = RewardOperationMock {
            account_id,
            bank_account,
        };

        let result = handle_reward_req(account_id, reward_op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(None, result.1 .0.bank_account);
    }

    #[tokio::test]
    async fn handle_reward_req_returns_bank_account() {
        let account_id = 9853;
        let bank_account = Some(BankAccount {
            bank_code: "0005".to_string(),
            branch_code: "045".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "タナカ　タロウ".to_string(),
        });
        let reward_op = RewardOperationMock {
            account_id,
            bank_account: bank_account.clone(),
        };

        let result = handle_reward_req(account_id, reward_op)
            .await
            .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(bank_account, result.1 .0.bank_account);
    }
}
