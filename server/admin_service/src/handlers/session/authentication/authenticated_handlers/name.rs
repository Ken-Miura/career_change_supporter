// Copyright 2023 Ken Miura

use common::ErrResp;
use entity::sea_orm::DatabaseConnection;
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
    let model = super::find_identity_by_user_account_id(pool, user_account_id).await?;
    let model = model.ok_or_else(|| {
        error!("no identity (user_account_id: {}) found", user_account_id);
        unexpected_err_resp()
    })?;
    Ok(Name {
        first_name_furigana: model.first_name_furigana,
        last_name_furigana: model.last_name_furigana,
    })
}
