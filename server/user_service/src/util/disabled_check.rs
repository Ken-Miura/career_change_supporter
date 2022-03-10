// Copyright 2021 Ken Miura

use axum::async_trait;
use common::ErrResp;
use entity::{
    prelude::UserAccount,
    sea_orm::{DatabaseConnection, EntityTrait},
};

use crate::err::unexpected_err_resp;

#[async_trait]
pub(crate) trait DisabledCheckOperation {
    /// アカウントが無効化されているかどうか
    ///
    /// - アカウントが存在しない場合、Noneを返す
    /// - アカウントが存在する場合で
    ///   - アカウントが無効化されている場合、Some(true)を返す
    ///   - アカウントが無効化されていない場合、Some(false)を返す
    async fn check_if_account_is_disabled(&self, account_id: i32) -> Result<Option<bool>, ErrResp>;
}

pub(crate) struct DisabledCheckOperationImpl {
    pool: DatabaseConnection,
}

impl DisabledCheckOperationImpl {
    pub(crate) fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DisabledCheckOperation for DisabledCheckOperationImpl {
    async fn check_if_account_is_disabled(&self, account_id: i32) -> Result<Option<bool>, ErrResp> {
        let model = UserAccount::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("failed to find account (accound id: {}): {}", account_id, e);
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.disabled_at.is_some()))
    }
}
