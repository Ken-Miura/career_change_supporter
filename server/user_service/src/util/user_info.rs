// Copyright 2023 Ken Miura

use axum::async_trait;
use chrono::{DateTime, FixedOffset};
use common::ErrResp;
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use tracing::error;

use crate::err::unexpected_err_resp;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct UserInfo {
    pub(crate) account_id: i64,
    pub(crate) email_address: String,
    pub(crate) mfa_enabled_at: Option<DateTime<FixedOffset>>,
    pub(crate) disabled_at: Option<DateTime<FixedOffset>>,
}

#[async_trait]
pub(crate) trait FindUserInfoOperation {
    async fn find_user_info_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<UserInfo>, ErrResp>;
}

pub(crate) struct FindUserInfoOperationImpl<'a> {
    pool: &'a DatabaseConnection,
}

impl<'a> FindUserInfoOperationImpl<'a> {
    pub(crate) fn new(pool: &'a DatabaseConnection) -> Self {
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
