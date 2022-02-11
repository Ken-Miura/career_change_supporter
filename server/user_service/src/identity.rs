// Copyright 2021 Ken Miura

use std::error::Error;
use std::io::Cursor;

use async_session::serde_json;
use axum::async_trait;
use axum::{
    extract::{ContentLengthLimit, Multipart},
    http::StatusCode,
    Json,
};
use bytes::Bytes;
use chrono::{NaiveDate, Utc};
use common::{
    smtp::{SendMail, SmtpClient, SOCKET_FOR_SMTP_SERVER},
    ApiError, DatabaseConnection, ErrResp, RespResult,
};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection,
};
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
    ContentLengthLimit(multipart): ContentLengthLimit<
        Multipart,
        {
            9 * 1024 * 1024 /* 9mb */
        },
    >,
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<IdentityResult> {
    let multipart_wrapper = MultipartWrapperImpl { multipart };
    let current_date = Utc::now()
        .with_timezone(&JAPANESE_TIME_ZONE.to_owned())
        .naive_local()
        .date();
    let (identity, identity_image1, identity_image2_option) =
        handle_multipart(multipart_wrapper, current_date).await?;

    let op = SubmitIdentityOperationImpl::new(conn);
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
    let submitted_identity = SubmittedIdentity {
        account_id,
        identity,
        identity_image1: ("image1-test.png".to_string(), identity_image1),
        identity_image2: identity_image2_option.map(|image| ("image2-test.png".to_string(), image)),
    };
    let result = post_identity_internal(submitted_identity, op, smtp_client).await?;
    Ok(result)
}

#[derive(Serialize, Debug)]
pub(crate) struct IdentityResult {}

#[async_trait]
trait MultipartWrapper {
    async fn next_field(&mut self) -> Result<Option<IdentityField>, ErrResp>;
}

struct MultipartWrapperImpl {
    multipart: Multipart,
}

#[async_trait]
impl MultipartWrapper for MultipartWrapperImpl {
    async fn next_field(&mut self) -> Result<Option<IdentityField>, ErrResp> {
        let field_option = self.multipart.next_field().await.map_err(|e| {
            tracing::error!("failed to get next_field: {}", e);
            unexpected_err_resp()
        })?;
        match field_option {
            Some(f) => {
                let name = f.name().map(|s| s.to_string());
                let file_name = f.file_name().map(|s| s.to_string());
                let data = f.bytes().await.map_err(|e| e.into());
                Ok(Some(IdentityField {
                    name,
                    file_name,
                    data,
                }))
            }
            None => Ok(None),
        }
    }
}

struct IdentityField {
    name: Option<String>,
    file_name: Option<String>,
    data: Result<Bytes, Box<dyn Error>>,
}

async fn handle_multipart(
    mut multipart: impl MultipartWrapper,
    current_date: NaiveDate,
) -> Result<(Identity, Vec<u8>, Option<Vec<u8>>), ErrResp> {
    let mut identity_option = None;
    let mut identity_image1_option = None;
    let mut identity_image2_option = None;
    while let Some(field) = multipart.next_field().await? {
        let name = field.name.ok_or_else(|| {
            tracing::error!("failed to get name in field");
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::NoNameFound as u32,
                }),
            )
        })?;
        let file_name_option = field.file_name;
        let data = field.data.map_err(|e| {
            tracing::error!("failed to get data in field: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::DataParseFailure as u32,
                }),
            )
        })?;
        if name == "identity" {
            let identity = extract_identity(data)?;
            let _ = validate_identity(&identity, &current_date).map_err(|e| {
                tracing::error!("invalid identity: {}", e);
                create_invalid_identity_err(&e)
            })?;
            identity_option = Some(trim_space_from_identity(identity));
        } else if name == "identity-image1" {
            let _ = validate_identity_image_file_name(file_name_option)?;
            let _ = validate_identity_image_size(data.len())?;
            let png_binary = convert_jpeg_to_png(data)?;
            identity_image1_option = Some(png_binary);
        } else if name == "identity-image2" {
            let _ = validate_identity_image_file_name(file_name_option)?;
            let _ = validate_identity_image_size(data.len())?;
            let png_binary = convert_jpeg_to_png(data)?;
            identity_image2_option = Some(png_binary);
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
    let (identity, identity_image1) =
        ensure_mandatory_params_exist(identity_option, identity_image1_option)?;
    Ok((identity, identity_image1, identity_image2_option))
}

