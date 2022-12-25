// Copyright 2022 Ken Miura

use common::ErrResp;
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use tracing::error;

use crate::err::unexpected_err_resp;

/// Identityが存在するか確認する。存在する場合、trueを返す。そうでない場合、falseを返す。
///
/// 個人情報の登録をしていないと使えないAPIに関して、処理を継続してよいか確認するために利用する。
pub(crate) async fn check_if_identity_exists(
    pool: &DatabaseConnection,
    account_id: i64,
) -> Result<bool, ErrResp> {
    let model = entity::prelude::Identity::find_by_id(account_id)
        .one(pool)
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
