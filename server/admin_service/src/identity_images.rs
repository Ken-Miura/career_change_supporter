// Copyright 2021 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::Path, Json};
use common::{
    err::Code::InvalidUuidFormat,
    storage::{download_object, IDENTITY_IMAGES_BUCKET_NAME},
    util::validator::validate_uuid,
    ApiError, ErrResp,
};
use headers::{HeaderMap, HeaderValue};

use crate::{err::unexpected_err_resp, util::session::Admin};

pub(crate) async fn get_identity_images(
    Admin { account_id: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    Path((user_account_id, image_name)): Path<(String, String)>,
) -> Result<(HeaderMap, Vec<u8>), ErrResp> {
    let op = DownloadIdentityImageOperationImpl {};
    get_identity_images_internal(user_account_id, image_name, op).await
}

async fn get_identity_images_internal(
    user_account_id: String,
    image_name: String,
    op: impl DownloadIdentityImageOperation,
) -> Result<(HeaderMap, Vec<u8>), ErrResp> {
    let _ = validate_uuid(&image_name).map_err(|e| {
        tracing::error!("failed to validate image name ({}): {}", image_name, e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: InvalidUuidFormat as u32,
            }),
        )
    })?;
    let key = format!("{}/{}.png", user_account_id, image_name);
    let image_binary = op.download_identity_image(&key).await?;
    let mut headers = HeaderMap::new();
    let val = HeaderValue::from_str("image/png").map_err(|e| {
        tracing::error!("failed to create header value: {}", e);
        unexpected_err_resp()
    })?;
    headers.insert("content-type", val);
    Ok((headers, image_binary))
}

#[async_trait]
trait DownloadIdentityImageOperation {
    async fn download_identity_image(&self, key: &str) -> Result<Vec<u8>, ErrResp>;
}

struct DownloadIdentityImageOperationImpl {}

#[async_trait]
impl DownloadIdentityImageOperation for DownloadIdentityImageOperationImpl {
    async fn download_identity_image(&self, key: &str) -> Result<Vec<u8>, ErrResp> {
        let image_binary = download_object(IDENTITY_IMAGES_BUCKET_NAME, key)
            .await
            .map_err(|e| {
                tracing::error!("failed to download object (image key: {}): {}", key, e);
                unexpected_err_resp()
            })?;
        Ok(image_binary)
    }
}

#[cfg(test)]
mod tests {}
