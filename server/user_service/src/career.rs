// Copyright 2021 Ken Miura

// TODO: career向けに変更と調整

use std::error::Error;
use std::io::Cursor;

use crate::err::Code::IdentityReqAlreadyExists;
use async_session::serde_json;
use axum::async_trait;
use axum::extract::Extension;
use axum::{
    extract::{ContentLengthLimit, Multipart},
    http::StatusCode,
    Json,
};
use bytes::Bytes;
use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, Utc};
use common::smtp::{ADMIN_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS};
use common::storage::{upload_object, IDENTITY_IMAGES_BUCKET_NAME};
use common::util::{Identity, Ymd};
use common::{
    smtp::{SendMail, SmtpClient, SOCKET_FOR_SMTP_SERVER},
    ApiError, ErrResp, RespResult,
};
use common::{ErrRespStruct, JAPANESE_TIME_ZONE, WEB_SITE_NAME};
use entity::prelude::{CreateIdentityReq, UpdateIdentityReq};
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionError, TransactionTrait,
};
use entity::{create_identity_req, update_identity_req};
use image::{ImageError, ImageFormat};
use serde::Serialize;
use tracing::{error, info};
use uuid::Uuid;

use crate::{
    err::{self, unexpected_err_resp, Code},
    util::{
        session::User,
        validator::{
            file_name_validator::validate_extension_is_jpeg,
            identity_validator::{validate_identity, IdentityValidationError},
        },
    },
};

/// 身分証の画像ファイルのバイト単位での最大値（4MB）
const MAX_CAREER_IMAGE_SIZE_IN_BYTES: usize = 4 * 1024 * 1024;

pub(crate) async fn post_career(
    User { account_id }: User,
    ContentLengthLimit(multipart): ContentLengthLimit<
        Multipart,
        {
            9 * 1024 * 1024 /* 9mb */
        },
    >,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<CareerResult> {
    let multipart_wrapper = MultipartWrapperImpl { multipart };
    let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
    let current_date = current_date_time.naive_local().date();
    let (career_data, career_image1, career_image2_option) = handle_multipart(
        multipart_wrapper,
        MAX_CAREER_IMAGE_SIZE_IN_BYTES,
        current_date,
    )
    .await?;

    let op = SubmitCareerOperationImpl::new(pool);
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
    let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
    let image2_file_name_without_ext = Uuid::new_v4().simple().to_string();
    let submitted_career = SubmittedCareer {
        account_id,
        career_data,
        career_image1: (image1_file_name_without_ext, career_image1),
        career_image2: career_image2_option.map(|image| (image2_file_name_without_ext, image)),
    };
    let result = handle_career_req(submitted_career, current_date_time, op, smtp_client).await?;
    Ok(result)
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct CareerResult {}

#[async_trait]
trait MultipartWrapper {
    async fn next_field(&mut self) -> Result<Option<CareerField>, ErrResp>;
}

struct MultipartWrapperImpl {
    multipart: Multipart,
}

#[async_trait]
impl MultipartWrapper for MultipartWrapperImpl {
    async fn next_field(&mut self) -> Result<Option<CareerField>, ErrResp> {
        let field_option = self.multipart.next_field().await.map_err(|e| {
            error!("failed next_field: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::InvalidMultiPartFormData as u32,
                }),
            )
        })?;
        match field_option {
            Some(f) => {
                let name = f.name().map(|s| s.to_string());
                let file_name = f.file_name().map(|s| s.to_string());
                let data = f.bytes().await.map_err(|e| e.into());
                Ok(Some(CareerField {
                    name,
                    file_name,
                    data,
                }))
            }
            None => Ok(None),
        }
    }
}

struct CareerField {
    name: Option<String>,
    file_name: Option<String>,
    data: Result<Bytes, Box<dyn Error>>,
}

