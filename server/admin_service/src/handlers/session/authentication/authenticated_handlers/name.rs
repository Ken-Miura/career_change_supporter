// Copyright 2023 Ken Miura

use common::ErrResp;
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use tracing::error;

use crate::err::unexpected_err_resp;

#[derive(Clone)]
pub(super) struct Name {
    pub(super) last_name_furigana: String,
    pub(super) first_name_furigana: String,
}

pub(super) async fn find_name_by_user_account_id(
    pool: &DatabaseConnection,
    user_account_id: i64,
) -> Result<Name, ErrResp> {
    let id = entity::identity::Entity::find_by_id(user_account_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find identity (user_account_id: {}): {}",
                user_account_id, e
            );
            unexpected_err_resp()
        })?;
    let id = id.ok_or_else(|| {
        error!("no identity (user_account_id: {}) found", user_account_id);
        unexpected_err_resp()
    })?;
    Ok(Name {
        first_name_furigana: id.first_name_furigana,
        last_name_furigana: id.last_name_furigana,
    })
}
