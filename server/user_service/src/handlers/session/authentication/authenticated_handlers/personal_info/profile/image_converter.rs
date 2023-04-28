// Copyright 2022 Ken Miura

use axum::Json;
use axum::{body::Bytes, http::StatusCode};
use common::{ApiError, ErrResp};
use image::{ImageError, ImageFormat};
use std::io::Cursor;
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

/// jpeg画像をpng画像に変換する<br>
/// <br>
/// 画像ファイルの中のメタデータに悪意ある内容が含まれている場合が考えられるので、画像情報以外のメタデータを取り除く必要がある。
/// メタデータを取り除くのに画像形式を変換するのが最も容易な実装のため、画像形式の変換を行っている。
pub(super) fn convert_jpeg_to_png(data: Bytes) -> Result<Cursor<Vec<u8>>, ErrResp> {
    let img = image::io::Reader::with_format(Cursor::new(data), ImageFormat::Jpeg)
        .decode()
        .map_err(|e| {
            error!("failed to decode jpeg image: {}", e);
            match e {
                ImageError::Decoding(_) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidJpegImage as u32,
                    }),
                ),
                _ => unexpected_err_resp(),
            }
        })?;
    let mut bytes = Cursor::new(vec![]);
    img.write_to(&mut bytes, image::ImageOutputFormat::Png)
        .map_err(|e| {
            error!("failed to write image on buffer: {}", e);
            unexpected_err_resp()
        })?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {

    use image::{ImageBuffer, ImageOutputFormat, RgbImage};

    use super::*;

    #[test]
    fn convert_jpeg_to_png_returns_ok_if_convert_is_success() {
        let jpeg_image = create_dummy_jpeg_image();
        let expected_image = create_converted_image(jpeg_image.clone().into_inner());

        let result = convert_jpeg_to_png(Bytes::from(jpeg_image.into_inner()));

        let result_image = result.expect("failed to get Ok");
        assert_eq!(expected_image, result_image);
    }

    #[test]
    fn convert_jpeg_to_png_returns_err_if_format_other_than_jpg_is_passed() {
        let png_image = create_dummy_png_image();

        let result = convert_jpeg_to_png(Bytes::from(png_image.into_inner()));

        let result = result.expect_err("failed to get Err");
        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1.code, Code::InvalidJpegImage as u32);
    }

    fn create_dummy_jpeg_image() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(128, 128);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        img.write_to(&mut bytes, ImageOutputFormat::Jpeg(85))
            .expect("failed to get Ok");
        bytes
    }

    fn create_dummy_png_image() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(128, 128);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        img.write_to(&mut bytes, ImageOutputFormat::Png)
            .expect("failed to get Ok");
        bytes
    }

    fn create_converted_image(jpg_img: Vec<u8>) -> Cursor<Vec<u8>> {
        let img = image::io::Reader::with_format(Cursor::new(jpg_img), ImageFormat::Jpeg)
            .decode()
            .expect("failed to get Ok");
        let mut png_img = Cursor::new(vec![]);
        img.write_to(&mut png_img, image::ImageOutputFormat::Png)
            .expect("failed to get Ok");
        png_img
    }
}
