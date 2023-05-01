// Copyright 2023 Ken Miura

use common::ErrRespStruct;
use entity::{
    sea_orm::{DatabaseTransaction, EntityTrait, QuerySelect},
    user_account,
};
use tracing::error;

use crate::err::unexpected_err_resp;

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
