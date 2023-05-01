// Copyright 2022 Ken Miura

pub(crate) mod create_request;
pub(crate) mod identity_images;
pub(crate) mod update_request;

use common::{
    storage::{self, IDENTITY_IMAGES_BUCKET_NAME},
    ErrRespStruct,
};
use tracing::error;

use crate::err::unexpected_err_resp;

async fn delete_identity_images(
    user_account_id: i64,
    image1_file_name_without_ext: String,
    image2_file_name_without_ext: Option<String>,
) -> Result<(), ErrRespStruct> {
    let image1_key = format!("{}/{}.png", user_account_id, image1_file_name_without_ext);
    storage::delete_object(IDENTITY_IMAGES_BUCKET_NAME, image1_key.as_str())
        .await
        .map_err(|e| {
            error!(
                "failed to delete identity image1 (key: {}): {}",
                image1_key, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;

    if let Some(image2_file_name_without_ext) = image2_file_name_without_ext {
        let image2_key = format!("{}/{}.png", user_account_id, image2_file_name_without_ext);
        storage::delete_object(IDENTITY_IMAGES_BUCKET_NAME, image2_key.as_str())
            .await
            .map_err(|e| {
                error!(
                    "failed to delete identity image2 (key: {}): {}",
                    image2_key, e
                );
                ErrRespStruct {
                    err_resp: unexpected_err_resp(),
                }
            })?;
    }

    Ok(())
}
