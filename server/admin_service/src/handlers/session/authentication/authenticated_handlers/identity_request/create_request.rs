// Copyright 2021 Ken Miura

pub(crate) mod approval;
pub(crate) mod detail;
pub(crate) mod list;
pub(crate) mod rejection;
pub(crate) mod users_by_date_of_birth;

use common::ErrRespStruct;
use entity::{
    create_identity_req,
    sea_orm::{DatabaseTransaction, EntityTrait, QuerySelect},
};
use tracing::error;

use crate::err::unexpected_err_resp;

async fn delete_create_identity_req(
    user_account_id: i64,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    let _ = entity::create_identity_req::Entity::delete_by_id(user_account_id)
        .exec(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to delete create_identity_req (user_account_id: {}): {}",
                user_account_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(())
}

async fn find_create_identity_req_model_by_user_account_id_with_exclusive_lock(
    txn: &DatabaseTransaction,
    user_account_id: i64,
) -> Result<create_identity_req::Model, ErrRespStruct> {
    let req_option = create_identity_req::Entity::find_by_id(user_account_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find create_identity_req (user_account_id: {}): {}",
                user_account_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    let req = req_option.ok_or_else(|| {
        error!(
            "no create_identity_req (user_account_id: {}) found",
            user_account_id
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(req)
}