fn extract_identity(data: Bytes) -> Result<Identity, ErrResp> {
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
    Ok(identity)
}

fn validate_identity_image_file_name(file_name_option: Option<String>) -> Result<(), ErrResp> {
    let file_name = match file_name_option {
        Some(name) => name,
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
    Ok(())
}

fn validate_identity_image_size(size: usize) -> Result<(), ErrResp> {
    if size > MAX_IDENTITY_IMAGE_SIZE_IN_BYTES {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ExceedMaxIdentityImageSizeLimit as u32,
            }),
        ));
    };
    Ok(())
}

// 画像ファイルの中のメタデータに悪意ある内容が含まれている場合が考えられるので、画像情報以外のメタデータを取り除く必要がある。
// メタデータを取り除くのに画像形式を変換するのが最も容易な実装のため、画像形式の変換を行っている。
fn convert_jpeg_to_png(data: Bytes) -> Result<Vec<u8>, ErrResp> {
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
    Ok(bytes)
}

fn ensure_mandatory_params_exist(
    identity_option: Option<Identity>,
    identity_image1_option: Option<Vec<u8>>,
) -> Result<(Identity, Vec<u8>), ErrResp> {
    let identity = match identity_option {
        Some(id) => id,
        None => {
            tracing::error!("no identity found");
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::NoIdentityFound as u32,
                }),
            ));
        }
    };
    let identity_image1 = match identity_image1_option {
        Some(image1) => image1,
        None => {
            tracing::error!("no identity-image1 found");
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::NoIdentityImage1Found as u32,
                }),
            ));
        }
    };
    Ok((identity, identity_image1))
}

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

async fn post_identity_internal(
    submitted_identity: SubmittedIdentity,
    op: impl SubmitIdentityOperation,
    send_mail: impl SendMail,
) -> RespResult<IdentityResult> {
    let account_id = submitted_identity.account_id;
    let identity_exists = async move {
        let exists = op.check_if_identity_already_exists(account_id)?;
        if exists {
            let _ = op.update_identity(submitted_identity)?;
        } else {
            let _ = op.post_identity(submitted_identity)?;
        };
        Ok(exists)
    }
    .await?;
    let text = identity_exists.to_string() + &account_id.to_string();
    let _ = send_mail.send_mail("to", "from", "subject", &text)?;
    Ok((StatusCode::OK, Json(IdentityResult {})))
}

struct SubmittedIdentity {
    account_id: i32,
    identity: Identity,
    identity_image1: (String, Vec<u8>),
    identity_image2: Option<(String, Vec<u8>)>,
}

trait SubmitIdentityOperation {
    fn check_if_identity_already_exists(&self, account_id: i32) -> Result<bool, ErrResp>;
    fn post_identity(&self, identity: SubmittedIdentity) -> Result<(), ErrResp>;
    fn update_identity(&self, identity: SubmittedIdentity) -> Result<(), ErrResp>;
}

struct SubmitIdentityOperationImpl {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl SubmitIdentityOperationImpl {
    fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { conn }
    }
}

impl SubmitIdentityOperation for SubmitIdentityOperationImpl {
    fn check_if_identity_already_exists(&self, account_id: i32) -> Result<bool, ErrResp> {
        todo!()
    }

    fn post_identity(&self, identity: SubmittedIdentity) -> Result<(), ErrResp> {
        todo!()
    }

    fn update_identity(&self, identity: SubmittedIdentity) -> Result<(), ErrResp> {
        todo!()
    }
}