async fn handle_multipart(
    mut multipart: impl MultipartWrapper,
    max_image_size_in_bytes: usize,
    current_date: NaiveDate,
) -> Result<(Identity, Cursor<Vec<u8>>, Option<Cursor<Vec<u8>>>), ErrResp> {
    let mut career_data_option = None;
    let mut career_image1_option = None;
    let mut career_image2_option = None;
    while let Some(field) = multipart.next_field().await? {
        let name = field.name.ok_or_else(|| {
            error!("failed to get \"name\" in field");
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::NoNameFound as u32,
                }),
            )
        })?;
        let file_name_option = field.file_name;
        let data = field.data.map_err(|e| {
            error!("failed to get data in field: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::DataParseFailure as u32,
                }),
            )
        })?;
        if name == "career-data" {
            let identity = extract_career_data(data)?;
            // TODO
            let _ = validate_identity(&identity, &current_date).map_err(|e| {
                error!("invalid identity: {}", e);
                create_invalid_identity_err(&e)
            })?;
            career_data_option = Some(trim_space_from_identity(identity));
        } else if name == "career-image1" {
            // TODO
            let _ = validate_identity_image_file_name(file_name_option)?;
            let _ = validate_identity_image_size(data.len(), max_image_size_in_bytes)?;
            let png_binary = convert_jpeg_to_png(data)?;
            career_image1_option = Some(png_binary);
        } else if name == "career-image2" {
            // TODO
            let _ = validate_identity_image_file_name(file_name_option)?;
            let _ = validate_identity_image_size(data.len(), max_image_size_in_bytes)?;
            let png_binary = convert_jpeg_to_png(data)?;
            career_image2_option = Some(png_binary);
        } else {
            error!("invalid name in field: {}", name);
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::InvalidNameInField as u32,
                }),
            ));
        }
    }
    let (career_data, career_image1) =
        ensure_mandatory_params_exist(career_data_option, career_image1_option)?;
    Ok((career_data, career_image1, career_image2_option))
}

fn extract_career_data(data: Bytes) -> Result<Identity, ErrResp> {
    let identity_json_str = std::str::from_utf8(&data).map_err(|e| {
        error!("invalid utf-8 sequence: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidUtf8Sequence as u32,
            }),
        )
    })?;
    let identity = serde_json::from_str::<Identity>(identity_json_str).map_err(|e| {
        error!("invalid Identity JSON object: {}", e);
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
            error!("failed to get file name in field");
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::NoFileNameFound as u32,
                }),
            ));
        }
    };
    let _ = validate_extension_is_jpeg(&file_name).map_err(|e| {
        error!("invalid file name ({}): {}", file_name, e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NotJpegExtension as u32,
            }),
        )
    })?;
    Ok(())
}

fn validate_identity_image_size(size: usize, max_size_in_bytes: usize) -> Result<(), ErrResp> {
    if size > max_size_in_bytes {
        error!(
            "invalid identity image size (received {} bytes, max size in bytes = {})",
            size, max_size_in_bytes
        );
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
fn convert_jpeg_to_png(data: Bytes) -> Result<Cursor<Vec<u8>>, ErrResp> {
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

fn ensure_mandatory_params_exist(
    career_data_option: Option<Identity>,
    career_image1_option: Option<Cursor<Vec<u8>>>,
) -> Result<(Identity, Cursor<Vec<u8>>), ErrResp> {
    let identity = match career_data_option {
        Some(id) => id,
        None => {
            error!("no identity found");
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::NoIdentityFound as u32,
                }),
            ));
        }
    };
    let career_image1 = match career_image1_option {
        Some(image1) => image1,
        None => {
            error!("no identity-image1 found");
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::NoIdentityImage1Found as u32,
                }),
            ));
        }
    };
    Ok((identity, career_image1))
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

