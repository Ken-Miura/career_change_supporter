// Copyright 2021 Ken Miura

use std::error::Error;
use std::io::Cursor;

use crate::err::Code::IdentityInfoReqAlreadyExists;
use async_session::serde_json;
use axum::async_trait;
use axum::extract::Extension;
use axum::{
    extract::{ContentLengthLimit, Multipart},
    http::StatusCode,
    Json,
};
use bytes::Bytes;
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use common::smtp::{ADMIN_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS};
use common::storage::{upload_object, IDENTITY_IMAGES_BUCKET_NAME};
use common::ErrRespStruct;
use common::{
    smtp::{SendMail, SmtpClient, SOCKET_FOR_SMTP_SERVER},
    ApiError, ErrResp, RespResult,
};
use entity::prelude::{CreateIdentityInfoReq, IdentityInfo, UpdateIdentityInfoReq};
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionError, TransactionTrait,
};
use entity::{create_identity_info_req, update_identity_info_req};
use image::{ImageError, ImageFormat};
use serde::Serialize;
use uuid::Uuid;

use crate::util::WEB_SITE_NAME;
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
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<IdentityResult> {
    let multipart_wrapper = MultipartWrapperImpl { multipart };
    let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
    let current_date = current_date_time.naive_local().date();
    let (identity, identity_image1, identity_image2_option) =
        handle_multipart(multipart_wrapper, current_date).await?;

    let op = SubmitIdentityOperationImpl::new(pool);
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
    let image1_file_name_without_ext = Uuid::new_v4().to_simple().to_string();
    let image2_file_name_without_ext = Uuid::new_v4().to_simple().to_string();
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
) -> Result<(Identity, Cursor<Vec<u8>>, Option<Cursor<Vec<u8>>>), ErrResp> {
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
fn convert_jpeg_to_png(data: Bytes) -> Result<Cursor<Vec<u8>>, ErrResp> {
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
    let mut bytes = Cursor::new(vec![]);
    img.write_to(&mut bytes, image::ImageOutputFormat::Png)
        .map_err(|e| {
            tracing::error!("failed to write image on buffer: {}", e);
            unexpected_err_resp()
        })?;
    Ok(bytes)
}

fn ensure_mandatory_params_exist(
    identity_option: Option<Identity>,
    identity_image1_option: Option<Cursor<Vec<u8>>>,
) -> Result<(Identity, Cursor<Vec<u8>>), ErrResp> {
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

async fn handle_identity_req(
    submitted_identity: SubmittedIdentity,
    current_date_time: DateTime<FixedOffset>,
    op: impl SubmitIdentityOperation,
    send_mail: impl SendMail,
) -> RespResult<IdentityResult> {
    let account_id = submitted_identity.account_id;
    let identity_exists = op
        .check_if_identity_already_exists(account_id)
        .await
        .map_err(|e| {
            tracing::error!(
                "failed to check user's identity existence (id: {})",
                account_id
            );
            e
        })?;
    if identity_exists {
        let update_req_exists = op
            .check_if_update_identity_req_already_exists(account_id)
            .await?;
        if update_req_exists {
            tracing::error!(
                "update identity info req (account id: {}) exists",
                account_id
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: IdentityInfoReqAlreadyExists as u32,
                }),
            ));
        }
        let _ = op
            .request_update_identity(submitted_identity, current_date_time)
            .await
            .map_err(|e| {
                tracing::error!("failed to handle update reqest (id: {})", account_id);
                e
            })?;
    } else {
        let create_req_exists = op
            .check_if_create_identity_req_already_exists(account_id)
            .await?;
        if create_req_exists {
            tracing::error!(
                "create identity info req (account id: {}) exists",
                account_id
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: IdentityInfoReqAlreadyExists as u32,
                }),
            ));
        }
        let _ = op
            .request_create_identity(submitted_identity, current_date_time)
            .await
            .map_err(|e| {
                tracing::error!("failed to handle post request (id: {})", account_id);
                e
            })?;
    };
    let subject = create_subject(account_id, identity_exists);
    let text = create_text(account_id, identity_exists);
    let _ =
        async { send_mail.send_mail(ADMIN_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS, &subject, &text) }
            .await?;
    Ok((StatusCode::OK, Json(IdentityResult {})))
}

