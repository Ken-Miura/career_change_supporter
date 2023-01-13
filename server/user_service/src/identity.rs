// Copyright 2021 Ken Miura

use std::error::Error;
use std::io::Cursor;

use crate::err::Code::IdentityReqAlreadyExists;
use crate::util::{
    image_converter::convert_jpeg_to_png, multipart::clone_file_name_if_exists,
    multipart::FileNameAndBinary,
};
use async_session::serde_json;
use axum::async_trait;
use axum::extract::State;
use axum::{extract::Multipart, http::StatusCode, Json};
use bytes::Bytes;
use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, Utc};
use common::smtp::{
    ADMIN_EMAIL_ADDRESS, SMTP_HOST, SMTP_PASSWORD, SMTP_PORT, SMTP_USERNAME, SYSTEM_EMAIL_ADDRESS,
};
use common::storage::{upload_object, IDENTITY_IMAGES_BUCKET_NAME};
use common::util::{Identity, Ymd};
use common::{
    smtp::{SendMail, SmtpClient},
    ApiError, ErrResp, RespResult,
};
use common::{ErrRespStruct, JAPANESE_TIME_ZONE, WEB_SITE_NAME};
use entity::prelude::{CreateIdentityReq, UpdateIdentityReq};
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionError, TransactionTrait,
};
use entity::{create_identity_req, update_identity_req};
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
pub(crate) const MAX_IDENTITY_IMAGE_SIZE_IN_BYTES: usize = 4 * 1024 * 1024;

pub(crate) async fn post_identity(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
    multipart: Multipart,
) -> RespResult<IdentityResult> {
    let multipart_wrapper = MultipartWrapperImpl { multipart };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let current_date = current_date_time.naive_local().date();
    let (identity, identity_image1, identity_image2_option) = handle_multipart(
        multipart_wrapper,
        MAX_IDENTITY_IMAGE_SIZE_IN_BYTES,
        current_date,
    )
    .await?;

    let op = SubmitIdentityOperationImpl::new(pool);
    let smtp_client = SmtpClient::new(
        SMTP_HOST.to_string(),
        *SMTP_PORT,
        SMTP_USERNAME.to_string(),
        SMTP_PASSWORD.to_string(),
    );
    let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
    let image2_file_name_without_ext = Uuid::new_v4().simple().to_string();
    let submitted_identity = SubmittedIdentity {
        account_id,
        identity,
        identity_image1: (image1_file_name_without_ext, identity_image1),
        identity_image2: identity_image2_option.map(|image| (image2_file_name_without_ext, image)),
    };
    let result =
        handle_identity_req(submitted_identity, current_date_time, op, smtp_client).await?;
    Ok(result)
}

