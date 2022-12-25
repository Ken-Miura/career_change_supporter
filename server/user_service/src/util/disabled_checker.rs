// Copyright 2022 Ken Miura

use chrono::{DateTime, FixedOffset};
use common::ErrResp;
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use tracing::error;

use crate::err::unexpected_err_resp;

#[derive(Clone, Debug)]
pub(crate) struct UserAccount {
    pub(crate) email_address: String,
    pub(crate) disabled_at: Option<DateTime<FixedOffset>>,
}

/// ユーザーが存在する場合、[UserAccount]を返す。存在しない場合、Noneを返す。
async fn get_if_user_exists(
    pool: &DatabaseConnection,
    user_account_id: i64,
) -> Result<Option<UserAccount>, ErrResp> {
    let model = entity::prelude::UserAccount::find_by_id(user_account_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find user_account (user_account_id): {}): {}",
                user_account_id, e
            );
            unexpected_err_resp()
        })?;
    Ok(model.map(|m| UserAccount {
        email_address: m.email_address,
        disabled_at: m.disabled_at,
    }))
}

/// ユーザーが利用可能な場合（UserAccountが存在し、かつdisabled_atがNULLである場合）、[UserAccount]を返す
pub(crate) async fn get_if_user_account_is_available(
    pool: &DatabaseConnection,
    user_account_id: i64,
) -> Result<Option<UserAccount>, ErrResp> {
    let user = get_if_user_exists(pool, user_account_id).await?;
    let result = match user {
        Some(u) => {
            if u.disabled_at.is_none() {
                Some(u)
            } else {
                None
            }
        }
        None => None,
    };
    Ok(result)
}