async fn handle_career_req(
    submitted_career: SubmittedCareer,
    current_date_time: DateTime<FixedOffset>,
    op: impl SubmitIdentityOperation,
    send_mail: impl SendMail,
) -> RespResult<CareerResult> {
    let account_id = submitted_career.account_id;
    let career_data_option = op
        .find_identity_by_account_id(account_id)
        .await
        .map_err(|e| {
            error!("failed to find identity (account id: {})", account_id);
            e
        })?;
    let identity_exists = career_data_option.is_some();
    if let Some(identity) = career_data_option {
        info!(
            "request to update identity from account id ({})",
            account_id
        );
        let _ = check_update_identity_requirement(&identity, &submitted_career.career_data)?;
        let _ = handle_update_identity_request(account_id, submitted_career, current_date_time, op)
            .await?;
    } else {
        info!(
            "request to create identity from account id ({})",
            account_id
        );
        let _ = handle_create_identity_request(account_id, submitted_career, current_date_time, op)
            .await?;
    };
    let subject = create_subject(account_id, identity_exists);
    let text = create_text(account_id, identity_exists);
    let _ =
        async { send_mail.send_mail(ADMIN_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS, &subject, &text) }
            .await?;
    Ok((StatusCode::OK, Json(CareerResult {})))
}

fn check_update_identity_requirement(
    identity: &Identity,
    identity_to_update: &Identity,
) -> Result<(), ErrResp> {
    if identity.date_of_birth != identity_to_update.date_of_birth {
        error!(
            "date of birth ({:?}) is not same as submitted one ({:?})",
            identity.date_of_birth, identity_to_update.date_of_birth
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::DateOfBirthIsNotMatch as u32,
            }),
        ));
    }
    if identity.first_name != identity_to_update.first_name {
        error!(
            "first name ({}) is not same as submitted one ({})",
            identity.first_name, identity_to_update.first_name
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::FirstNameIsNotMatch as u32,
            }),
        ));
    }
    if identity == identity_to_update {
        error!("identity ({:?}) is exactly same as submitted one", identity);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoIdentityUpdated as u32,
            }),
        ));
    }
    Ok(())
}

async fn handle_update_identity_request(
    account_id: i64,
    submitted_career: SubmittedCareer,
    current_date_time: DateTime<FixedOffset>,
    op: impl SubmitIdentityOperation,
) -> Result<(), ErrResp> {
    let update_req_exists = op
        .check_if_update_identity_req_already_exists(account_id)
        .await?;
    if update_req_exists {
        error!(
            "update identity request (account id: {}) already exists",
            account_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: IdentityReqAlreadyExists as u32,
            }),
        ));
    }
    let _ = op
        .request_update_identity(submitted_career, current_date_time)
        .await
        .map_err(|e| {
            error!(
                "failed to handle update identity reqest (account id: {})",
                account_id
            );
            e
        })?;
    Ok(())
}

async fn handle_create_identity_request(
    account_id: i64,
    submitted_career: SubmittedCareer,
    current_date_time: DateTime<FixedOffset>,
    op: impl SubmitIdentityOperation,
) -> Result<(), ErrResp> {
    let create_req_exists = op
        .check_if_create_identity_req_already_exists(account_id)
        .await?;
    if create_req_exists {
        error!(
            "create identity request (account id: {}) already exists",
            account_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: IdentityReqAlreadyExists as u32,
            }),
        ));
    }
    let _ = op
        .request_create_identity(submitted_career, current_date_time)
        .await
        .map_err(|e| {
            error!(
                "failed to handle create identity request (account id: {})",
                account_id
            );
            e
        })?;
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
struct SubmittedCareer {
    account_id: i64,
    career_data: Identity,
    career_image1: FileNameAndBinary,
    career_image2: Option<FileNameAndBinary>,
}

type FileNameAndBinary = (String, Cursor<Vec<u8>>);

fn create_subject(id: i64, update: bool) -> String {
    let request_type = if update { "更新" } else { "新規" };
    format!(
        "[{}] ユーザー (id: {}) からの本人確認依頼 ({})",
        WEB_SITE_NAME, id, request_type
    )
}

fn create_text(id: i64, update: bool) -> String {
    let request_type = if update { "更新" } else { "新規" };
    format!(
        "ユーザー (id: {}) からの本人確認依頼 ({}) が届きました。管理者サイトから対応をお願いいたします。",
        id, request_type
    )
}

