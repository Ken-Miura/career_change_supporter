// Copyright 2023 Ken Miura

pub(crate) mod authenticated_handlers;
pub(crate) mod login;
pub(crate) mod logout;
pub(crate) mod mfa;
mod user_operation;

use chrono::{DateTime, FixedOffset};
use common::ErrResp;
use entity::sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use tracing::error;

use crate::err::unexpected_err_resp;

async fn update_last_login(
    account_id: i64,
    login_time: &DateTime<FixedOffset>,
    pool: &DatabaseConnection,
) -> Result<(), ErrResp> {
    let user_account_model = entity::user_account::ActiveModel {
        user_account_id: Set(account_id),
        last_login_time: Set(Some(*login_time)),
        ..Default::default()
    };
    let _ = user_account_model.update(pool).await.map_err(|e| {
        error!(
            "failed to update user_account (user_account_id: {}): {}",
            account_id, e
        );
        unexpected_err_resp()
    })?;
    Ok(())
}
