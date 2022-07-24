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
use common::smtp::{
    ADMIN_EMAIL_ADDRESS, SMTP_HOST, SMTP_PASSWORD, SMTP_PORT, SMTP_USERNAME, SYSTEM_EMAIL_ADDRESS,
};
use common::storage::{upload_object, CAREER_IMAGES_BUCKET_NAME};
use common::util::Career;
use common::{
    smtp::{SendMail, SmtpClient},
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

pub(crate) async fn career(
    User { account_id }: User,
    ContentLengthLimit(multipart): ContentLengthLimit<
        Multipart,
        {
            9 * 1024 * 1024 /* 9mb */ /* サイズをオーバーした場合、ContentLengthLimitはステータスコード413を返却する */
        },
    >,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<CareerResult> {
    let multipart_wrapper = MultipartWrapperImpl { multipart };
    let (career, career_image1, career_image2_option) =
        handle_multipart(multipart_wrapper, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await?;

    let op = SubmitCareerOperationImpl::new(pool);
    let smtp_client = SmtpClient::new(
        SMTP_HOST.to_string(),
        *SMTP_PORT,
        SMTP_USERNAME.to_string(),
        SMTP_PASSWORD.to_string(),
    );
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
    let code = match e {
        CareerValidationError::InvalidCompanyNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => Code::InvalidCompanyNameLength,
        CareerValidationError::IllegalCharInCompanyName(_) => Code::IllegalCharInCompanyName,
        CareerValidationError::InvalidDepartmentNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => Code::InvalidDepartmentNameLength,
        CareerValidationError::IllegalCharInDepartmentName(_) => Code::IllegalCharInDepartmentName,
        CareerValidationError::InvalidOfficeLength {
            length: _,
            min_length: _,
            max_length: _,
        } => Code::InvalidOfficeLength,
        CareerValidationError::IllegalCharInOffice(_) => Code::IllegalCharInOffice,
        CareerValidationError::IllegalCareerStartDate {
            year: _,
            month: _,
            day: _,
        } => Code::IllegalCareerStartDate,
        CareerValidationError::IllegalCareerEndDate {
            year: _,
            month: _,
            day: _,
        } => Code::IllegalCareerEndDate,
        CareerValidationError::CareerStartDateExceedsCareerEndDate {
            career_start_date: _,
            career_end_date: _,
        } => Code::CareerStartDateExceedsCareerEndDate,
        CareerValidationError::IllegalContractType(_) => Code::IllegalContractType,
        CareerValidationError::InvalidProfessionLength {
            length: _,
            min_length: _,
            max_length: _,
        } => Code::InvalidProfessionLength,
        CareerValidationError::IllegalCharInProfession(_) => Code::IllegalCharInProfession,
        CareerValidationError::IllegalAnnualIncomeInManYen(_) => Code::IllegalAnnualIncomeInManYen,
        CareerValidationError::InvalidPositionNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => Code::InvalidPositionNameLength,
        CareerValidationError::IllegalCharInPositionName(_) => Code::IllegalCharInPositionName,
        CareerValidationError::InvalidNoteLength {
            length: _,
            min_length: _,
            max_length: _,
        } => Code::InvalidNoteLength,
        CareerValidationError::IllegalCharInNote(_) => Code::IllegalCharInNote,
    };
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

    let identity_exists = op.check_if_identity_exists(account_id).await?;
    if !identity_exists {
        error!("identity is not registered (account id: {})", account_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoIdentityRegistered as u32,
            }),
        ));
    }

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
    let num = op.count_create_career_req(account_id).await?;
    // create_career_reqの最大もMAX_NUM_OF_CAREER_PER_USER_ACCOUNTとする
    if num >= MAX_NUM_OF_CAREER_PER_USER_ACCOUNT as usize {
        error!(
            "already reach max num of create career request per user account (num: {}, max num: {})",
            num, MAX_NUM_OF_CAREER_PER_USER_ACCOUNT
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ReachCreateCareerReqNumLimit as u32,
            }),
        ));
    }

    let _ = op
        .request_create_career(submitted_career, current_date_time)
        .await?;

    let subject = create_subject(account_id);
    let text = create_text(account_id);
    let _ = send_mail
        .send_mail(ADMIN_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS, &subject, &text)
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
    /// Identityが存在するか確認する。存在する場合、trueを返す。そうでない場合、falseを返す。
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
    async fn count_career(&self, account_id: i64) -> Result<usize, ErrResp>;
    async fn count_create_career_req(&self, account_id: i64) -> Result<usize, ErrResp>;
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
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
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
        Ok(model.is_some())
    }

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

    async fn count_create_career_req(&self, account_id: i64) -> Result<usize, ErrResp> {
        let num = entity::prelude::CreateCareerReq::find()
            .filter(create_career_req::Column::UserAccountId.eq(account_id))
            .count(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to count create_career_req (user_account_id: {}): {}",
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
mod tests {
    use std::cmp::max;
    use std::error::Error;
    use std::fmt::Display;
    use std::io::Cursor;

    use async_session::serde_json;
    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use bytes::Bytes;
    use chrono::{DateTime, FixedOffset, Utc};
    use common::smtp::{ADMIN_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS};
    use common::{
        util::{Career, Ymd},
        ApiError, ErrResp,
    };
    use common::{JAPANESE_TIME_ZONE, MAX_NUM_OF_CAREER_PER_USER_ACCOUNT};
    use image::{ImageBuffer, ImageOutputFormat, RgbImage};
    use serde::{Deserialize, Serialize};
    use uuid::Uuid;

    use super::{handle_multipart, CareerResult, MAX_CAREER_IMAGE_SIZE_IN_BYTES};
    use crate::util::tests::SendMailMock;
    use crate::{err::Code, util::convert_jpeg_to_png};

    use super::{
        create_subject, create_text, handle_career_req, CareerField, MultipartWrapper,
        SubmitCareerOperation, SubmittedCareer,
    };

    // CareerFieldのdataのResult<Bytes, Box<dyn Error>>がSendを実装しておらず、asyncメソッド内のselfに含められない
    // そのため、テスト用にdataの型を一部修正したダミークラスを用意
    struct DummyCareerField {
        name: Option<String>,
        file_name: Option<String>,
        data: Bytes,
    }

    struct MultipartWrapperMock {
        count: usize,
        fields: Vec<DummyCareerField>,
        invalid_multipart_form_data: bool,
    }

    #[async_trait]
    impl MultipartWrapper for MultipartWrapperMock {
        async fn next_field(&mut self) -> Result<Option<CareerField>, ErrResp> {
            if self.invalid_multipart_form_data {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidMultiPartFormData as u32,
                    }),
                ));
            }
            let dummy_field = self.fields.get(self.count);
            let field = dummy_field.map(|f| CareerField {
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
        async fn next_field(&mut self) -> Result<Option<CareerField>, ErrResp> {
            let field = CareerField {
                name: Some(String::from("career-image1")),
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
        let career = create_dummy_career();
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let input = result.expect("failed to get Ok");
        assert_eq!(career, input.0);
        let career_image1_png =
            convert_jpeg_to_png(Bytes::from(career_image1.into_inner())).expect("failed to get Ok");
        assert_eq!(career_image1_png.into_inner(), input.1.into_inner());
        let career_image2_png =
            convert_jpeg_to_png(Bytes::from(career_image2.into_inner())).expect("failed to get Ok");
        assert_eq!(
            career_image2_png.into_inner(),
            input.2.expect("failed to get Ok").into_inner()
        );
    }

    fn create_dummy_career() -> Career {
        Career {
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
        }
    }

    fn create_dummy_career_field(name: Option<String>, career: &Career) -> DummyCareerField {
        let career_str = serde_json::to_string(career).expect("failed to get Ok");
        let data = Bytes::from(career_str);
        DummyCareerField {
            name,
            file_name: None,
            data,
        }
    }

    fn create_dummy_career_image1() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(128, 128);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        let _ = img
            .write_to(&mut bytes, ImageOutputFormat::Jpeg(85))
            .expect("failed to get Ok");
        bytes
    }

    fn create_dummy_career_image2() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(64, 64);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        let _ = img
            .write_to(&mut bytes, ImageOutputFormat::Jpeg(90))
            .expect("failed to get Ok");
        bytes
    }

    fn create_dummy_career_image_field(
        name: Option<String>,
        file_name: Option<String>,
        jpeg_img: Cursor<Vec<u8>>,
    ) -> DummyCareerField {
        let data = Bytes::from(jpeg_img.into_inner());
        DummyCareerField {
            name,
            file_name,
            data,
        }
    }

    #[tokio::test]
    async fn handle_multipart_success_without_career_image2() {
        let career = create_dummy_career();
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let fields = vec![career_field, career_image1_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let input = result.expect("failed to get Ok");
        assert_eq!(career, input.0);
        let career_image1_png =
            convert_jpeg_to_png(Bytes::from(career_image1.into_inner())).expect("failed to get Ok");
        assert_eq!(career_image1_png.into_inner(), input.1.into_inner());
        assert_eq!(None, input.2);
    }

    #[tokio::test]
    async fn handle_multipart_success_image_size_is_equal_to_max_size() {
        let image1_size_in_bytes = Bytes::from(create_dummy_career_image1().into_inner()).len();
        let image2_size_in_bytes = Bytes::from(create_dummy_career_image2().into_inner()).len();
        let max_image_size_in_bytes = max(image1_size_in_bytes, image2_size_in_bytes);

        let career = create_dummy_career();
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, max_image_size_in_bytes).await;

        let input = result.expect("failed to get Ok");
        assert_eq!(career, input.0);
        let career_image1_png =
            convert_jpeg_to_png(Bytes::from(career_image1.into_inner())).expect("failed to get Ok");
        assert_eq!(career_image1_png.into_inner(), input.1.into_inner());
        let career_image2_png =
            convert_jpeg_to_png(Bytes::from(career_image2.into_inner())).expect("failed to get Ok");
        assert_eq!(
            career_image2_png.into_inner(),
            input.2.expect("failed to get Ok").into_inner()
        );
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_multipart_form_data() {
        let career = create_dummy_career();
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: true,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::InvalidMultiPartFormData as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_no_name_found() {
        let career = create_dummy_career();
        let career_field = create_dummy_career_field(/* no name specified */ None, &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::NoNameFound as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_data_parse() {
        let mock = MultipartWrapperErrMock {};

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(Code::DataParseFailure as u32, err_resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_name_in_field() {
        let career = create_dummy_career();
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            /* invalid name in field */ Some(String::from("1' or '1' = '1';--")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::InvalidNameInField as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_no_career_found() {
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::NoCareerFound as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_no_career_image1_found() {
        let career = create_dummy_career();
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::NoCareerImage1Found as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_no_file_name_found() {
        let career = create_dummy_career();
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            /* no file name set */ None,
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::NoFileNameFound as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_not_jpeg_extension() {
        let career = create_dummy_career();
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            /* not jpeg extension */ Some(String::from("test2.zip")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::NotJpegExtension as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_career_json() {
        let career = create_dummy_err_career();
        let career_field = create_err_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::InvalidCareerJson as u32, resp.1.code);
    }

    fn create_dummy_err_career() -> ErrCareer {
        ErrCareer {
            company_name: String::from("テスト株式会社"),
            invalid_key: String::from("<script>alert('test')</script>"),
        }
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    struct ErrCareer {
        company_name: String,
        invalid_key: String,
    }

    fn create_err_career_field(name: Option<String>, err_career: &ErrCareer) -> DummyCareerField {
        let career_str = serde_json::to_string(err_career).expect("failed to get Ok");
        let data = Bytes::from(career_str);
        DummyCareerField {
            name,
            file_name: None,
            data,
        }
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_utf8_sequence() {
        let career_field = create_invalid_utf8_career_field(Some(String::from("career")));
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::InvalidUtf8Sequence as u32, resp.1.code);
    }

    fn create_invalid_utf8_career_field(name: Option<String>) -> DummyCareerField {
        // invalid utf-8 bytes
        // https://stackoverflow.com/questions/1301402/example-invalid-utf8-string
        let data = Bytes::from(vec![0xf0, 0x28, 0x8c, 0xbc]);
        DummyCareerField {
            name,
            file_name: None,
            data,
        }
    }

    #[tokio::test]
    async fn handle_multipart_fail_invalid_jpeg_image() {
        let career = create_dummy_career();
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1_png();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            /* 実体はpng画像だが、ファイル名で弾かれないようにjpegに設定 */
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2_bmp();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            /* 実体はbmp画像だが、ファイル名で弾かれないようにjpegに設定 */
            Some(String::from("test2.jpg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::InvalidJpegImage as u32, resp.1.code);
    }

    fn create_dummy_career_image1_png() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(128, 128);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        let _ = img
            .write_to(&mut bytes, ImageOutputFormat::Png)
            .expect("failed to get Ok");
        bytes
    }

    fn create_dummy_career_image2_bmp() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(64, 64);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        let _ = img
            .write_to(&mut bytes, ImageOutputFormat::Bmp)
            .expect("failed to get Ok");
        bytes
    }

    #[tokio::test]
    async fn handle_multipart_fail_exceed_max_career_image_size_limit() {
        let image1_size_in_bytes = Bytes::from(create_dummy_career_image1().into_inner()).len();
        let image2_size_in_bytes = Bytes::from(create_dummy_career_image2().into_inner()).len();
        // 最大値は、実際のバイト数 - 1 を指定
        let max_image_size_in_bytes = max(image1_size_in_bytes, image2_size_in_bytes) - 1;

        let career = create_dummy_career();
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, max_image_size_in_bytes).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::ExceedMaxCareerImageSizeLimit as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_invalid_company_name_length() {
        let career = Career {
            company_name: "".to_string(),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::InvalidCompanyNameLength as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_illegal_char_in_company_name() {
        let career = Career {
            company_name: /* 改行 (LF) */ '\u{000A}'.to_string(),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::IllegalCharInCompanyName as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_invalid_department_name_length() {
        let career = Career {
            company_name: "テスト株式会社".to_string(),
            department_name: Some("".to_string()),
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::InvalidDepartmentNameLength as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_illegal_char_in_department_name() {
        let career = Career {
            company_name: "テスト株式会社".to_string(),
            department_name: Some(/* 改行 (CR) */ '\u{000D}'.to_string()),
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::IllegalCharInDepartmentName as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_invalid_office_length() {
        let career = Career {
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: Some("".to_string()),
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::InvalidOfficeLength as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_illegal_char_in_office() {
        let career = Career {
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: Some(/* 水平タブ */ '\u{0009}'.to_string()),
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::IllegalCharInOffice as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_illegal_career_start_date() {
        let career = Career {
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 9,
                day: 31,
            },
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::IllegalCareerStartDate as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_illegal_career_end_date() {
        let career = Career {
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: Some(Ymd {
                year: 2010,
                month: 10,
                day: 32,
            }),
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::IllegalCareerEndDate as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_career_start_date_exceeds_career_end_date() {
        let career = Career {
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: Some(Ymd {
                year: 2005,
                month: 10,
                day: 31,
            }),
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(
            Code::CareerStartDateExceedsCareerEndDate as u32,
            resp.1.code
        );
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_illegal_contract_type() {
        let career = Career {
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "jfap;fje;fljwae;fj".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::IllegalContractType as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_invalid_profession_length() {
        let career = Career {
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: Some("".to_string()),
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::InvalidProfessionLength as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_illegal_char_in_profession() {
        let career = Career {
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: Some("<script>alert('test')</script>".to_string()),
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::IllegalCharInProfession as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_illegal_annual_income_in_man_yen() {
        let career = Career {
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: Some(-200),
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::IllegalAnnualIncomeInManYen as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_invalid_position_name_length() {
        let career = Career {
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: Some("".to_string()),
            is_new_graduate: true,
            note: None,
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::InvalidPositionNameLength as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_illegal_char_in_position_name() {
        let career = Career {
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: Some("1' or '1' = '1';--".to_string()),
            is_new_graduate: true,
            note: None,
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::IllegalCharInPositionName as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_invalid_position_note_length() {
        let career = Career {
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: Some("".to_string()),
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::InvalidNoteLength as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_multipart_fail_fail_illegal_char_in_note() {
        let career = Career {
            company_name: "テスト株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2008,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: true,
            position_name: None,
            is_new_graduate: false,
            note: Some("1' or '1' = '1';--".to_string()),
        };
        let career_field = create_dummy_career_field(Some(String::from("career")), &career);
        let career_image1 = create_dummy_career_image1();
        let career_image1_field = create_dummy_career_image_field(
            Some(String::from("career-image1")),
            Some(String::from("test1.jpeg")),
            career_image1.clone(),
        );
        let career_image2 = create_dummy_career_image2();
        let career_image2_field = create_dummy_career_image_field(
            Some(String::from("career-image2")),
            Some(String::from("test2.jpeg")),
            career_image2.clone(),
        );
        let fields = vec![career_field, career_image1_field, career_image2_field];
        let mock = MultipartWrapperMock {
            count: 0,
            fields,
            invalid_multipart_form_data: false,
        };

        let result = handle_multipart(mock, MAX_CAREER_IMAGE_SIZE_IN_BYTES).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::IllegalCharInNote as u32, resp.1.code);
    }

    struct SubmitCareerOperationMock {
        account_id: i64,
        submitted_career: SubmittedCareer,
        current_date_time: DateTime<FixedOffset>,
        num_of_career: usize,
        num_of_create_career_req: usize,
        identity_exists: bool,
    }

    #[async_trait]
    impl SubmitCareerOperation for SubmitCareerOperationMock {
        async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
            assert_eq!(self.account_id, account_id);
            Ok(self.identity_exists)
        }

        async fn count_career(&self, account_id: i64) -> Result<usize, ErrResp> {
            assert_eq!(self.account_id, account_id);
            Ok(self.num_of_career)
        }

        async fn count_create_career_req(&self, account_id: i64) -> Result<usize, ErrResp> {
            assert_eq!(self.account_id, account_id);
            Ok(self.num_of_create_career_req)
        }

        async fn request_create_career(
            &self,
            submitted_career: SubmittedCareer,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.submitted_career, submitted_career);
            assert_eq!(self.current_date_time, current_date_time);
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_career_req_success() {
        let account_id = 78515;
        let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
        let submitted_career = SubmittedCareer {
            account_id,
            career: create_dummy_career(),
            career_image1: (image1_file_name_without_ext, create_dummy_career_image1()),
            career_image2: None,
        };
        let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
        let op = SubmitCareerOperationMock {
            account_id,
            submitted_career: submitted_career.clone(),
            current_date_time,
            num_of_career: (MAX_NUM_OF_CAREER_PER_USER_ACCOUNT - 1) as usize,
            num_of_create_career_req: (MAX_NUM_OF_CAREER_PER_USER_ACCOUNT - 1) as usize,
            identity_exists: true,
        };
        let smtp_client = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            create_subject(account_id),
            create_text(account_id),
        );

        let result = handle_career_req(submitted_career, current_date_time, op, smtp_client).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(CareerResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_career_req_success_with_image2() {
        let account_id = 78515;
        let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
        let image2_file_name_without_ext = Uuid::new_v4().simple().to_string();
        let submitted_career = SubmittedCareer {
            account_id,
            career: create_dummy_career(),
            career_image1: (image1_file_name_without_ext, create_dummy_career_image1()),
            career_image2: Some((image2_file_name_without_ext, create_dummy_career_image2())),
        };
        let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
        let op = SubmitCareerOperationMock {
            account_id,
            submitted_career: submitted_career.clone(),
            current_date_time,
            num_of_career: 0,
            num_of_create_career_req: 0,
            identity_exists: true,
        };
        let smtp_client = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            create_subject(account_id),
            create_text(account_id),
        );

        let result = handle_career_req(submitted_career, current_date_time, op, smtp_client).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(CareerResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn handle_career_req_fail_reach_career_num_limit() {
        let account_id = 78515;
        let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
        let submitted_career = SubmittedCareer {
            account_id,
            career: create_dummy_career(),
            career_image1: (image1_file_name_without_ext, create_dummy_career_image1()),
            career_image2: None,
        };
        let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
        let op = SubmitCareerOperationMock {
            account_id,
            submitted_career: submitted_career.clone(),
            current_date_time,
            num_of_career: MAX_NUM_OF_CAREER_PER_USER_ACCOUNT as usize,
            num_of_create_career_req: (MAX_NUM_OF_CAREER_PER_USER_ACCOUNT - 1) as usize,
            identity_exists: true,
        };
        let smtp_client = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            create_subject(account_id),
            create_text(account_id),
        );

        let result = handle_career_req(submitted_career, current_date_time, op, smtp_client).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::ReachCareerNumLimit as u32, resp.1.code);
    }

    // 基本的にMAX_NUM_OF_CAREER_PER_USER_ACCOUNTに達した場合、
    // Careerを作れないのでありえないケースだが、テストは用意しておく
    #[tokio::test]
    async fn handle_career_req_fail_over_career_num_limit() {
        let account_id = 78515;
        let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
        let submitted_career = SubmittedCareer {
            account_id,
            career: create_dummy_career(),
            career_image1: (image1_file_name_without_ext, create_dummy_career_image1()),
            career_image2: None,
        };
        let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
        let op = SubmitCareerOperationMock {
            account_id,
            submitted_career: submitted_career.clone(),
            current_date_time,
            num_of_career: (MAX_NUM_OF_CAREER_PER_USER_ACCOUNT + 1) as usize,
            num_of_create_career_req: (MAX_NUM_OF_CAREER_PER_USER_ACCOUNT - 1) as usize,
            identity_exists: true,
        };
        let smtp_client = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            create_subject(account_id),
            create_text(account_id),
        );

        let result = handle_career_req(submitted_career, current_date_time, op, smtp_client).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::ReachCareerNumLimit as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_career_req_fail_no_identity_registered() {
        let account_id = 78515;
        let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
        let submitted_career = SubmittedCareer {
            account_id,
            career: create_dummy_career(),
            career_image1: (image1_file_name_without_ext, create_dummy_career_image1()),
            career_image2: None,
        };
        let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
        let op = SubmitCareerOperationMock {
            account_id,
            submitted_career: submitted_career.clone(),
            current_date_time,
            num_of_career: (MAX_NUM_OF_CAREER_PER_USER_ACCOUNT - 1) as usize,
            num_of_create_career_req: (MAX_NUM_OF_CAREER_PER_USER_ACCOUNT - 1) as usize,
            identity_exists: false,
        };
        let smtp_client = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            create_subject(account_id),
            create_text(account_id),
        );

        let result = handle_career_req(submitted_career, current_date_time, op, smtp_client).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::NoIdentityRegistered as u32, resp.1.code);
    }

    #[tokio::test]
    async fn handle_career_req_fail_reach_create_career_req_num_limit() {
        let account_id = 78515;
        let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
        let submitted_career = SubmittedCareer {
            account_id,
            career: create_dummy_career(),
            career_image1: (image1_file_name_without_ext, create_dummy_career_image1()),
            career_image2: None,
        };
        let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
        let op = SubmitCareerOperationMock {
            account_id,
            submitted_career: submitted_career.clone(),
            current_date_time,
            num_of_career: (MAX_NUM_OF_CAREER_PER_USER_ACCOUNT - 1) as usize,
            num_of_create_career_req: MAX_NUM_OF_CAREER_PER_USER_ACCOUNT as usize,
            identity_exists: true,
        };
        let smtp_client = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            create_subject(account_id),
            create_text(account_id),
        );

        let result = handle_career_req(submitted_career, current_date_time, op, smtp_client).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::ReachCreateCareerReqNumLimit as u32, resp.1.code);
    }

    // 基本的にMAX_NUM_OF_CAREER_PER_USER_ACCOUNTに達した場合、
    // CreateCareerReqを作れないのでありえないケースだが、テストは用意しておく
    #[tokio::test]
    async fn handle_career_req_fail_over_creqte_career_req_num_limit() {
        let account_id = 78515;
        let image1_file_name_without_ext = Uuid::new_v4().simple().to_string();
        let submitted_career = SubmittedCareer {
            account_id,
            career: create_dummy_career(),
            career_image1: (image1_file_name_without_ext, create_dummy_career_image1()),
            career_image2: None,
        };
        let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
        let op = SubmitCareerOperationMock {
            account_id,
            submitted_career: submitted_career.clone(),
            current_date_time,
            num_of_career: (MAX_NUM_OF_CAREER_PER_USER_ACCOUNT - 1) as usize,
            num_of_create_career_req: (MAX_NUM_OF_CAREER_PER_USER_ACCOUNT + 1) as usize,
            identity_exists: true,
        };
        let smtp_client = SendMailMock::new(
            ADMIN_EMAIL_ADDRESS.to_string(),
            SYSTEM_EMAIL_ADDRESS.to_string(),
            create_subject(account_id),
            create_text(account_id),
        );

        let result = handle_career_req(submitted_career, current_date_time, op, smtp_client).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::ReachCreateCareerReqNumLimit as u32, resp.1.code);
    }
}