#[async_trait]
trait SubmitIdentityOperation {
    async fn find_identity_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<Identity>, ErrResp>;
    async fn check_if_create_identity_req_already_exists(
        &self,
        account_id: i64,
    ) -> Result<bool, ErrResp>;
    async fn request_create_identity(
        &self,
        submitted_career: SubmittedCareer,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
    async fn check_if_update_identity_req_already_exists(
        &self,
        account_id: i64,
    ) -> Result<bool, ErrResp>;
    async fn request_update_identity(
        &self,
        submitted_career: SubmittedCareer,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct SubmitCareerOperationImpl {
    pool: DatabaseConnection,
}

impl SubmitCareerOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubmitIdentityOperation for SubmitCareerOperationImpl {
    async fn find_identity_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<Identity>, ErrResp> {
        let model = entity::prelude::Identity::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find identity (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| Identity {
            last_name: m.last_name,
            first_name: m.first_name,
            last_name_furigana: m.last_name_furigana,
            first_name_furigana: m.first_name_furigana,
            date_of_birth: Ymd {
                year: m.date_of_birth.year(),
                month: m.date_of_birth.month(),
                day: m.date_of_birth.day(),
            },
            prefecture: m.prefecture,
            city: m.city,
            address_line1: m.address_line1,
            address_line2: m.address_line2,
            telephone_number: m.telephone_number,
        }))
    }

    async fn check_if_create_identity_req_already_exists(
        &self,
        account_id: i64,
    ) -> Result<bool, ErrResp> {
        let model = CreateIdentityReq::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find create_identity_req (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.is_some())
    }

    async fn request_create_identity(
        &self,
        submitted_career: SubmittedCareer,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        let account_id = submitted_career.account_id;
        let identity = submitted_career.career_data;
        let career_image1 = submitted_career.career_image1;
        let image1_file_name_without_ext = career_image1.0.clone();
        let (career_image2_option, image2_file_name_without_ext) =
            SubmitCareerOperationImpl::extract_file_name(submitted_career.career_image2);
        let _ = self
            .pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let active_model =
                        SubmitCareerOperationImpl::generate_create_identity_req_active_model(
                            account_id,
                            identity,
                            image1_file_name_without_ext,
                            image2_file_name_without_ext,
                            current_date_time,
                        );
                    let _ = active_model.insert(txn).await.map_err(|e| {
                        error!(
                            "failed to insert create_identity_req (user_account_id: {}): {}",
                            account_id, e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;
                    let _ = SubmitCareerOperationImpl::upload_png_images_to_identity_storage(
                        account_id,
                        career_image1,
                        career_image2_option,
                    )
                    .await?;
                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(db_err) => {
                    error!("failed to insert create_identity_req: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    error!("failed to insert create_identity_req: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }

    async fn check_if_update_identity_req_already_exists(
        &self,
        account_id: i64,
    ) -> Result<bool, ErrResp> {
        let model = UpdateIdentityReq::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find update_identity_req (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.is_some())
    }

    async fn request_update_identity(
        &self,
        submitted_career: SubmittedCareer,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        let account_id = submitted_career.account_id;
        let identity = submitted_career.career_data;
        let career_image1 = submitted_career.career_image1;
        let image1_file_name_without_ext = career_image1.0.clone();
        let (career_image2_option, image2_file_name_without_ext) =
            SubmitCareerOperationImpl::extract_file_name(submitted_career.career_image2);
        let _ = self
            .pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let active_model =
                        SubmitCareerOperationImpl::generate_update_identity_req_active_model(
                            account_id,
                            identity,
                            image1_file_name_without_ext,
                            image2_file_name_without_ext,
                            current_date_time,
                        );
                    let _ = active_model.insert(txn).await.map_err(|e| {
                        error!(
                            "failed to insert update_identity_req (user_account_id: {}): {}",
                            account_id, e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;
                    let _ = SubmitCareerOperationImpl::upload_png_images_to_identity_storage(
                        account_id,
                        career_image1,
                        career_image2_option,
                    )
                    .await?;
                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(db_err) => {
                    error!("failed to insert update_identity_req: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    error!("failed to insert update_identity_req: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

impl SubmitCareerOperationImpl {
    fn extract_file_name(
        file_name_and_binary_option: Option<FileNameAndBinary>,
    ) -> (Option<FileNameAndBinary>, Option<String>) {
        if let Some(file_name_and_binary) = file_name_and_binary_option {
            let identity_image2 = Some((file_name_and_binary.0.clone(), file_name_and_binary.1));
            let image2_file_name_without_ext = Some(file_name_and_binary.0);
            return (identity_image2, image2_file_name_without_ext);
        };
        (None, None)
    }

    fn generate_create_identity_req_active_model(
        account_id: i64,
        identity: Identity,
        image1_file_name_without_ext: String,
        image2_file_name_without_ext: Option<String>,
        current_date_time: DateTime<FixedOffset>,
    ) -> create_identity_req::ActiveModel {
        let date_of_birth = NaiveDate::from_ymd(
            identity.date_of_birth.year,
            identity.date_of_birth.month,
            identity.date_of_birth.day,
        );
        create_identity_req::ActiveModel {
            user_account_id: Set(account_id),
            last_name: Set(identity.last_name),
            first_name: Set(identity.first_name),
            last_name_furigana: Set(identity.last_name_furigana),
            first_name_furigana: Set(identity.first_name_furigana),
            date_of_birth: Set(date_of_birth),
            prefecture: Set(identity.prefecture),
            city: Set(identity.city),
            address_line1: Set(identity.address_line1),
            address_line2: Set(identity.address_line2),
            telephone_number: Set(identity.telephone_number),
            image1_file_name_without_ext: Set(image1_file_name_without_ext),
            image2_file_name_without_ext: Set(image2_file_name_without_ext),
            requested_at: Set(current_date_time),
        }
    }

    fn generate_update_identity_req_active_model(
        account_id: i64,
        identity: Identity,
        image1_file_name_without_ext: String,
        image2_file_name_without_ext: Option<String>,
        current_date_time: DateTime<FixedOffset>,
    ) -> update_identity_req::ActiveModel {
        let date_of_birth = NaiveDate::from_ymd(
            identity.date_of_birth.year,
            identity.date_of_birth.month,
            identity.date_of_birth.day,
        );
        update_identity_req::ActiveModel {
            user_account_id: Set(account_id),
            last_name: Set(identity.last_name),
            first_name: Set(identity.first_name),
            last_name_furigana: Set(identity.last_name_furigana),
            first_name_furigana: Set(identity.first_name_furigana),
            date_of_birth: Set(date_of_birth),
            prefecture: Set(identity.prefecture),
            city: Set(identity.city),
            address_line1: Set(identity.address_line1),
            address_line2: Set(identity.address_line2),
            telephone_number: Set(identity.telephone_number),
            image1_file_name_without_ext: Set(image1_file_name_without_ext),
            image2_file_name_without_ext: Set(image2_file_name_without_ext),
            requested_at: Set(current_date_time),
        }
    }

    async fn upload_png_images_to_identity_storage(
        account_id: i64,
        career_image1: FileNameAndBinary,
        career_image2_option: Option<FileNameAndBinary>,
    ) -> Result<(), ErrRespStruct> {
        let image1_key = format!("{}/{}.png", account_id, career_image1.0);
        let image1_obj = career_image1.1.into_inner();
        let _ = upload_object(IDENTITY_IMAGES_BUCKET_NAME, &image1_key, image1_obj)
            .await
            .map_err(|e| {
                error!(
                    "failed to upload object (image1 key: {}): {}",
                    image1_key, e
                );
                ErrRespStruct {
                    err_resp: unexpected_err_resp(),
                }
            })?;
        if let Some(identity_image2) = career_image2_option {
            let image2_key = format!("{}/{}.png", account_id, identity_image2.0);
            let image2_obj = identity_image2.1.into_inner();
            let _ = upload_object(IDENTITY_IMAGES_BUCKET_NAME, &image2_key, image2_obj)
                .await
                .map_err(|e| {
                    error!(
                        "failed to upload object (image2 key: {}): {}",
                        image2_key, e
                    );
                    ErrRespStruct {
                        err_resp: unexpected_err_resp(),
                    }
                })?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {}
