// Copyright 2021 Ken Miura

use std::io::Cursor;

use async_session::serde_json;
use axum::{
    extract::{ContentLengthLimit, Multipart},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use common::{ApiError, ErrResp, RespResult};
use image::{ImageError, ImageFormat};
use serde::Serialize;

use crate::{
    err::{self, unexpected_err_resp, Code},
    util::{
        session::User,
        validator::{
            file_name_validator::validate_extension_is_jpeg,
            identity_validator::{validate_identity, IdentityValidationError},
        },
        Identity, JAPANESE_TIME_ZONE,
    },
};

/// 身分証の画像ファイルのバイト単位での最大値（4MB）
const MAX_IDENTITY_IMAGE_SIZE_IN_BYTES: usize = 4 * 1024 * 1024;

pub(crate) async fn post_identity(
    User { account_id }: User,
    ContentLengthLimit(mut multipart): ContentLengthLimit<
        Multipart,
        {
            9 * 1024 * 1024 /* 9mb */
        },
    >,
) -> RespResult<IdentityResult> {
    let mut identity_option = None;
    let mut identity_image1_option = None;
    let mut identity_image2_option = None;
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("failed to get next_field: {}", e);
        unexpected_err_resp()
    })? {
        let name = match field.name() {
            Some(n) => n.to_string(),
            None => {
                tracing::error!("failed to get name in field");
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoNameFound as u32,
                    }),
                ));
            }
        };
        let file_name_option = field.file_name().map(|s| s.to_string());
        let data = field.bytes().await.map_err(|e| {
            tracing::error!("failed to get data in field: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::DataParseFailure as u32,
                }),
            )
        })?;
        if name == "identity" {
            let identity_json_str = std::str::from_utf8(&data).map_err(|e| {
                tracing::error!("invalid utf-8 sequence: {}", e);
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidUtf8Sequence as u32,
                    }),
                )
            })?;
            let identity = serde_json::from_str::<Identity>(identity_json_str).map_err(|e| {
                tracing::error!("invalid Identity JSON object: {}", e);
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidIdentityJson as u32,
                    }),
                )
            })?;
            let current_date = Utc::now()
                .with_timezone(&JAPANESE_TIME_ZONE.to_owned())
                .naive_local()
                .date();
            let _ = validate_identity(&identity, &current_date).map_err(|e| {
                tracing::error!("invalid identity: {}", e);
                create_invalid_identity_err(&e)
            })?;
            identity_option = Some(trim_space_from_identity(identity));
        } else if name == "identity-image1" {
            let file_name = match file_name_option {
                Some(f) => f,
                None => {
                    tracing::error!("failed to get file name in field");
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ApiError {
                            code: Code::NoFileNameFound as u32,
                        }),
                    ));
                }
            };
            let _ = validate_extension_is_jpeg(&file_name).map_err(|e| {
                tracing::error!("invalid file name ({}): {}", file_name, e);
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NotJpegExtension as u32,
                    }),
                )
            })?;
            if data.len() > MAX_IDENTITY_IMAGE_SIZE_IN_BYTES {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::ExceedMaxIdentityImageSizeLimit as u32,
                    }),
                ));
            };
            let img = image::io::Reader::with_format(Cursor::new(data), ImageFormat::Jpeg)
                .decode()
                .map_err(|e| {
                    tracing::error!("failed to decode jpeg image: {}", e);
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
            let mut bytes: Vec<u8> = Vec::new();
            img.write_to(&mut bytes, image::ImageOutputFormat::Png)
                .map_err(|e| {
                    tracing::error!("failed to write image on buffer: {}", e);
                    unexpected_err_resp()
                })?;
            identity_image1_option = Some(bytes);
        } else if name == "identity-image2" {
            println!("identity-image2");
            let bytes: Vec<u8> = Vec::new();
            identity_image2_option = Some(bytes);
        } else {
            tracing::error!("invalid name in field: {}", name);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::InvalidNameInField as u32,
                }),
            ));
        }
    }
    println!("account_id: {}", account_id);
    println!("{:?}", identity_option);
    println!("{:?}", identity_image1_option);
    println!("{:?}", identity_image2_option);
    Ok((StatusCode::OK, Json(IdentityResult {})))
}

#[derive(Serialize, Debug)]
pub(crate) struct IdentityResult {}

fn create_invalid_identity_err(e: &IdentityValidationError) -> ErrResp {
    let code;
    match e {
        IdentityValidationError::InvalidLastNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = err::Code::InvalidLastNameLength,
        IdentityValidationError::IllegalCharInLastName(_) => {
            code = err::Code::IllegalCharInLastName
        }
        IdentityValidationError::InvalidFirstNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = err::Code::InvalidFirstNameLength,
        IdentityValidationError::IllegalCharInFirstName(_) => {
            code = err::Code::IllegalCharInFirstName
        }
        IdentityValidationError::InvalidLastNameFuriganaLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = err::Code::InvalidLastNameFuriganaLength,
        IdentityValidationError::IllegalCharInLastNameFurigana(_) => {
            code = err::Code::IllegalCharInLastNameFurigana
        }
        IdentityValidationError::InvalidFirstNameFuriganaLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = err::Code::InvalidFirstNameFuriganaLength,
        IdentityValidationError::IllegalCharInFirstNameFurigana(_) => {
            code = err::Code::IllegalCharInFirstNameFurigana
        }
        IdentityValidationError::IllegalDate {
            year: _,
            month: _,
            day: _,
        } => code = err::Code::IllegalDate,
        IdentityValidationError::IllegalAge {
            birth_year: _,
            birth_month: _,
            birth_day: _,
            current_year: _,
            current_month: _,
            current_day: _,
        } => code = err::Code::IllegalAge,
        IdentityValidationError::InvalidPrefecture(_) => code = err::Code::InvalidPrefecture,
        IdentityValidationError::InvalidCityLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = err::Code::InvalidCityLength,
        IdentityValidationError::IllegalCharInCity(_) => code = err::Code::IllegalCharInCity,
        IdentityValidationError::InvalidAddressLine1Length {
            length: _,
            min_length: _,
            max_length: _,
        } => code = err::Code::InvalidAddressLine1Length,
        IdentityValidationError::IllegalCharInAddressLine1(_) => {
            code = err::Code::IllegalCharInAddressLine1
        }
        IdentityValidationError::InvalidAddressLine2Length {
            length: _,
            min_length: _,
            max_length: _,
        } => code = err::Code::InvalidAddressLine2Length,
        IdentityValidationError::IllegalCharInAddressLine2(_) => {
            code = err::Code::IllegalCharInAddressLine2
        }
        IdentityValidationError::InvalidTelNumFormat(_) => code = err::Code::InvalidTelNumFormat,
    }
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError { code: code as u32 }),
    )
}

fn trim_space_from_identity(identity: Identity) -> Identity {
    Identity {
        last_name: identity.last_name.trim().to_string(),
        first_name: identity.first_name.trim().to_string(),
        last_name_furigana: identity.last_name_furigana.trim().to_string(),
        first_name_furigana: identity.first_name_furigana.trim().to_string(),
        date_of_birth: identity.date_of_birth,
        prefecture: identity.prefecture.trim().to_string(),
        city: identity.city.trim().to_string(),
        address_line1: identity.address_line1.trim().to_string(),
        address_line2: identity
            .address_line2
            .map(|address_line2| address_line2.trim().to_string()),
        telephone_number: identity.telephone_number.trim().to_string(),
    }
}
