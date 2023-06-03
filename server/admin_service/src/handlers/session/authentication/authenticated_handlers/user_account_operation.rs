// Copyright 2023 Ken Miura

use axum::async_trait;
use chrono::{DateTime, FixedOffset};
use common::{ErrResp, ErrRespStruct, JAPANESE_TIME_ZONE};
use entity::{
    sea_orm::{
        ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QueryFilter, QuerySelect,
    },
    user_account,
};
use serde::Deserialize;
use tracing::error;

use crate::err::unexpected_err_resp;

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(super) struct UserAccountInfo {
    pub(super) account_id: i64,
    pub(super) email_address: String,
    pub(super) last_login_time: Option<DateTime<FixedOffset>>,
    pub(super) created_at: DateTime<FixedOffset>,
    pub(super) mfa_enabled_at: Option<DateTime<FixedOffset>>,
    pub(super) disabled_at: Option<DateTime<FixedOffset>>,
}

#[async_trait]
pub(super) trait FindUserAccountInfoOperation {
    async fn find_user_account_info_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<UserAccountInfo>, ErrResp>;

    async fn find_user_account_info_by_email_address(
        &self,
        email_address: &str,
    ) -> Result<Option<UserAccountInfo>, ErrResp>;
}

pub(super) struct FindUserAccountInfoOperationImpl<'a> {
    pool: &'a DatabaseConnection,
}

impl<'a> FindUserAccountInfoOperationImpl<'a> {
    pub(super) fn new(pool: &'a DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> FindUserAccountInfoOperation for FindUserAccountInfoOperationImpl<'a> {
    async fn find_user_account_info_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<UserAccountInfo>, ErrResp> {
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
        Ok(model.map(|m| UserAccountInfo {
            account_id: m.user_account_id,
            email_address: m.email_address,
            last_login_time: m
                .last_login_time
                .map(|t| t.with_timezone(&(*JAPANESE_TIME_ZONE))),
            created_at: m.created_at.with_timezone(&(*JAPANESE_TIME_ZONE)),
            mfa_enabled_at: m
                .mfa_enabled_at
                .map(|t| t.with_timezone(&(*JAPANESE_TIME_ZONE))),
            disabled_at: m
                .disabled_at
                .map(|t| t.with_timezone(&(*JAPANESE_TIME_ZONE))),
        }))
    }

    async fn find_user_account_info_by_email_address(
        &self,
        email_address: &str,
    ) -> Result<Option<UserAccountInfo>, ErrResp> {
        let user_accounts = entity::user_account::Entity::find()
            .filter(entity::user_account::Column::EmailAddress.eq(email_address))
            .all(self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter user_account (email_address: {}): {}",
                    email_address, e
                );
                unexpected_err_resp()
            })?;
        if user_accounts.len() > 1 {
            error!(
                "multiple user_account found (email_address: {}, len: {})",
                email_address,
                user_accounts.len()
            );
            return Err(unexpected_err_resp());
        };
        let user_account = user_accounts.get(0);
        Ok(user_account.map(|m| UserAccountInfo {
            account_id: m.user_account_id,
            email_address: m.email_address.clone(),
            last_login_time: m
                .last_login_time
                .map(|t| t.with_timezone(&(*JAPANESE_TIME_ZONE))),
            created_at: m.created_at.with_timezone(&(*JAPANESE_TIME_ZONE)),
            mfa_enabled_at: m
                .mfa_enabled_at
                .map(|t| t.with_timezone(&(*JAPANESE_TIME_ZONE))),
            disabled_at: m
                .disabled_at
                .map(|t| t.with_timezone(&(*JAPANESE_TIME_ZONE))),
        }))
    }
}

/// 承認、拒否を行う際にユーザーがアカウントを削除しないことを保証するために明示的に共有ロックを取得し、user_accountを取得する
pub(super) async fn find_user_account_model_by_user_account_id_with_shared_lock(
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

/// ユーザーアカウントに関連する設定を変更する際、その間にユーザーが自身のアカウントを操作できないように明示的に排他ロックを取得し、user_accountを取得する
pub(super) async fn find_user_account_model_by_user_account_id_with_exclusive_lock(
    txn: &DatabaseTransaction,
    user_account_id: i64,
) -> Result<Option<user_account::Model>, ErrRespStruct> {
    let user_model_option = user_account::Entity::find_by_id(user_account_id)
        .lock_exclusive()
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
