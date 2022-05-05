// Copyright 2022 Ken Miura

use std::error::Error;
use std::io::Cursor;

use crate::util::validator::career_validator::{validate_career, CareerValidationError};
use crate::util::{clone_file_name_if_exists, convert_jpeg_to_png, FileNameAndBinary};
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
use common::storage::{upload_object, CAREER_IMAGES_BUCKET_NAME};
use common::util::Career;
use common::{
    smtp::{SendMail, SmtpClient, SOCKET_FOR_SMTP_SERVER},
    ApiError, ErrResp, RespResult,
};
use common::{
    ErrRespStruct, JAPANESE_TIME_ZONE, MAX_NUM_OF_CAREER_PER_USER_ACCOUNT, WEB_SITE_NAME,
};
use entity::sea_orm::ActiveValue::NotSet;
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set, TransactionError, TransactionTrait,
};
use entity::{career, create_career_req};
use serde::Serialize;
use tracing::error;
use uuid::Uuid;

use crate::{
    err::{unexpected_err_resp, Code},
    util::{session::User, validator::file_name_validator::validate_extension_is_jpeg},
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
    let (career, career_image1, career_image2_option) =
        handle_multipart(multipart_wrapper, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await?;

    let op = SubmitCareerOperationImpl::new(pool);
    let smtp_client = SmtpClient::new(SOCKET_FOR_SMTP_SERVER.to_string());
    let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
    let image2_file_name_without_ext = Uuid::new_v4().simple().to_string();
    let submitted_career = SubmittedCareer {
        account_id,
        career,
        career_image1: (image1_file_name_without_ext, career_image1),
        career_image2: career_image2_option.map(|image| (image2_file_name_without_ext, image)),
    };
    let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
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
) -> Result<(Career, Cursor<Vec<u8>>, Option<Cursor<Vec<u8>>>), ErrResp> {
    let mut career_option = None;
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
        if name == "career" {
            let career = extract_career(data)?;
            let _ = validate_career(&career).map_err(|e| {
                error!("invalid career: {}", e);
                create_invalid_career_err(&e)
            })?;
            career_option = Some(trim_space_from_career(career));
        } else if name == "career-image1" {
            let _ = validate_career_image_file_name(file_name_option)?;
            let _ = validate_career_image_size(data.len(), max_image_size_in_bytes)?;
            let png_binary = convert_jpeg_to_png(data)?;
            career_image1_option = Some(png_binary);
        } else if name == "career-image2" {
            let _ = validate_career_image_file_name(file_name_option)?;
            let _ = validate_career_image_size(data.len(), max_image_size_in_bytes)?;
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
    let (career, career_image1) =
        ensure_mandatory_params_exist(career_option, career_image1_option)?;
    Ok((career, career_image1, career_image2_option))
}

fn extract_career(data: Bytes) -> Result<Career, ErrResp> {
    let career_json_str = std::str::from_utf8(&data).map_err(|e| {
        error!("invalid utf-8 sequence: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidUtf8Sequence as u32,
            }),
        )
    })?;
    let career = serde_json::from_str::<Career>(career_json_str).map_err(|e| {
        error!("invalid Career JSON object: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidCareerJson as u32,
            }),
        )
    })?;
    Ok(career)
}

fn validate_career_image_file_name(file_name_option: Option<String>) -> Result<(), ErrResp> {
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

fn validate_career_image_size(size: usize, max_size_in_bytes: usize) -> Result<(), ErrResp> {
    if size > max_size_in_bytes {
        error!(
            "invalid career image size (received {} bytes, max size in bytes = {})",
            size, max_size_in_bytes
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ExceedMaxCareerImageSizeLimit as u32,
            }),
        ));
    };
    Ok(())
}

fn ensure_mandatory_params_exist(
    career_option: Option<Career>,
    career_image1_option: Option<Cursor<Vec<u8>>>,
) -> Result<(Career, Cursor<Vec<u8>>), ErrResp> {
    let career = match career_option {
        Some(c) => c,
        None => {
            error!("no career found");
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::NoCareerFound as u32,
                }),
            ));
        }
    };
    let career_image1 = match career_image1_option {
        Some(image1) => image1,
        None => {
            error!("no career-image1 found");
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::NoCareerImage1Found as u32,
                }),
            ));
        }
    };
    Ok((career, career_image1))
}

