// Copyright 2022 Ken Miura

use common::ErrRespStruct;
use entity::{
    create_career_req,
    sea_orm::{DatabaseTransaction, EntityTrait, QuerySelect},
};
use tracing::error;

use crate::err::unexpected_err_resp;

pub(crate) mod career_images;
pub(crate) mod create_career_request_approval;
pub(crate) mod create_career_request_detail;
pub(crate) mod create_career_request_rejection;
pub(crate) mod create_career_requests;

async fn find_create_career_req_model_by_create_career_req_id_with_exclusive_lock(
    txn: &DatabaseTransaction,
    create_career_req_id: i64,
) -> Result<create_career_req::Model, ErrRespStruct> {
    let req_option = create_career_req::Entity::find_by_id(create_career_req_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find create_career_req (create_career_req_id: {}): {}",
                create_career_req_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    let req = req_option.ok_or_else(|| {
        error!(
            "no create_career_req (create_career_req_id: {}) found",
            create_career_req_id
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(req)
}
