// Copyright 2023 Ken Miura

pub(crate) mod pass_code;

use common::ErrResp;
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use tracing::error;

use crate::err::unexpected_err_resp;

struct MfaInfo {
    base32_encoded_secret: String,
    hashed_recovery_code: Vec<u8>,
}

async fn get_mfa_info_by_account_id(
    account_id: i64,
    pool: &DatabaseConnection,
) -> Result<MfaInfo, ErrResp> {
    let result = entity::mfa_info::Entity::find_by_id(account_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find mfa_info (user_account_id: {}): {}",
                account_id, e
            );
            unexpected_err_resp()
        })?;
    let mi = result.ok_or_else(|| {
        error!("no mfa_info (user_account_id: {}) found", account_id);
        unexpected_err_resp()
    })?;
    Ok(MfaInfo {
        base32_encoded_secret: mi.base32_encoded_secret,
        hashed_recovery_code: mi.hashed_recovery_code,
    })
}