fn create_invalid_career_err(e: &CareerValidationError) -> ErrResp {
    let code;
    match e {
        CareerValidationError::InvalidCompanyNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = Code::InvalidCompanyNameLength,
        CareerValidationError::IllegalCharInCompanyName(_) => code = Code::IllegalCharInCompanyName,
        CareerValidationError::InvalidDepartmentNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = Code::InvalidDepartmentNameLength,
        CareerValidationError::IllegalCharInDepartmentName(_) => {
            code = Code::IllegalCharInDepartmentName
        }
        CareerValidationError::InvalidOfficeLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = Code::InvalidOfficeLength,
        CareerValidationError::IllegalCharInOffice(_) => code = Code::IllegalCharInOffice,
        CareerValidationError::IllegalCareerStartDate {
            year: _,
            month: _,
            day: _,
        } => code = Code::IllegalCareerStartDate,
        CareerValidationError::IllegalCareerEndDate {
            year: _,
            month: _,
            day: _,
        } => code = Code::IllegalCareerEndDate,
        CareerValidationError::CareerStartDateExceedsCareerEndDate {
            career_start_date: _,
            career_end_date: _,
        } => code = Code::CareerStartDateExceedsCareerEndDate,
        CareerValidationError::IllegalContractType(_) => code = Code::IllegalContractType,
        CareerValidationError::InvalidProfessionLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = Code::InvalidProfessionLength,
        CareerValidationError::IllegalCharInProfession(_) => code = Code::IllegalCharInProfession,
        CareerValidationError::IllegalAnnualIncomInManYen(_) => {
            code = Code::IllegalAnnualIncomInManYen
        }
        CareerValidationError::InvalidPositionNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = Code::InvalidPositionNameLength,
        CareerValidationError::IllegalCharInPositionName(_) => {
            code = Code::IllegalCharInPositionName
        }
        CareerValidationError::InvalidNoteLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = Code::InvalidNoteLength,
        CareerValidationError::IllegalCharInNote(_) => code = Code::IllegalCharInNote,
    }
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError { code: code as u32 }),
    )
}

fn trim_space_from_career(career: Career) -> Career {
    Career {
        company_name: career.company_name.trim().to_string(),
        department_name: career
            .department_name
            .map(|department_name| department_name.trim().to_string()),
        office: career.office.map(|office| office.trim().to_string()),
        career_start_date: career.career_start_date,
        career_end_date: career.career_end_date,
        contract_type: career.contract_type,
        profession: career
            .profession
            .map(|profession| profession.trim().to_string()),
        annual_income_in_man_yen: career.annual_income_in_man_yen,
        is_manager: career.is_manager,
        position_name: career
            .position_name
            .map(|position_name| position_name.trim().to_string()),
        is_new_graduate: career.is_new_graduate,
        note: career.note.map(|note| note.trim().to_string()),
    }
}

async fn handle_career_req(
    submitted_career: SubmittedCareer,
    current_date_time: DateTime<FixedOffset>,
    op: impl SubmitCareerOperation,
    send_mail: impl SendMail,
) -> RespResult<CareerResult> {
    let account_id = submitted_career.account_id;

    let num = op.count_career(account_id).await?;
    if num >= MAX_NUM_OF_CAREER_PER_USER_ACCOUNT as usize {
        error!(
            "already reach max num of career per user account (num: {}, max num: {})",
            num, MAX_NUM_OF_CAREER_PER_USER_ACCOUNT
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ReachCareerNumLimit as u32,
            }),
        ));
    }

    let _ = op
        .request_create_career(submitted_career, current_date_time)
        .await?;

    let subject = create_subject(account_id);
    let text = create_text(account_id);
    let _ =
        async { send_mail.send_mail(ADMIN_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS, &subject, &text) }
            .await?;
    Ok((StatusCode::OK, Json(CareerResult {})))
}

#[derive(Clone, Debug, PartialEq)]
struct SubmittedCareer {
    account_id: i64,
    career: Career,
    career_image1: FileNameAndBinary,
    career_image2: Option<FileNameAndBinary>,
}

