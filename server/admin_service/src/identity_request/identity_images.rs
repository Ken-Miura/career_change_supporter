// Copyright 2021 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::Path, Json};
use common::{
    err::Code::InvalidUuidFormat,
    storage::{download_object, IDENTITY_IMAGES_BUCKET_NAME},
    util::validator::uuid_validator::validate_uuid,
    ApiError, ErrResp,
};
use headers::{HeaderMap, HeaderValue};
use tracing::error;

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
        error!("failed to validate image name ({}): {}", image_name, e);
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
        error!("failed to create header value: {}", e);
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
                error!("failed to download object (image key: {}): {}", key, e);
                unexpected_err_resp()
            })?;
        Ok(image_binary)
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use axum::async_trait;
    use axum::http::StatusCode;
    use common::{err::Code::InvalidUuidFormat, ErrResp};
    use headers::HeaderValue;
    use image::{ImageBuffer, ImageOutputFormat, RgbImage};

    use super::{get_identity_images_internal, DownloadIdentityImageOperation};

    struct DownloadIdentityImageOperationMock {
        key: String,
        image: Vec<u8>,
    }

    #[async_trait]
    impl DownloadIdentityImageOperation for DownloadIdentityImageOperationMock {
        async fn download_identity_image(&self, key: &str) -> Result<Vec<u8>, ErrResp> {
            assert_eq!(self.key.as_str(), key);
            Ok(self.image.clone())
        }
    }

    fn create_dummy_identity_image() -> Vec<u8> {
        let img: RgbImage = ImageBuffer::new(64, 64);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        let _ = img
            .write_to(&mut bytes, ImageOutputFormat::Png)
            .expect("failed to get Ok");
        bytes.into_inner()
    }

    #[tokio::test]
    async fn get_identity_images_internal_success() {
        let user_account_id = String::from("4123");
        let image_name = String::from("bb2c15c13ebc1d7b2b7c8d53cb252b4c");
        let key = format!("{}/{}.png", user_account_id, image_name);
        let image = create_dummy_identity_image();
        let op_mock = DownloadIdentityImageOperationMock {
            key,
            image: image.clone(),
        };

        let result = get_identity_images_internal(user_account_id, image_name, op_mock).await;

        let resp = result.expect("failed to get Ok");
        let actual_value = resp.0.get("content-type").expect("failed to get value");
        let expected_value = HeaderValue::from_str("image/png").expect("failed to get Ok");
        assert_eq!(&expected_value, actual_value);
        assert_eq!(image, resp.1);
    }

    #[tokio::test]
    async fn get_identity_images_internal_fail_invalid_uuid() {
        let user_account_id = String::from("4123");
        // 31桁でinvalid
        let image_name = String::from("bb2c15c13ebc1d7b2b7c8d53cb252b4");
        let key = format!("{}/{}.png", user_account_id, image_name);
        let image = create_dummy_identity_image();
        let op_mock = DownloadIdentityImageOperationMock {
            key,
            image: image.clone(),
        };

        let result = get_identity_images_internal(user_account_id, image_name, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(InvalidUuidFormat as u32, resp.1 .0.code);
    }
}
