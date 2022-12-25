// Copyright 2021 Ken Miura

use axum::async_trait;
use common::ErrResp;
use entity::{
    prelude::UserAccount,
    sea_orm::{DatabaseConnection, EntityTrait},
};
use tracing::error;

use crate::err::unexpected_err_resp;

#[async_trait]
pub(super) trait DisabledCheckOperation {
    /// アカウントが無効化されているかどうか
    ///
    /// - アカウントが存在しない場合、Noneを返す
    /// - アカウントが存在する場合で
    ///   - アカウントが無効化されている場合、Some(true)を返す
    ///   - アカウントが無効化されていない場合、Some(false)を返す
    async fn check_if_account_is_disabled(&self, account_id: i64) -> Result<Option<bool>, ErrResp>;
}

pub(super) struct DisabledCheckOperationImpl<'a> {
    pool: &'a DatabaseConnection,
}

impl<'a> DisabledCheckOperationImpl<'a> {
    pub(super) fn new(pool: &'a DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> DisabledCheckOperation for DisabledCheckOperationImpl<'a> {
    async fn check_if_account_is_disabled(&self, account_id: i64) -> Result<Option<bool>, ErrResp> {
        let model = UserAccount::find_by_id(account_id)
            .one(self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find user_account (user_accound_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.disabled_at.is_some()))
    }
}