fn create_subject(id: i64) -> String {
    format!(
        "[{}] ユーザー (id: {}) からの職務経歴確認依頼",
        WEB_SITE_NAME, id
    )
}

fn create_text(id: i64) -> String {
    format!(
        "ユーザー (id: {}) からの職務経歴確認依頼が届きました。管理者サイトから対応をお願いいたします。",
        id
    )
}

#[async_trait]
trait SubmitCareerOperation {
    async fn count_career(&self, account_id: i64) -> Result<usize, ErrResp>;
    async fn request_create_career(
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
impl SubmitCareerOperation for SubmitCareerOperationImpl {
    async fn count_career(&self, account_id: i64) -> Result<usize, ErrResp> {
        let num = entity::prelude::Career::find()
            .filter(career::Column::UserAccountId.eq(account_id))
            .count(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to count career (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(num)
    }

    async fn request_create_career(
        &self,
        submitted_career: SubmittedCareer,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        let account_id = submitted_career.account_id;
        let career = submitted_career.career;
        let career_image1 = submitted_career.career_image1;
        let image1_file_name_without_ext = career_image1.0.clone();
        let (career_image2_option, image2_file_name_without_ext) =
            clone_file_name_if_exists(submitted_career.career_image2);
        let _ = self
            .pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let active_model =
                        SubmitCareerOperationImpl::generate_create_career_req_active_model(
                            account_id,
                            career,
                            image1_file_name_without_ext,
                            image2_file_name_without_ext,
                            current_date_time,
                        );
                    let _ = active_model.insert(txn).await.map_err(|e| {
                        error!(
                            "failed to insert create_career_req (user_account_id: {}): {}",
                            account_id, e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;
                    let _ = SubmitCareerOperationImpl::upload_png_images_to_career_storage(
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
                    error!("failed to insert create_career_req: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    error!("failed to insert create_career_req: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

impl SubmitCareerOperationImpl {
    fn generate_create_career_req_active_model(
        account_id: i64,
        career: Career,
        image1_file_name_without_ext: String,
        image2_file_name_without_ext: Option<String>,
        current_date_time: DateTime<FixedOffset>,
    ) -> create_career_req::ActiveModel {
        let start_date = NaiveDate::from_ymd(
            career.career_start_date.year,
            career.career_start_date.month,
            career.career_start_date.day,
        );
        let end_date = career
            .career_end_date
            .map(|ymd| NaiveDate::from_ymd(ymd.year, ymd.month, ymd.day));
        create_career_req::ActiveModel {
            create_career_req_id: NotSet,
            user_account_id: Set(account_id),
            company_name: Set(career.company_name),
            department_name: Set(career.department_name),
            office: Set(career.office),
            career_start_date: Set(start_date),
            career_end_date: Set(end_date),
            contract_type: Set(career.contract_type),
            profession: Set(career.profession),
            annual_income_in_man_yen: Set(career.annual_income_in_man_yen),
            is_manager: Set(career.is_manager),
            position_name: Set(career.position_name),
            is_new_graduate: Set(career.is_new_graduate),
            note: Set(career.note),
            image1_file_name_without_ext: Set(image1_file_name_without_ext),
            image2_file_name_without_ext: Set(image2_file_name_without_ext),
            requested_at: Set(current_date_time),
        }
    }

    async fn upload_png_images_to_career_storage(
        account_id: i64,
        career_image1: FileNameAndBinary,
        career_image2_option: Option<FileNameAndBinary>,
    ) -> Result<(), ErrRespStruct> {
        let image1_key = format!("{}/{}.png", account_id, career_image1.0);
        let image1_obj = career_image1.1.into_inner();
        let _ = upload_object(CAREER_IMAGES_BUCKET_NAME, &image1_key, image1_obj)
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
        if let Some(career_image2) = career_image2_option {
            let image2_key = format!("{}/{}.png", account_id, career_image2.0);
            let image2_obj = career_image2.1.into_inner();
            let _ = upload_object(CAREER_IMAGES_BUCKET_NAME, &image2_key, image2_obj)
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
