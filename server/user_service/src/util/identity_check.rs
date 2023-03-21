// Copyright 2023 Ken Miura

use axum::async_trait;
use common::ErrResp;
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use tracing::error;

use crate::err::unexpected_err_resp;

#[async_trait]
pub(super) trait IdentityCheckOperation {
    /// Identityが存在するか確認する。存在する場合、trueを返す。そうでない場合、falseを返す。
    ///
    /// 個人情報の登録をしていないと使えないAPIに関して、処理を継続してよいか確認するために利用する。
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
}

pub(super) struct IdentityCheckOperationImpl<'a> {
    pool: &'a DatabaseConnection,
}

impl<'a> IdentityCheckOperationImpl<'a> {
    pub(super) fn new(pool: &'a DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> IdentityCheckOperation for IdentityCheckOperationImpl<'a> {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        let model = entity::identity::Entity::find_by_id(account_id)
            .one(self.pool)
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