#[derive(Serialize, Debug, PartialEq)]
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
    max_image_size_in_bytes: usize,
    current_date: NaiveDate,
) -> Result<(Identity, Cursor<Vec<u8>>, Option<Cursor<Vec<u8>>>), ErrResp> {
    let mut identity_option = None;
    let mut identity_image1_option = None;
    let mut identity_image2_option = None;
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
        if name == "identity" {
            let identity = extract_identity(data)?;
            validate_identity(&identity, &current_date).map_err(|e| {
                error!("invalid identity: {}", e);
                create_invalid_identity_err(&e)
            })?;
            identity_option = Some(trim_space_from_identity(identity));
        } else if name == "identity-image1" {
            validate_identity_image_file_name(file_name_option)?;
            validate_identity_image_size(data.len(), max_image_size_in_bytes)?;
            let png_binary = convert_jpeg_to_png(data)?;
            identity_image1_option = Some(png_binary);
        } else if name == "identity-image2" {
            validate_identity_image_file_name(file_name_option)?;
            validate_identity_image_size(data.len(), max_image_size_in_bytes)?;
            let png_binary = convert_jpeg_to_png(data)?;
            identity_image2_option = Some(png_binary);
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
    let (identity, identity_image1) =
        ensure_mandatory_params_exist(identity_option, identity_image1_option)?;
    Ok((identity, identity_image1, identity_image2_option))
}

fn extract_identity(data: Bytes) -> Result<Identity, ErrResp> {
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
    validate_extension_is_jpeg(&file_name).map_err(|e| {
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

fn ensure_mandatory_params_exist(
    identity_option: Option<Identity>,
    identity_image1_option: Option<Cursor<Vec<u8>>>,
) -> Result<(Identity, Cursor<Vec<u8>>), ErrResp> {
    let identity = match identity_option {
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
    let identity_image1 = match identity_image1_option {
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
    Ok((identity, identity_image1))
}

fn create_invalid_identity_err(e: &IdentityValidationError) -> ErrResp {
    let code = match e {
        IdentityValidationError::InvalidLastNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => err::Code::InvalidLastNameLength,
        IdentityValidationError::IllegalCharInLastName(_) => err::Code::IllegalCharInLastName,
        IdentityValidationError::InvalidFirstNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => err::Code::InvalidFirstNameLength,
        IdentityValidationError::IllegalCharInFirstName(_) => err::Code::IllegalCharInFirstName,
        IdentityValidationError::InvalidLastNameFuriganaLength {
            length: _,
            min_length: _,
            max_length: _,
        } => err::Code::InvalidLastNameFuriganaLength,
        IdentityValidationError::IllegalCharInLastNameFurigana(_) => {
            err::Code::IllegalCharInLastNameFurigana
        }
        IdentityValidationError::InvalidFirstNameFuriganaLength {
            length: _,
            min_length: _,
            max_length: _,
        } => err::Code::InvalidFirstNameFuriganaLength,
        IdentityValidationError::IllegalCharInFirstNameFurigana(_) => {
            err::Code::IllegalCharInFirstNameFurigana
        }
        IdentityValidationError::IllegalDate {
            year: _,
            month: _,
            day: _,
        } => err::Code::IllegalDate,
        IdentityValidationError::IllegalAge {
            birth_year: _,
            birth_month: _,
            birth_day: _,
            current_year: _,
            current_month: _,
            current_day: _,
        } => err::Code::IllegalAge,
        IdentityValidationError::InvalidPrefecture(_) => err::Code::InvalidPrefecture,
        IdentityValidationError::InvalidCityLength {
            length: _,
            min_length: _,
            max_length: _,
        } => err::Code::InvalidCityLength,
        IdentityValidationError::IllegalCharInCity(_) => err::Code::IllegalCharInCity,
        IdentityValidationError::InvalidAddressLine1Length {
            length: _,
            min_length: _,
            max_length: _,
        } => err::Code::InvalidAddressLine1Length,
        IdentityValidationError::IllegalCharInAddressLine1(_) => {
            err::Code::IllegalCharInAddressLine1
        }
        IdentityValidationError::InvalidAddressLine2Length {
            length: _,
            min_length: _,
            max_length: _,
        } => err::Code::InvalidAddressLine2Length,
        IdentityValidationError::IllegalCharInAddressLine2(_) => {
            err::Code::IllegalCharInAddressLine2
        }
        IdentityValidationError::InvalidTelNumFormat(_) => err::Code::InvalidTelNumFormat,
    };
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

async fn handle_identity_req(
    submitted_identity: SubmittedIdentity,
    current_date_time: DateTime<FixedOffset>,
    op: impl SubmitIdentityOperation,
    send_mail: impl SendMail,
) -> RespResult<IdentityResult> {
    let account_id = submitted_identity.account_id;
    let identity_option = op
        .find_identity_by_account_id(account_id)
        .await
        .map_err(|e| {
            error!("failed to find identity (account id: {})", account_id);
            e
        })?;
    let identity_exists = identity_option.is_some();
    if let Some(identity) = identity_option {
        info!(
            "request to update identity from account id ({})",
            account_id
        );
        check_update_identity_requirement(&identity, &submitted_identity.identity)?;

        handle_update_identity_request(account_id, submitted_identity, current_date_time, op)
            .await?;
    } else {
        info!(
            "request to create identity from account id ({})",
            account_id
        );

        handle_create_identity_request(account_id, submitted_identity, current_date_time, op)
            .await?;
    };
    let subject = create_subject(account_id, identity_exists);
    let text = create_text(account_id, identity_exists);
    send_mail
        .send_mail(ADMIN_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS, &subject, &text)
        .await?;
    Ok((StatusCode::OK, Json(IdentityResult {})))
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
    submitted_identity: SubmittedIdentity,
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
    op.request_update_identity(submitted_identity, current_date_time)
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
    submitted_identity: SubmittedIdentity,
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
    op.request_create_identity(submitted_identity, current_date_time)
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
struct SubmittedIdentity {
    account_id: i64,
    identity: Identity,
    identity_image1: FileNameAndBinary,
    identity_image2: Option<FileNameAndBinary>,
}

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
        submitted_identity: SubmittedIdentity,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
    async fn check_if_update_identity_req_already_exists(
        &self,
        account_id: i64,
    ) -> Result<bool, ErrResp>;
    async fn request_update_identity(
        &self,
        submitted_identity: SubmittedIdentity,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct SubmitIdentityOperationImpl {
    pool: DatabaseConnection,
}

impl SubmitIdentityOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SubmitIdentityOperation for SubmitIdentityOperationImpl {
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
        submitted_identity: SubmittedIdentity,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        let account_id = submitted_identity.account_id;
        let identity = submitted_identity.identity;
        let identity_image1 = submitted_identity.identity_image1;
        let image1_file_name_without_ext = identity_image1.0.clone();
        let (identity_image2_option, image2_file_name_without_ext) =
            clone_file_name_if_exists(submitted_identity.identity_image2);
        let _ = self
            .pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let active_model =
                        SubmitIdentityOperationImpl::generate_create_identity_req_active_model(
                            account_id,
                            identity,
                            image1_file_name_without_ext,
                            image2_file_name_without_ext,
                            current_date_time,
                        )?;
                    let _ = active_model.insert(txn).await.map_err(|e| {
                        error!(
                            "failed to insert create_identity_req (user_account_id: {}): {}",
                            account_id, e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;
                    let _ = SubmitIdentityOperationImpl::upload_png_images_to_identity_storage(
                        account_id,
                        identity_image1,
                        identity_image2_option,
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
        submitted_identity: SubmittedIdentity,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        let account_id = submitted_identity.account_id;
        let identity = submitted_identity.identity;
        let identity_image1 = submitted_identity.identity_image1;
        let image1_file_name_without_ext = identity_image1.0.clone();
        let (identity_image2_option, image2_file_name_without_ext) =
            clone_file_name_if_exists(submitted_identity.identity_image2);
        let _ = self
            .pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let active_model =
                        SubmitIdentityOperationImpl::generate_update_identity_req_active_model(
                            account_id,
                            identity,
                            image1_file_name_without_ext,
                            image2_file_name_without_ext,
                            current_date_time,
                        )?;
                    let _ = active_model.insert(txn).await.map_err(|e| {
                        error!(
                            "failed to insert update_identity_req (user_account_id: {}): {}",
                            account_id, e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;
                    let _ = SubmitIdentityOperationImpl::upload_png_images_to_identity_storage(
                        account_id,
                        identity_image1,
                        identity_image2_option,
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

impl SubmitIdentityOperationImpl {
    fn generate_create_identity_req_active_model(
        account_id: i64,
        identity: Identity,
        image1_file_name_without_ext: String,
        image2_file_name_without_ext: Option<String>,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<create_identity_req::ActiveModel, ErrRespStruct> {
        let date_of_birth = NaiveDate::from_ymd_opt(
            identity.date_of_birth.year,
            identity.date_of_birth.month,
            identity.date_of_birth.day,
        )
        .ok_or_else(|| {
            error!(
                "failed to get NaiveDate (date_of_birth: {:?})",
                identity.date_of_birth
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
        Ok(create_identity_req::ActiveModel {
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
        })
    }

    fn generate_update_identity_req_active_model(
        account_id: i64,
        identity: Identity,
        image1_file_name_without_ext: String,
        image2_file_name_without_ext: Option<String>,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<update_identity_req::ActiveModel, ErrRespStruct> {
        let date_of_birth = NaiveDate::from_ymd_opt(
            identity.date_of_birth.year,
            identity.date_of_birth.month,
            identity.date_of_birth.day,
        )
        .ok_or_else(|| {
            error!(
                "failed to get NaiveDate (date_of_birth: {:?})",
                identity.date_of_birth
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
        Ok(update_identity_req::ActiveModel {
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
        })
    }

    async fn upload_png_images_to_identity_storage(
        account_id: i64,
        identity_image1: FileNameAndBinary,
        identity_image2_option: Option<FileNameAndBinary>,
    ) -> Result<(), ErrRespStruct> {
        let image1_key = format!("{}/{}.png", account_id, identity_image1.0);
        let image1_obj = identity_image1.1.into_inner();
        upload_object(IDENTITY_IMAGES_BUCKET_NAME, &image1_key, image1_obj)
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
        if let Some(identity_image2) = identity_image2_option {
            let image2_key = format!("{}/{}.png", account_id, identity_image2.0);
            let image2_obj = identity_image2.1.into_inner();
            upload_object(IDENTITY_IMAGES_BUCKET_NAME, &image2_key, image2_obj)
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
mod tests {

    use std::cmp::max;
    use std::{error::Error, fmt::Display, io::Cursor};

    use crate::err::Code::DateOfBirthIsNotMatch;
    use crate::err::Code::ExceedMaxIdentityImageSizeLimit;
    use crate::err::Code::FirstNameIsNotMatch;
    use crate::err::Code::IdentityReqAlreadyExists;
    use crate::err::Code::IllegalAge;
    use crate::err::Code::IllegalCharInAddressLine1;
    use crate::err::Code::IllegalCharInAddressLine2;
    use crate::err::Code::IllegalCharInCity;
    use crate::err::Code::IllegalCharInFirstName;
    use crate::err::Code::IllegalCharInFirstNameFurigana;
    use crate::err::Code::IllegalCharInLastName;
    use crate::err::Code::IllegalCharInLastNameFurigana;
    use crate::err::Code::IllegalDate;
    use crate::err::Code::InvalidAddressLine1Length;
    use crate::err::Code::InvalidAddressLine2Length;
    use crate::err::Code::InvalidCityLength;
    use crate::err::Code::InvalidFirstNameFuriganaLength;
    use crate::err::Code::InvalidFirstNameLength;
    use crate::err::Code::InvalidIdentityJson;
    use crate::err::Code::InvalidJpegImage;
    use crate::err::Code::InvalidLastNameFuriganaLength;
    use crate::err::Code::InvalidLastNameLength;
    use crate::err::Code::InvalidNameInField;
    use crate::err::Code::InvalidPrefecture;
    use crate::err::Code::InvalidTelNumFormat;
    use crate::err::Code::InvalidUtf8Sequence;
    use crate::err::Code::NoFileNameFound;
    use crate::err::Code::NoIdentityFound;
    use crate::err::Code::NoIdentityImage1Found;
    use crate::err::Code::NoIdentityUpdated;
    use crate::err::Code::NoNameFound;
    use crate::err::Code::NotJpegExtension;
    use crate::err::Code::{self, DataParseFailure};
    use crate::identity::{IdentityResult, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES};
    use crate::util::tests::SendMailMock;
    use async_session::serde_json;
    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use bytes::Bytes;
    use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, TimeZone};
    use common::smtp::{ADMIN_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS};
    use common::util::{Identity, Ymd};
    use common::{ApiError, ErrResp, JAPANESE_TIME_ZONE};
    use image::{ImageBuffer, ImageOutputFormat, RgbImage};
    use serde::Deserialize;
    use serde::Serialize;
    use uuid::Uuid;

    use crate::{
        identity::convert_jpeg_to_png, util::validator::identity_validator::MIN_AGE_REQUIREMENT,
    };

    use super::{
        create_subject, create_text, handle_identity_req, handle_multipart, IdentityField,
        MultipartWrapper, SubmitIdentityOperation, SubmittedIdentity,
    };

    // IdentityFieldのdataのResult<Bytes, Box<dyn Error>>がSendを実装しておらず、asyncメソッド内のselfに含められない
    // そのため、テスト用にdataの型を一部修正したダミークラスを用意
    struct DummyIdentityField {
        name: Option<String>,
        file_name: Option<String>,
        data: Bytes,
    }

    struct MultipartWrapperMock {
        count: usize,
        fields: Vec<DummyIdentityField>,
        invalid_multipart_form_data: bool,
    }

    #[async_trait]
    impl MultipartWrapper for MultipartWrapperMock {
        async fn next_field(&mut self) -> Result<Option<IdentityField>, ErrResp> {
            if self.invalid_multipart_form_data {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidMultiPartFormData as u32,
                    }),
                ));
            }
            let dummy_field = self.fields.get(self.count);
            let field = dummy_field.map(|f| IdentityField {
                name: f.name.clone(),
                file_name: f.file_name.clone(),
                data: Ok(f.data.clone()),
            });
            self.count += 1;
            Ok(field)
        }
    }

    struct MultipartWrapperErrMock {}

    #[async_trait]
    impl MultipartWrapper for MultipartWrapperErrMock {
        async fn next_field(&mut self) -> Result<Option<IdentityField>, ErrResp> {
            let field = IdentityField {
                name: Some(String::from("identity-image1")),
                file_name: Some(String::from("test1.jpeg")),
                data: Err(Box::new(DummyError {})),
            };
            Ok(Some(field))
        }
    }

    #[derive(Debug)]
    struct DummyError {}

    impl Error for DummyError {}

    impl Display for DummyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "dummy error")
        }
    }

    #[tokio::test]
    async fn handle_multipart_success() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = create_dummy_identity(&current_date);
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let identity_image2 = create_dummy_identity_image2();
        let identity_image2_field = create_dummy_identity_image_field(
            Some(String::from("identity-image2")),
            Some(String::from("test2.jpeg")),
            identity_image2.clone(),
        );
        let fields = vec![identity_field, identity_image1_field, identity_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let input = result.expect("failed to get Ok");
        assert_eq!(identity, input.0);
        let identity_image1_png = convert_jpeg_to_png(Bytes::from(identity_image1.into_inner()))
            .expect("failed to get Ok");
        assert_eq!(identity_image1_png.into_inner(), input.1.into_inner());
        let identity_image2_png = convert_jpeg_to_png(Bytes::from(identity_image2.into_inner()))
            .expect("failed to get Ok");
        assert_eq!(
            identity_image2_png.into_inner(),
            input.2.expect("failed to get Ok").into_inner()
        );
    }

    fn create_dummy_identity(current_date: &NaiveDate) -> Identity {
        Identity {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        }
    }

    fn create_dummy_identity_field(
        name: Option<String>,
        identity: &Identity,
    ) -> DummyIdentityField {
        let identity_str = serde_json::to_string(identity).expect("failed to get Ok");
        let data = Bytes::from(identity_str);
        DummyIdentityField {
            name,
            file_name: None,
            data,
        }
    }

    fn create_dummy_identity_image1() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(128, 128);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        img.write_to(&mut bytes, ImageOutputFormat::Jpeg(85))
            .expect("failed to get Ok");
        bytes
    }

    fn create_dummy_identity_image2() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(64, 64);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        img.write_to(&mut bytes, ImageOutputFormat::Jpeg(90))
            .expect("failed to get Ok");
        bytes
    }

    fn create_dummy_identity_image_field(
        name: Option<String>,
        file_name: Option<String>,
        jpeg_img: Cursor<Vec<u8>>,
    ) -> DummyIdentityField {
        let data = Bytes::from(jpeg_img.into_inner());
        DummyIdentityField {
            name,
            file_name,
            data,
        }
    }

    #[tokio::test]
    async fn handle_multipart_success_without_identity_image2() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = create_dummy_identity(&current_date);
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let input = result.expect("failed to get Ok");
        assert_eq!(identity, input.0);
        let identity_image1_png = convert_jpeg_to_png(Bytes::from(identity_image1.into_inner()))
            .expect("failed to get Ok");
        assert_eq!(identity_image1_png.into_inner(), input.1.into_inner());
        assert_eq!(None, input.2);
    }

    #[tokio::test]
    async fn handle_multipart_success_image_size_is_equal_to_max_size() {
        let image1_size_in_bytes = Bytes::from(create_dummy_identity_image1().into_inner()).len();
        let image2_size_in_bytes = Bytes::from(create_dummy_identity_image2().into_inner()).len();
        let max_image_size_in_bytes = max(image1_size_in_bytes, image2_size_in_bytes);
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = create_dummy_identity(&current_date);
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let identity_image2 = create_dummy_identity_image2();
        let identity_image2_field = create_dummy_identity_image_field(
            Some(String::from("identity-image2")),
            Some(String::from("test2.jpeg")),
            identity_image2.clone(),
        );
        let fields = vec![identity_field, identity_image1_field, identity_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, max_image_size_in_bytes, current_date).await;

        let input = result.expect("failed to get Ok");
        assert_eq!(identity, input.0);
        let identity_image1_png = convert_jpeg_to_png(Bytes::from(identity_image1.into_inner()))
            .expect("failed to get Ok");
        assert_eq!(identity_image1_png.into_inner(), input.1.into_inner());
        let identity_image2_png = convert_jpeg_to_png(Bytes::from(identity_image2.into_inner()))
            .expect("failed to get Ok");
        assert_eq!(
            identity_image2_png.into_inner(),
            input.2.expect("failed to get Ok").into_inner()
        );
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_multipart_form_data() {
        let image1_size_in_bytes = Bytes::from(create_dummy_identity_image1().into_inner()).len();
        let image2_size_in_bytes = Bytes::from(create_dummy_identity_image2().into_inner()).len();
        let max_image_size_in_bytes = max(image1_size_in_bytes, image2_size_in_bytes);
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = create_dummy_identity(&current_date);
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let identity_image2 = create_dummy_identity_image2();
        let identity_image2_field = create_dummy_identity_image_field(
            Some(String::from("identity-image2")),
            Some(String::from("test2.jpeg")),
            identity_image2.clone(),
        );
        let fields = vec![identity_field, identity_image1_field, identity_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: true,
        };

        let result = handle_multipart(mock, max_image_size_in_bytes, current_date).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::InvalidMultiPartFormData as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_no_name_found() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = create_dummy_identity(&current_date);
        let identity_field =
            create_dummy_identity_field(/* no name specified */ None, &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let identity_image2 = create_dummy_identity_image2();
        let identity_image2_field = create_dummy_identity_image_field(
            Some(String::from("identity-image2")),
            Some(String::from("test2.jpeg")),
            identity_image2.clone(),
        );
        let fields = vec![identity_field, identity_image1_field, identity_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(NoNameFound as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_data_parse() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let mock = MultipartWrapperErrMock {};

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(DataParseFailure as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_name_in_field() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = create_dummy_identity(&current_date);
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            /* invalid name in field */ Some(String::from("1' or '1' = '1';--")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let identity_image2 = create_dummy_identity_image2();
        let identity_image2_field = create_dummy_identity_image_field(
            Some(String::from("identity-image2")),
            Some(String::from("test2.jpeg")),
            identity_image2.clone(),
        );
        let fields = vec![identity_field, identity_image1_field, identity_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(InvalidNameInField as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_no_identity_found() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let identity_image2 = create_dummy_identity_image2();
        let identity_image2_field = create_dummy_identity_image_field(
            Some(String::from("identity-image2")),
            Some(String::from("test2.jpeg")),
            identity_image2.clone(),
        );
        let fields = vec![identity_image1_field, identity_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(NoIdentityFound as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_no_identity_image1_found() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = create_dummy_identity(&current_date);
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image2 = create_dummy_identity_image2();
        let identity_image2_field = create_dummy_identity_image_field(
            Some(String::from("identity-image2")),
            Some(String::from("test2.jpeg")),
            identity_image2.clone(),
        );
        let fields = vec![identity_field, identity_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(NoIdentityImage1Found as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_no_file_name_found() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = create_dummy_identity(&current_date);
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            /* no file name set */ None,
            identity_image1.clone(),
        );
        let identity_image2 = create_dummy_identity_image2();
        let identity_image2_field = create_dummy_identity_image_field(
            Some(String::from("identity-image2")),
            Some(String::from("test2.jpeg")),
            identity_image2.clone(),
        );
        let fields = vec![identity_field, identity_image1_field, identity_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(NoFileNameFound as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_not_jpeg_extension() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = create_dummy_identity(&current_date);
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let identity_image2 = create_dummy_identity_image2();
        let identity_image2_field = create_dummy_identity_image_field(
            Some(String::from("identity-image2")),
            /* not jpeg extension */ Some(String::from("test2.zip")),
            identity_image2.clone(),
        );
        let fields = vec![identity_field, identity_image1_field, identity_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(NotJpegExtension as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_identity_json() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let err_identity = create_dummy_err_identity();
        let identity_field =
            create_err_identity_field(Some(String::from("identity")), &err_identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let identity_image2 = create_dummy_identity_image2();
        let identity_image2_field = create_dummy_identity_image_field(
            Some(String::from("identity-image2")),
            Some(String::from("test2.jpeg")),
            identity_image2.clone(),
        );
        let fields = vec![identity_field, identity_image1_field, identity_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(InvalidIdentityJson as u32, err_resp.1.code);
    }

    fn create_dummy_err_identity() -> ErrIdentity {
        ErrIdentity {
            last_name: String::from("山田"),
            invalid_key: String::from("<script>alert('test')</script>"),
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    struct ErrIdentity {
        last_name: String,
        invalid_key: String,
    }

    fn create_err_identity_field(
        name: Option<String>,
        err_identity: &ErrIdentity,
    ) -> DummyIdentityField {
        let identity_str = serde_json::to_string(err_identity).expect("failed to get Ok");
        let data = Bytes::from(identity_str);
        DummyIdentityField {
            name,
            file_name: None,
            data,
        }
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_utf8_sequence() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity_field = create_invalid_utf8_identity_field(Some(String::from("identity")));
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let identity_image2 = create_dummy_identity_image2();
        let identity_image2_field = create_dummy_identity_image_field(
            Some(String::from("identity-image2")),
            Some(String::from("test2.jpeg")),
            identity_image2.clone(),
        );
        let fields = vec![identity_field, identity_image1_field, identity_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(InvalidUtf8Sequence as u32, err_resp.1.code);
    }

    fn create_invalid_utf8_identity_field(name: Option<String>) -> DummyIdentityField {
        // invalid utf-8 bytes
        // https://stackoverflow.com/questions/1301402/example-invalid-utf8-string
        let data = Bytes::from(vec![0xf0, 0x28, 0x8c, 0xbc]);
        DummyIdentityField {
            name,
            file_name: None,
            data,
        }
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_jpeg_image() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = create_dummy_identity(&current_date);
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1_png();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            /* 実体はpng画像だが、ファイル名で弾かれないようにjpegに設定 */
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let identity_image2 = create_dummy_identity_image2_bmp();
        let identity_image2_field = create_dummy_identity_image_field(
            Some(String::from("identity-image2")),
            /* 実体はbmp画像だが、ファイル名で弾かれないようにjpegに設定 */
            Some(String::from("test2.jpeg")),
            identity_image2.clone(),
        );
        let fields = vec![identity_field, identity_image1_field, identity_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(InvalidJpegImage as u32, err_resp.1.code);
    }

    fn create_dummy_identity_image1_png() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(128, 128);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        img.write_to(&mut bytes, ImageOutputFormat::Png)
            .expect("failed to get Ok");
        bytes
    }

    fn create_dummy_identity_image2_bmp() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(64, 64);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        img.write_to(&mut bytes, ImageOutputFormat::Bmp)
            .expect("failed to get Ok");
        bytes
    }

    #[tokio::test]
    async fn handle_multipart_fail_exceed_max_identity_image_size_limit() {
        let image1_size_in_bytes = Bytes::from(create_dummy_identity_image1().into_inner()).len();
        let image2_size_in_bytes = Bytes::from(create_dummy_identity_image2().into_inner()).len();
        // 最大値は、実際のバイト数 - 1 を指定
        let max_image_size_in_bytes = max(image1_size_in_bytes, image2_size_in_bytes) - 1;
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = create_dummy_identity(&current_date);
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let identity_image2 = create_dummy_identity_image2();
        let identity_image2_field = create_dummy_identity_image_field(
            Some(String::from("identity-image2")),
            Some(String::from("test2.jpeg")),
            identity_image2.clone(),
        );
        let fields = vec![identity_field, identity_image1_field, identity_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, max_image_size_in_bytes, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(ExceedMaxIdentityImageSizeLimit as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_last_name_length() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from(""),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(InvalidLastNameLength as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_illegal_char_in_last_name() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: /* 改行 (LF) */ '\u{000A}'.to_string(),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(IllegalCharInLastName as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_first_name_length() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: String::from(""),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(InvalidFirstNameLength as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_illegal_char_in_first_name() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: /* 改行 (CR) */ '\u{000D}'.to_string(),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(IllegalCharInFirstName as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_last_name_furigana_length() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from(""),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(InvalidLastNameFuriganaLength as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_illegal_char_in_last_name_furigana() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("山田"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(IllegalCharInLastNameFurigana as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_first_name_furigana_length() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from(""),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(InvalidFirstNameFuriganaLength as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_illegal_char_in_first_name_furigana() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("太郎"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(IllegalCharInFirstNameFurigana as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_illegal_date() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: 13,
                day: 32,
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(IllegalDate as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_illegal_age() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - (MIN_AGE_REQUIREMENT - 1),
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(IllegalAge as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_prefecture() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("とうきょうと"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(InvalidPrefecture as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_city_length() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from(""),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(InvalidCityLength as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_illegal_char_in_city() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: /* 水平タブ */ '\u{0009}'.to_string(),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(IllegalCharInCity as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_address_line1_length() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from(""),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(InvalidAddressLine1Length as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_illegal_char_in_address_line1() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1:/* バックスペース */ '\u{0008}'.to_string(),
            address_line2: None,
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(IllegalCharInAddressLine1 as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_address_line2_length() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: Some(String::from("")),
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(InvalidAddressLine2Length as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_illegal_char_in_address_line2() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: Some(/* エスケープ */ '\u{001B}'.to_string()),
            telephone_number: String::from("09012345678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(IllegalCharInAddressLine2 as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_tel_num_format() {
        let current_date = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap()
            .naive_local()
            .date();
        let identity = Identity {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: current_date.year() - MIN_AGE_REQUIREMENT,
                month: current_date.month(),
                day: current_date.day(),
            },
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            // 数字のみ許容 (記号は許容しない)
            telephone_number: String::from("+81-90-1234-5678"),
        };
        let identity_field = create_dummy_identity_field(Some(String::from("identity")), &identity);
        let identity_image1 = create_dummy_identity_image1();
        let identity_image1_field = create_dummy_identity_image_field(
            Some(String::from("identity-image1")),
            Some(String::from("test1.jpeg")),
            identity_image1.clone(),
        );
        let fields = vec![identity_field, identity_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(InvalidTelNumFormat as u32, err_resp.1.code);
    }

    struct SubmitIdentityOperationMock {
        create_identity_req_exists: bool,
        update_identity_req_exists: bool,
        account_id: i64,
        identity_option: Option<Identity>,
        submitted_identity: SubmittedIdentity,
        current_date_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl SubmitIdentityOperation for SubmitIdentityOperationMock {
        async fn find_identity_by_account_id(
            &self,
            account_id: i64,
        ) -> Result<Option<Identity>, ErrResp> {
            assert_eq!(self.account_id, account_id);
            Ok(self.identity_option.clone())
        }

        async fn check_if_create_identity_req_already_exists(
            &self,
            account_id: i64,
        ) -> Result<bool, ErrResp> {
            assert_eq!(self.account_id, account_id);
            Ok(self.create_identity_req_exists)
        }

        async fn request_create_identity(
            &self,
            submitted_identity: SubmittedIdentity,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.submitted_identity, submitted_identity);
            assert_eq!(self.current_date_time, current_date_time);
            Ok(())
        }

        async fn check_if_update_identity_req_already_exists(
            &self,
            account_id: i64,
        ) -> Result<bool, ErrResp> {
            assert_eq!(self.account_id, account_id);
            Ok(self.update_identity_req_exists)
        }

        async fn request_update_identity(
            &self,
            submitted_identity: SubmittedIdentity,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.submitted_identity, submitted_identity);
            assert_eq!(self.current_date_time, current_date_time);
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_identity_req_success_create_identity_req() {
        let account_id = 1234;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap();
        let current_date = current_date_time.naive_local().date();
        let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
        let submitted_identity = SubmittedIdentity {
            account_id,
            identity: create_dummy_identity(&current_date),
            identity_image1: (image1_file_name_without_ext, create_dummy_identity_image1()),
            identity_image2: None,
        };
        let op = SubmitIdentityOperationMock {
            identity_option: None,
            create_identity_req_exists: false,
            update_identity_req_exists: false,
            account_id,
            submitted_identity: submitted_identity.clone(),
            current_date_time,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            create_subject(account_id, op.identity_option.is_some()),
            create_text(account_id, op.identity_option.is_some()),
        );

        let result =
            handle_identity_req(submitted_identity, current_date_time, op, send_mail_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(IdentityResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_identity_req_success_update_identity_req() {
        let account_id = 1234;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap();
        let current_date = current_date_time.naive_local().date();
        let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
        let submitted_identity = SubmittedIdentity {
            account_id,
            identity: create_dummy_identity(&current_date),
            identity_image1: (image1_file_name_without_ext, create_dummy_identity_image1()),
            identity_image2: None,
        };
        let mut identity = submitted_identity.identity.clone();
        identity.telephone_number = String::from("08012345678");
        assert_ne!(
            identity.telephone_number,
            submitted_identity.identity.telephone_number
        );
        let op = SubmitIdentityOperationMock {
            identity_option: Some(identity),
            create_identity_req_exists: false,
            update_identity_req_exists: false,
            account_id,
            submitted_identity: submitted_identity.clone(),
            current_date_time,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            create_subject(account_id, op.identity_option.is_some()),
            create_text(account_id, op.identity_option.is_some()),
        );

        let result =
            handle_identity_req(submitted_identity, current_date_time, op, send_mail_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(IdentityResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_identity_req_fail_create_identity_req_already_exists() {
        let account_id = 1234;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap();
        let current_date = current_date_time.naive_local().date();
        let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
        let submitted_identity = SubmittedIdentity {
            account_id,
            identity: create_dummy_identity(&current_date),
            identity_image1: (image1_file_name_without_ext, create_dummy_identity_image1()),
            identity_image2: None,
        };
        let op = SubmitIdentityOperationMock {
            identity_option: None,
            create_identity_req_exists: true,
            update_identity_req_exists: false,
            account_id,
            submitted_identity: submitted_identity.clone(),
            current_date_time,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            create_subject(account_id, op.identity_option.is_some()),
            create_text(account_id, op.identity_option.is_some()),
        );

        let result =
            handle_identity_req(submitted_identity, current_date_time, op, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(IdentityReqAlreadyExists as u32, resp.1 .0.code);
    }

    #[tokio::test]
    async fn handle_identity_req_fail_update_identity_req_already_exists() {
        let account_id = 1234;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap();
        let current_date = current_date_time.naive_local().date();
        let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
        let submitted_identity = SubmittedIdentity {
            account_id,
            identity: create_dummy_identity(&current_date),
            identity_image1: (image1_file_name_without_ext, create_dummy_identity_image1()),
            identity_image2: None,
        };
        let mut identity = submitted_identity.identity.clone();
        identity.telephone_number = String::from("08012345678");
        assert_ne!(
            identity.telephone_number,
            submitted_identity.identity.telephone_number
        );
        let op = SubmitIdentityOperationMock {
            identity_option: Some(identity),
            create_identity_req_exists: false,
            update_identity_req_exists: true,
            account_id,
            submitted_identity: submitted_identity.clone(),
            current_date_time,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            create_subject(account_id, op.identity_option.is_some()),
            create_text(account_id, op.identity_option.is_some()),
        );

        let result =
            handle_identity_req(submitted_identity, current_date_time, op, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(IdentityReqAlreadyExists as u32, resp.1 .0.code);
    }

    #[tokio::test]
    async fn handle_identity_req_fail_date_of_birth_is_not_match() {
        let account_id = 1234;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap();
        let current_date = current_date_time.naive_local().date();
        let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
        let submitted_identity = SubmittedIdentity {
            account_id,
            identity: create_dummy_identity(&current_date),
            identity_image1: (image1_file_name_without_ext, create_dummy_identity_image1()),
            identity_image2: None,
        };
        let mut identity = submitted_identity.identity.clone();
        identity.date_of_birth = Ymd {
            year: 1978,
            month: 5,
            day: 12,
        };
        assert_ne!(
            identity.date_of_birth,
            submitted_identity.identity.date_of_birth
        );
        identity.telephone_number = String::from("08012345678");
        assert_ne!(
            identity.telephone_number,
            submitted_identity.identity.telephone_number
        );
        let op = SubmitIdentityOperationMock {
            identity_option: Some(identity),
            create_identity_req_exists: false,
            update_identity_req_exists: true,
            account_id,
            submitted_identity: submitted_identity.clone(),
            current_date_time,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            create_subject(account_id, op.identity_option.is_some()),
            create_text(account_id, op.identity_option.is_some()),
        );

        let result =
            handle_identity_req(submitted_identity, current_date_time, op, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(DateOfBirthIsNotMatch as u32, resp.1 .0.code);
    }

    #[tokio::test]
    async fn handle_identity_req_fail_first_name_is_not_match() {
        let account_id = 1234;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap();
        let current_date = current_date_time.naive_local().date();
        let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
        let submitted_identity = SubmittedIdentity {
            account_id,
            identity: create_dummy_identity(&current_date),
            identity_image1: (image1_file_name_without_ext, create_dummy_identity_image1()),
            identity_image2: None,
        };
        let mut identity = submitted_identity.identity.clone();
        identity.first_name = String::from("次郎");
        identity.telephone_number = String::from("08012345678");
        assert_ne!(identity.first_name, submitted_identity.identity.first_name);
        assert_ne!(
            identity.telephone_number,
            submitted_identity.identity.telephone_number
        );
        let op = SubmitIdentityOperationMock {
            identity_option: Some(identity),
            create_identity_req_exists: false,
            update_identity_req_exists: true,
            account_id,
            submitted_identity: submitted_identity.clone(),
            current_date_time,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            create_subject(account_id, op.identity_option.is_some()),
            create_text(account_id, op.identity_option.is_some()),
        );

        let result =
            handle_identity_req(submitted_identity, current_date_time, op, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(FirstNameIsNotMatch as u32, resp.1 .0.code);
    }

    #[tokio::test]
    async fn handle_identity_req_fail_no_identity_updated() {
        let account_id = 1234;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 3, 7, 15, 30, 45)
            .unwrap();
        let current_date = current_date_time.naive_local().date();
        let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
        let submitted_identity = SubmittedIdentity {
            account_id,
            identity: create_dummy_identity(&current_date),
            identity_image1: (image1_file_name_without_ext, create_dummy_identity_image1()),
            identity_image2: None,
        };
        let identity = submitted_identity.identity.clone();
        let op = SubmitIdentityOperationMock {
            identity_option: Some(identity),
            create_identity_req_exists: false,
            update_identity_req_exists: true,
            account_id,
            submitted_identity: submitted_identity.clone(),
            current_date_time,
        };
        let send_mail_mock = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            create_subject(account_id, op.identity_option.is_some()),
            create_text(account_id, op.identity_option.is_some()),
        );

        let result =
            handle_identity_req(submitted_identity, current_date_time, op, send_mail_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(NoIdentityUpdated as u32, resp.1 .0.code);
    }
}