struct SubmittedIdentity {
    account_id: i32,
    identity: Identity,
    identity_image1: FileNameAndBinary,
    identity_image2: Option<FileNameAndBinary>,
}

type FileNameAndBinary = (String, Cursor<Vec<u8>>);

fn create_subject(id: i32, update: bool) -> String {
    let request_type = if update { "更新" } else { "新規" };
    format!(
        "[{}] ユーザー (id: {}) からの本人確認依頼 ({})",
        WEB_SITE_NAME, id, request_type
    )
}

fn create_text(id: i32, update: bool) -> String {
    let request_type = if update { "更新" } else { "新規" };
    format!(
        "ユーザー (id: {}) からの本人確認依頼 ({}) が届きました。管理者サイトから対応をお願いいたします。",
        id, request_type
    )
}

#[async_trait]
trait SubmitIdentityOperation {
    async fn check_if_identity_already_exists(&self, account_id: i32) -> Result<bool, ErrResp>;
    async fn check_if_create_identity_req_already_exists(
        &self,
        account_id: i32,
    ) -> Result<bool, ErrResp>;
    async fn request_create_identity(
        &self,
        identity: SubmittedIdentity,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
    async fn check_if_update_identity_req_already_exists(
        &self,
        account_id: i32,
    ) -> Result<bool, ErrResp>;
    async fn request_update_identity(
        &self,
        identity: SubmittedIdentity,
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
    async fn check_if_identity_already_exists(&self, account_id: i32) -> Result<bool, ErrResp> {
        let model = IdentityInfo::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "failed to find identity info (account id: {}): {}",
                    account_id,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(model.is_some())
    }

    async fn check_if_create_identity_req_already_exists(
        &self,
        account_id: i32,
    ) -> Result<bool, ErrResp> {
        let model = CreateIdentityInfoReq::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "failed to find create identity info req (account id: {}): {}",
                    account_id,
                    e
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
            SubmitIdentityOperationImpl::extract_file_name(submitted_identity.identity_image2);
        let _ = self
            .pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let active_model =
                        SubmitIdentityOperationImpl::generate_create_identity_info_req_active_model(
                            account_id,
                            identity,
                            image1_file_name_without_ext,
                            image2_file_name_without_ext,
                            current_date_time,
                        );
                    let _ = active_model.insert(txn).await.map_err(|e| {
                        tracing::error!(
                            "failed to insert create identity info req (account id: {}): {}",
                            account_id,
                            e
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
                    tracing::error!("failed to insert create identity info req: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    tracing::error!(
                        "failed to insert create identity info req: {}",
                        err_resp_struct
                    );
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }

    async fn check_if_update_identity_req_already_exists(
        &self,
        account_id: i32,
    ) -> Result<bool, ErrResp> {
        let model = UpdateIdentityInfoReq::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "failed to find update identity info req (account id: {}): {}",
                    account_id,
                    e
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
            SubmitIdentityOperationImpl::extract_file_name(submitted_identity.identity_image2);
        let _ = self
            .pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let active_model =
                        SubmitIdentityOperationImpl::generate_update_identity_info_req_active_model(
                            account_id,
                            identity,
                            image1_file_name_without_ext,
                            image2_file_name_without_ext,
                            current_date_time,
                        );
                    let _ = active_model.insert(txn).await.map_err(|e| {
                        tracing::error!(
                            "failed to insert update identity info req (account id: {}): {}",
                            account_id,
                            e
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
                    tracing::error!("failed to insert update identity info req: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    tracing::error!(
                        "failed to insert update identity info req: {}",
                        err_resp_struct
                    );
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

impl SubmitIdentityOperationImpl {
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

    fn generate_create_identity_info_req_active_model(
        account_id: i32,
        identity: Identity,
        image1_file_name_without_ext: String,
        image2_file_name_without_ext: Option<String>,
        current_date_time: DateTime<FixedOffset>,
    ) -> create_identity_info_req::ActiveModel {
        let date_of_birth = NaiveDate::from_ymd(
            identity.date_of_birth.year,
            identity.date_of_birth.month,
            identity.date_of_birth.day,
        );
        create_identity_info_req::ActiveModel {
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

    fn generate_update_identity_info_req_active_model(
        account_id: i32,
        identity: Identity,
        image1_file_name_without_ext: String,
        image2_file_name_without_ext: Option<String>,
        current_date_time: DateTime<FixedOffset>,
    ) -> update_identity_info_req::ActiveModel {
        let date_of_birth = NaiveDate::from_ymd(
            identity.date_of_birth.year,
            identity.date_of_birth.month,
            identity.date_of_birth.day,
        );
        update_identity_info_req::ActiveModel {
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
        account_id: i32,
        identity_image1: FileNameAndBinary,
        identity_image2_option: Option<FileNameAndBinary>,
    ) -> Result<(), ErrRespStruct> {
        let image1_key = format!("{}/{}.png", account_id, identity_image1.0);
        let image1_obj = identity_image1.1.into_inner();
        let _ = upload_object(IDENTITY_IMAGES_BUCKET_NAME, &image1_key, image1_obj)
            .await
            .map_err(|e| {
                tracing::error!(
                    "failed to upload object (image1 key: {}): {}",
                    image1_key,
                    e
                );
                ErrRespStruct {
                    err_resp: unexpected_err_resp(),
                }
            })?;
        if let Some(identity_image2) = identity_image2_option {
            let image2_key = format!("{}/{}.png", account_id, identity_image2.0);
            let image2_obj = identity_image2.1.into_inner();
            let _ = upload_object(IDENTITY_IMAGES_BUCKET_NAME, &image2_key, image2_obj)
                .await
                .map_err(|e| {
                    tracing::error!(
                        "failed to upload object (image2 key: {}): {}",
                        image2_key,
                        e
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

    use std::{error::Error, fmt::Display, io::Cursor};

    use crate::identity::Code::DataParseFailure;
    use crate::identity::Code::InvalidIdentityJson;
    use crate::identity::Code::InvalidJpegImage;
    use crate::identity::Code::InvalidNameInField;
    use crate::identity::Code::InvalidUtf8Sequence;
    use crate::identity::Code::NoFileNameFound;
    use crate::identity::Code::NoIdentityFound;
    use crate::identity::Code::NoIdentityImage1Found;
    use crate::identity::Code::NoNameFound;
    use crate::identity::Code::NotJpegExtension;
    use async_session::serde_json;
    use axum::async_trait;
    use axum::http::StatusCode;
    use bytes::Bytes;
    use chrono::{Datelike, NaiveDate, TimeZone, Utc};
    use common::ErrResp;
    use image::{ImageBuffer, ImageOutputFormat, RgbImage};
    use serde::Deserialize;
    use serde::Serialize;

    use crate::{
        identity::convert_jpeg_to_png,
        util::{
            validator::identity_validator::MIN_AGE_REQUIREMENT, Identity, Ymd, JAPANESE_TIME_ZONE,
        },
    };

    use super::{handle_multipart, IdentityField, MultipartWrapper};

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
    }

    #[async_trait]
    impl MultipartWrapper for MultipartWrapperMock {
        async fn next_field(&mut self) -> Result<Option<IdentityField>, ErrResp> {
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
        let current_date = Utc
            .ymd(2022, 3, 7)
            .and_hms(15, 30, 45)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned())
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
        let mock = MultipartWrapperMock { count: 0, fields };

        let result = handle_multipart(mock, current_date).await;

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
        let _ = img
            .write_to(&mut bytes, ImageOutputFormat::Jpeg(85))
            .expect("failed to get Ok");
        bytes
    }

    fn create_dummy_identity_image2() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(64, 64);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        let _ = img
            .write_to(&mut bytes, ImageOutputFormat::Jpeg(90))
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
        let current_date = Utc
            .ymd(2022, 3, 7)
            .and_hms(15, 30, 45)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned())
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
        let mock = MultipartWrapperMock { count: 0, fields };

        let result = handle_multipart(mock, current_date).await;

        let input = result.expect("failed to get Ok");
        assert_eq!(identity, input.0);
        let identity_image1_png = convert_jpeg_to_png(Bytes::from(identity_image1.into_inner()))
            .expect("failed to get Ok");
        assert_eq!(identity_image1_png.into_inner(), input.1.into_inner());
        assert_eq!(None, input.2);
    }

    #[tokio::test]
    async fn handle_multipart_fail_no_name_found() {
        let current_date = Utc
            .ymd(2022, 3, 7)
            .and_hms(15, 30, 45)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned())
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
        let mock = MultipartWrapperMock { count: 0, fields };

        let result = handle_multipart(mock, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(NoNameFound as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_data_parse() {
        let current_date = Utc
            .ymd(2022, 3, 7)
            .and_hms(15, 30, 45)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned())
            .naive_local()
            .date();
        let mock = MultipartWrapperErrMock {};

        let result = handle_multipart(mock, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(DataParseFailure as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_name_in_field() {
        let current_date = Utc
            .ymd(2022, 3, 7)
            .and_hms(15, 30, 45)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned())
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
        let mock = MultipartWrapperMock { count: 0, fields };

        let result = handle_multipart(mock, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(InvalidNameInField as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_no_identity_found() {
        let current_date = Utc
            .ymd(2022, 3, 7)
            .and_hms(15, 30, 45)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned())
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
        let mock = MultipartWrapperMock { count: 0, fields };

        let result = handle_multipart(mock, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(NoIdentityFound as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_no_identity_image1_found() {
        let current_date = Utc
            .ymd(2022, 3, 7)
            .and_hms(15, 30, 45)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned())
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
        let mock = MultipartWrapperMock { count: 0, fields };

        let result = handle_multipart(mock, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(NoIdentityImage1Found as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_no_file_name_found() {
        let current_date = Utc
            .ymd(2022, 3, 7)
            .and_hms(15, 30, 45)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned())
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
        let mock = MultipartWrapperMock { count: 0, fields };

        let result = handle_multipart(mock, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(NoFileNameFound as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_not_jpeg_extension() {
        let current_date = Utc
            .ymd(2022, 3, 7)
            .and_hms(15, 30, 45)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned())
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
        let mock = MultipartWrapperMock { count: 0, fields };

        let result = handle_multipart(mock, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(NotJpegExtension as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_identity_json() {
        let current_date = Utc
            .ymd(2022, 3, 7)
            .and_hms(15, 30, 45)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned())
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
        let mock = MultipartWrapperMock { count: 0, fields };

        let result = handle_multipart(mock, current_date).await;

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
        let current_date = Utc
            .ymd(2022, 3, 7)
            .and_hms(15, 30, 45)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned())
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
        let mock = MultipartWrapperMock { count: 0, fields };

        let result = handle_multipart(mock, current_date).await;

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
        let current_date = Utc
            .ymd(2022, 3, 7)
            .and_hms(15, 30, 45)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned())
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
            /* 実体はpng画像だが、ファイル名で弾かれないようにjpegに設定 */
            Some(String::from("test2.jpeg")),
            identity_image2.clone(),
        );
        let fields = vec![identity_field, identity_image1_field, identity_image2_field];
        let mock = MultipartWrapperMock { count: 0, fields };

        let result = handle_multipart(mock, current_date).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(InvalidJpegImage as u32, err_resp.1.code);
    }

    fn create_dummy_identity_image1_png() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(128, 128);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        let _ = img
            .write_to(&mut bytes, ImageOutputFormat::Png)
            .expect("failed to get Ok");
        bytes
    }

    fn create_dummy_identity_image2_bmp() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(64, 64);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        let _ = img
            .write_to(&mut bytes, ImageOutputFormat::Bmp)
            .expect("failed to get Ok");
        bytes
    }
}
