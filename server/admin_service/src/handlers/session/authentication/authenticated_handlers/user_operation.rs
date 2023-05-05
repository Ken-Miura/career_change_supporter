// Copyright 2023 Ken Miura

use axum::async_trait;
use chrono::{DateTime, FixedOffset};
use common::{ErrResp, ErrRespStruct};
use entity::{
    sea_orm::{DatabaseConnection, DatabaseTransaction, EntityTrait, QuerySelect},
    user_account,
};
use serde::Deserialize;
use tracing::error;

use crate::err::unexpected_err_resp;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(super) struct UserInfo {
    pub(super) account_id: i64,
    pub(super) email_address: String,
    pub(super) mfa_enabled_at: Option<DateTime<FixedOffset>>,
    pub(super) disabled_at: Option<DateTime<FixedOffset>>,
}

#[async_trait]
pub(super) trait FindUserInfoOperation {
    async fn find_user_info_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<UserInfo>, ErrResp>;
}

pub(super) struct FindUserInfoOperationImpl<'a> {
    pool: &'a DatabaseConnection,
}

impl<'a> FindUserInfoOperationImpl<'a> {
    pub(super) fn new(pool: &'a DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> FindUserInfoOperation for FindUserInfoOperationImpl<'a> {
    async fn find_user_info_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<UserInfo>, ErrResp> {
        let model = entity::user_account::Entity::find_by_id(account_id)
            .one(self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find user_account (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| UserInfo {
            account_id: m.user_account_id,
            email_address: m.email_address,
            mfa_enabled_at: m.mfa_enabled_at,
            disabled_at: m.disabled_at,
        }))
    }
}

/// 承認、拒否を行う際にユーザーがアカウントを削除しないことを保証するために明示的に共有ロックを取得し、user_accountを取得する
pub(super) async fn find_user_model_by_user_account_id_with_shared_lock(
    txn: &DatabaseTransaction,
    user_account_id: i64,
) -> Result<Option<user_account::Model>, ErrRespStruct> {
    let user_model_option = user_account::Entity::find_by_id(user_account_id)
        .lock_shared()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find user_account (user_account_id: {}): {}",
                user_account_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(user_model_option)
}
