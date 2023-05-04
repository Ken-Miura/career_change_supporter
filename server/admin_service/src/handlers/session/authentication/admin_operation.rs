// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::{http::StatusCode, Json};
use chrono::{DateTime, FixedOffset};
use common::{ApiError, ErrResp};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::Deserialize;
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(super) struct AdminInfo {
    pub(super) account_id: i64,
    pub(super) email_address: String,
    pub(super) mfa_enabled_at: Option<DateTime<FixedOffset>>,
}

pub(super) async fn get_admin_info_by_account_id(
    account_id: i64,
    op: &impl FindAdminInfoOperation,
) -> Result<AdminInfo, ErrResp> {
    let admin_info = op.find_admin_info_by_account_id(account_id).await?;
    let admin_info = admin_info.ok_or_else(|| {
        error!("no account ({}) found", account_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoAccountFound as u32,
            }),
        )
    })?;
    Ok(admin_info)
}

#[async_trait]
pub(super) trait FindAdminInfoOperation {
    async fn find_admin_info_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<AdminInfo>, ErrResp>;
}

pub(super) struct FindAdminInfoOperationImpl<'a> {
    pool: &'a DatabaseConnection,
}

impl<'a> FindAdminInfoOperationImpl<'a> {
    pub(super) fn new(pool: &'a DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> FindAdminInfoOperation for FindAdminInfoOperationImpl<'a> {
    async fn find_admin_info_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<AdminInfo>, ErrResp> {
        let model = entity::admin_account::Entity::find_by_id(account_id)
            .one(self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find admin_account (admin_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| AdminInfo {
            account_id: m.admin_account_id,
            email_address: m.email_address,
            mfa_enabled_at: m.mfa_enabled_at,
        }))
    }
}
