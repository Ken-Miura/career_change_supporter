// Copyright 2023 Ken Miura

use common::ErrRespStruct;
use entity::sea_orm::{DatabaseTransaction, EntityTrait, QuerySelect};
use tracing::error;

use crate::err::unexpected_err_resp;

pub(super) async fn find_document_model_by_user_account_id_with_exclusive_lock(
    txn: &DatabaseTransaction,
    user_account_id: i64,
) -> Result<Option<entity::document::Model>, ErrRespStruct> {
    let doc_option = entity::document::Entity::find_by_id(user_account_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find document (user_account_id: {}): {}",
                user_account_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(doc_option)
}
