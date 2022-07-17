// Copyright 2022 Ken Miura

use async_session::async_trait;
use axum::http::StatusCode;
use axum::{Extension, Json};
use common::{ApiError, ErrResp, RespResult};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::util::validator::consultant_search_param::fee_per_hour_yen_param_validator::FeePerHourYenParamError;
use crate::util::validator::consultant_search_param::sort_param_validator::SortParamError;
use crate::{
    err::{unexpected_err_resp, Code},
    util::{
        session::User,
        validator::consultant_search_param::{
            career_param_validator::{validate_career_param, CareerParamValidationError},
            fee_per_hour_yen_param_validator::validate_fee_per_hour_yen_param,
            sort_param_validator::validate_sort_param,
        },
    },
};

pub(crate) const VALID_SIZE: i32 = 20;

pub(crate) async fn post_consultants_search(
    User { account_id }: User,
    Json(req): Json<ConsultantSearchParam>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<ConsultantsSearchResult> {
    let op = ConsultantsSearchOperationImpl { pool };
    handle_consultants_search(account_id, req, op).await
}

#[derive(Deserialize)]
pub(crate) struct ConsultantSearchParam {
    pub career_param: CareerParam,
    pub fee_per_hour_yen_param: FeePerHourYenParam,
    pub sort_param: Option<SortParam>,
    pub from: i32,
    pub size: i32,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CareerParam {
    pub company_name: Option<String>,
    pub department_name: Option<String>,
    pub office: Option<String>,
    pub years_of_service: Option<String>,
    pub employed: Option<bool>,
    pub contract_type: Option<String>,
    pub profession: Option<String>,
    pub annual_income_in_man_yen: AnnualInComeInManYenParam,
    pub is_manager: Option<bool>,
    pub position_name: Option<String>,
    pub is_new_graduate: Option<bool>,
    pub note: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AnnualInComeInManYenParam {
    pub equal_or_more: Option<i32>,
    pub equal_or_less: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct FeePerHourYenParam {
    pub equal_or_more: Option<i32>,
    pub equal_or_less: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SortParam {
    pub key: String,
    pub order: String,
}

#[derive(Serialize, Debug)]
pub(crate) struct ConsultantsSearchResult {
    total: i32,
    consultants: Vec<ConsultantDescription>,
}

#[derive(Serialize, Debug)]
pub(crate) struct ConsultantDescription {
    account_id: i64,
    fee_per_hour_in_yen: i32,
    rating: f64,
    num_of_rated: i32,
    careers: Vec<ConsultantCareerDescription>,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct ConsultantCareerDescription {
    company_name: String,
    profession: Option<String>,
    office: Option<String>,
}

async fn handle_consultants_search(
    account_id: i64,
    param: ConsultantSearchParam,
    op: impl ConsultantsSearchOperation,
) -> RespResult<ConsultantsSearchResult> {
    let _ = validate_career_param(&param.career_param).map_err(|e| {
        error!("invalid career_param: {} (account id: {})", e, account_id);
        create_invalid_career_param_err(&e)
    })?;
    let _ = validate_fee_per_hour_yen_param(&param.fee_per_hour_yen_param).map_err(|e| {
        error!(
            "invalid fee_per_hour_yen_param: {} (account id: {})",
            e, account_id
        );
        create_invalid_fee_per_hour_yen_param_err(&e)
    })?;
    if let Some(sort_param) = param.sort_param {
        let _ = validate_sort_param(&sort_param).map_err(|e| {
            error!("invalid sort_param: {} (account id: {})", e, account_id);
            create_invalid_sort_param_err(&e)
        });
    }
    if param.from.is_negative() {
        error!(
            "from is negative: {} (account id: {})",
            param.from, account_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidConsultantSearchParamFrom as u32,
            }),
        ));
    }
    if param.size != VALID_SIZE {
        error!("invalid size: {} (account id: {})", param.size, account_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidConsultantSearchParamSize as u32,
            }),
        ));
    }
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
    todo!()
}

#[async_trait]
trait ConsultantsSearchOperation {
    /// Identityが存在するか確認する。存在する場合、trueを返す。そうでない場合、falseを返す。
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
}

struct ConsultantsSearchOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultantsSearchOperation for ConsultantsSearchOperationImpl {
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
}

fn create_invalid_career_param_err(e: &CareerParamValidationError) -> ErrResp {
    let code;
    match e {
        CareerParamValidationError::InvalidCompanyNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = Code::InvalidCompanyNameLength,
        CareerParamValidationError::IllegalCharInCompanyName(_) => {
            code = Code::IllegalCharInCompanyName
        }
        CareerParamValidationError::InvalidDepartmentNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = Code::InvalidDepartmentNameLength,
        CareerParamValidationError::IllegalCharInDepartmentName(_) => {
            code = Code::IllegalCharInDepartmentName
        }
        CareerParamValidationError::InvalidOfficeLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = Code::InvalidOfficeLength,
        CareerParamValidationError::IllegalCharInOffice(_) => code = Code::IllegalCharInOffice,
        CareerParamValidationError::IllegalYearsOfService(_) => code = Code::IllegalYearsOfService,
        CareerParamValidationError::IllegalContractType(_) => code = Code::IllegalContractType,
        CareerParamValidationError::InvalidProfessionLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = Code::InvalidProfessionLength,
        CareerParamValidationError::IllegalCharInProfession(_) => {
            code = Code::IllegalCharInProfession
        }
        CareerParamValidationError::InvalidEqualOrMoreInAnnualIncomInManYen {
            value: _,
            min: _,
            max: _,
        } => code = Code::IllegalAnnualIncomInManYen,
        CareerParamValidationError::InvalidEqualOrLessInAnnualIncomInManYen {
            value: _,
            min: _,
            max: _,
        } => code = Code::IllegalAnnualIncomInManYen,
        CareerParamValidationError::EqualOrMoreExceedsEqualOrLessInAnnualIncomInManYen {
            equal_or_more: _,
            equal_or_less: _,
        } => code = Code::EqualOrMoreExceedsEqualOrLessInAnnualIncomInManYen,
        CareerParamValidationError::InvalidPositionNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = Code::InvalidPositionNameLength,
        CareerParamValidationError::IllegalCharInPositionName(_) => {
            code = Code::IllegalCharInPositionName
        }
        CareerParamValidationError::InvalidNoteLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = Code::InvalidNoteLength,
        CareerParamValidationError::IllegalCharInNote(_) => code = Code::IllegalCharInNote,
    }
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError { code: code as u32 }),
    )
}

fn create_invalid_fee_per_hour_yen_param_err(e: &FeePerHourYenParamError) -> ErrResp {
    let code;
    match e {
        FeePerHourYenParamError::InvalidEqualOrMore {
            value: _,
            min: _,
            max: _,
        } => code = Code::IllegalFeePerHourInYen,
        FeePerHourYenParamError::InvalidEqualOrLess {
            value: _,
            min: _,
            max: _,
        } => code = Code::IllegalFeePerHourInYen,
        FeePerHourYenParamError::EqualOrMoreExceedsEqualOrLess {
            equal_or_more: _,
            equal_or_less: _,
        } => code = Code::EqualOrMoreExceedsEqualOrLessInFeePerHourYen,
    }
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError { code: code as u32 }),
    )
}

fn create_invalid_sort_param_err(e: &SortParamError) -> ErrResp {
    let code;
    match e {
        SortParamError::InvalidKey(_) => code = Code::InvalidSortKey,
        SortParamError::InvalidOrder(_) => code = Code::InvalidSortOrder,
    }
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError { code: code as u32 }),
    )
}
