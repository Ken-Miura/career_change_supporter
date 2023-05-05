// Copyright 2023 Ken Miura

pub(crate) mod approval;
pub(crate) mod career_images;
pub(crate) mod detail;
pub(crate) mod list;
pub(crate) mod rejection;

use common::ErrRespStruct;
use entity::sea_orm::{DatabaseTransaction, EntityTrait};
use tracing::error;

use crate::err::unexpected_err_resp;

async fn delete_create_career_req(
    create_career_req_id: i64,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    let _ = entity::create_career_req::Entity::delete_by_id(create_career_req_id)
        .exec(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to delete create_career_req (create_career_req_id: {}): {}",
                create_career_req_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(())
}
