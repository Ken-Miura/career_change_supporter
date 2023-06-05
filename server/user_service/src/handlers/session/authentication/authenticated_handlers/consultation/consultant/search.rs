// Copyright 2022 Ken Miura

use async_session::async_trait;
use async_session::serde_json::{json, Value};
use axum::http::StatusCode;
use axum::{extract::State, Json};
use common::opensearch::{search_documents, Sort, INDEX_NAME};
use common::rating::round_rating_to_one_decimal_places;
use common::{ApiError, ErrResp, RespResult, MAX_NUM_OF_CAREER_PER_USER_ACCOUNT};
use opensearch::OpenSearch;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use super::career_param_validator::validate_career_param;
use super::fee_per_hour_in_yen_param_validator::FeePerHourInYenParamError;
use super::sort_param_validator::SortParamError;
use crate::err::Code;
use crate::handlers::session::authentication::authenticated_handlers::authenticated_users::verified_user::VerifiedUser;
use crate::handlers::session::authentication::authenticated_handlers::consultation::consultant::fee_per_hour_in_yen_param_validator::validate_fee_per_hour_in_yen_param;
use crate::handlers::session::authentication::authenticated_handlers::consultation::consultant::sort_param_validator::validate_sort_param;

use super::career_param_validator::CareerParamValidationError;

const VALID_SIZE: i64 = 20;

pub(crate) async fn post_consultants_search(
    VerifiedUser { user_info }: VerifiedUser,
    State(index_client): State<OpenSearch>,
    Json(req): Json<ConsultantSearchParam>,
) -> RespResult<ConsultantsSearchResult> {
    let op = ConsultantsSearchOperationImpl { index_client };
    handle_consultants_search(user_info.account_id, req, op).await
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ConsultantSearchParam {
    career_param: CareerParam,
    fee_per_hour_in_yen_param: FeePerHourInYenParam,
    sort_param: Option<SortParam>,
    from: i64,
    size: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct CareerParam {
    pub(super) company_name: Option<String>,
    pub(super) department_name: Option<String>,
    pub(super) office: Option<String>,
    pub(super) years_of_service: YearsOfServiceParam,
    pub(super) employed: Option<bool>,
    pub(super) contract_type: Option<String>,
    pub(super) profession: Option<String>,
    pub(super) annual_income_in_man_yen: AnnualInComeInManYenParam,
    pub(super) is_manager: Option<bool>,
    pub(super) position_name: Option<String>,
    pub(super) is_new_graduate: Option<bool>,
    pub(super) note: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct YearsOfServiceParam {
    pub(super) equal_or_more: Option<i32>,
    pub(super) less_than: Option<i32>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct AnnualInComeInManYenParam {
    pub(super) equal_or_more: Option<i32>,
    pub(super) equal_or_less: Option<i32>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct FeePerHourInYenParam {
    pub(super) equal_or_more: Option<i32>,
    pub(super) equal_or_less: Option<i32>,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct SortParam {
    pub(super) key: String,
    pub(super) order: String,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct ConsultantsSearchResult {
    total: i64,
    consultants: Vec<ConsultantDescription>,
}

#[derive(Clone, Serialize, Debug, PartialEq)]
pub(crate) struct ConsultantDescription {
    consultant_id: i64,
    fee_per_hour_in_yen: i32,
    rating: Option<String>, // 適切な型は浮動少数だが、PartialEqの==を正しく動作させるために文字列として処理する
    num_of_rated: i32,
    careers: Vec<ConsultantCareerDescription>,
}

#[derive(Clone, Serialize, Debug, PartialEq)]
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
    validate_career_param(&param.career_param).map_err(|e| {
        error!("invalid career_param: {} (account id: {})", e, account_id);
        create_invalid_career_param_err(&e)
    })?;
    validate_fee_per_hour_in_yen_param(&param.fee_per_hour_in_yen_param).map_err(|e| {
        error!(
            "invalid fee_per_hour_in_yen_param: {} (account id: {})",
            e, account_id
        );
        create_invalid_fee_per_hour_in_yen_param_err(&e)
    })?;
    if let Some(sort_param) = param.sort_param.clone() {
        validate_sort_param(&sort_param).map_err(|e| {
            error!("invalid sort_param: {} (account id: {})", e, account_id);
            create_invalid_sort_param_err(&e)
        })?;
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

    info!(
        "query param (account_id: {}, career_param: {:?}, fee_per_hour_in_yen_param: {:?}, sort_param: {:?})",
        account_id, param.career_param, param.fee_per_hour_in_yen_param, param.sort_param
    );
    let query = create_query_json(
        account_id,
        param.career_param,
        param.fee_per_hour_in_yen_param,
    )?;
    let sort = param.sort_param.map(|s| Sort {
        key: s.key,
        order: s.order,
    });
    let query_result = op
        .search_documents(INDEX_NAME, param.from, param.size, sort, &query)
        .await?;

    parse_query_result(query_result)
}

#[async_trait]
trait ConsultantsSearchOperation {
    async fn search_documents(
        &self,
        index_name: &str,
        from: i64,
        size: i64,
        sort: Option<Sort>,
        query: &Value,
    ) -> Result<Value, ErrResp>;
}

struct ConsultantsSearchOperationImpl {
    index_client: OpenSearch,
}

#[async_trait]
impl ConsultantsSearchOperation for ConsultantsSearchOperationImpl {
    async fn search_documents(
        &self,
        index_name: &str,
        from: i64,
        size: i64,
        sort: Option<Sort>,
        query: &Value,
    ) -> Result<Value, ErrResp> {
        let result =
            search_documents(index_name, from, size, sort, query, &self.index_client).await?;
        Ok(result)
    }
}

fn create_invalid_career_param_err(e: &CareerParamValidationError) -> ErrResp {
    let code = match e {
        CareerParamValidationError::InvalidCompanyNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => Code::InvalidCompanyNameLength,
        CareerParamValidationError::IllegalCharInCompanyName(_) => Code::IllegalCharInCompanyName,
        CareerParamValidationError::InvalidDepartmentNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => Code::InvalidDepartmentNameLength,
        CareerParamValidationError::IllegalCharInDepartmentName(_) => {
            Code::IllegalCharInDepartmentName
        }
        CareerParamValidationError::InvalidOfficeLength {
            length: _,
            min_length: _,
            max_length: _,
        } => Code::InvalidOfficeLength,
        CareerParamValidationError::IllegalCharInOffice(_) => Code::IllegalCharInOffice,
        CareerParamValidationError::IllegalYearsOfService(_) => Code::IllegalYearsOfService,
        CareerParamValidationError::EqualOrMoreIsLessThanOrMoreYearsOfService {
            equal_or_more: _,
            less_than: _,
        } => Code::EqualOrMoreIsLessThanOrMoreYearsOfService,
        CareerParamValidationError::IllegalContractType(_) => Code::IllegalContractType,
        CareerParamValidationError::InvalidProfessionLength {
            length: _,
            min_length: _,
            max_length: _,
        } => Code::InvalidProfessionLength,
        CareerParamValidationError::IllegalCharInProfession(_) => Code::IllegalCharInProfession,
        CareerParamValidationError::InvalidEqualOrMoreInAnnualIncomeInManYen {
            value: _,
            min: _,
            max: _,
        } => Code::IllegalAnnualIncomeInManYen,
        CareerParamValidationError::InvalidEqualOrLessInAnnualIncomeInManYen {
            value: _,
            min: _,
            max: _,
        } => Code::IllegalAnnualIncomeInManYen,
        CareerParamValidationError::EqualOrMoreExceedsEqualOrLessInAnnualIncomeInManYen {
            equal_or_more: _,
            equal_or_less: _,
        } => Code::EqualOrMoreExceedsEqualOrLessInAnnualIncomeInManYen,
        CareerParamValidationError::InvalidPositionNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => Code::InvalidPositionNameLength,
        CareerParamValidationError::IllegalCharInPositionName(_) => Code::IllegalCharInPositionName,
        CareerParamValidationError::InvalidNoteLength {
            length: _,
            min_length: _,
            max_length: _,
        } => Code::InvalidNoteLength,
        CareerParamValidationError::IllegalCharInNote(_) => Code::IllegalCharInNote,
    };
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError { code: code as u32 }),
    )
}

fn create_invalid_fee_per_hour_in_yen_param_err(e: &FeePerHourInYenParamError) -> ErrResp {
    let code = match e {
        FeePerHourInYenParamError::InvalidEqualOrMore {
            value: _,
            min: _,
            max: _,
        } => Code::IllegalFeePerHourInYen,
        FeePerHourInYenParamError::InvalidEqualOrLess {
            value: _,
            min: _,
            max: _,
        } => Code::IllegalFeePerHourInYen,
        FeePerHourInYenParamError::EqualOrMoreExceedsEqualOrLess {
            equal_or_more: _,
            equal_or_less: _,
        } => Code::EqualOrMoreExceedsEqualOrLessInFeePerHourInYen,
    };
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError { code: code as u32 }),
    )
}

fn create_invalid_sort_param_err(e: &SortParamError) -> ErrResp {
    let code = match e {
        SortParamError::InvalidKey(_) => Code::InvalidSortKey,
        SortParamError::InvalidOrder(_) => Code::InvalidSortOrder,
    };
    (
        StatusCode::BAD_REQUEST,
        Json(ApiError { code: code as u32 }),
    )
}

fn create_query_json(
    account_id: i64,
    career_param: CareerParam,
    fee_per_hour_in_yen_param: FeePerHourInYenParam,
) -> Result<Value, ErrResp> {
    let mut params = Vec::<Value>::new();
    if let Some(company_name) = career_param.company_name {
        let company_name_criteria = create_company_name_criteria(company_name.as_str());
        params.push(company_name_criteria);
    }
    if let Some(department_name) = career_param.department_name {
        let department_name_criteria = create_department_name_criteria(department_name.as_str());
        params.push(department_name_criteria);
    }
    if let Some(office) = career_param.office {
        let office_criteria = create_office_criteria(office.as_str());
        params.push(office_criteria);
    }
    if let Some(equal_or_more) = career_param.years_of_service.equal_or_more {
        let years_of_service_equal_or_more_criteria =
            create_years_of_service_equal_or_more_criteria(equal_or_more);
        params.push(years_of_service_equal_or_more_criteria);
    }
    if let Some(less_than) = career_param.years_of_service.less_than {
        let years_of_service_less_than_criteria =
            create_years_of_service_less_than_criteria(less_than);
        params.push(years_of_service_less_than_criteria);
    }
    if let Some(employed) = career_param.employed {
        let employed_criteria = create_employed_criteria(employed);
        params.push(employed_criteria);
    }
    if let Some(contract_type) = career_param.contract_type {
        let contract_type_criteria = create_contract_type_criteria(contract_type.as_str());
        params.push(contract_type_criteria);
    }
    if let Some(profession) = career_param.profession {
        let profession_criteria = create_profession_criteria(profession.as_str());
        params.push(profession_criteria);
    }
    if let Some(equal_or_more) = career_param.annual_income_in_man_yen.equal_or_more {
        let equal_or_more_criteria =
            create_annual_income_in_man_yen_equal_or_more_criteria(equal_or_more);
        params.push(equal_or_more_criteria);
    }
    if let Some(equal_or_less) = career_param.annual_income_in_man_yen.equal_or_less {
        let equal_or_less_criteria =
            create_annual_income_in_man_yen_equal_or_less_criteria(equal_or_less);
        params.push(equal_or_less_criteria);
    }
    if let Some(is_manager) = career_param.is_manager {
        let is_manager_criteria = create_is_manager_criteria(is_manager);
        params.push(is_manager_criteria);
    }
    if let Some(position_name) = career_param.position_name {
        let position_name_criteria = create_position_name_criteria(position_name.as_str());
        params.push(position_name_criteria);
    }
    if let Some(is_new_graduate) = career_param.is_new_graduate {
        let is_new_graduate_criteria = create_is_new_graduate_criteria(is_new_graduate);
        params.push(is_new_graduate_criteria);
    }
    if let Some(note) = career_param.note {
        let note_criteria = create_note_criteria(note.as_str());
        params.push(note_criteria);
    }
    if let Some(equal_or_more) = fee_per_hour_in_yen_param.equal_or_more {
        let equal_or_more_criteria =
            create_fee_per_hour_in_yen_equal_or_more_criteria(equal_or_more);
        params.push(equal_or_more_criteria);
    }
    if let Some(equal_or_less) = fee_per_hour_in_yen_param.equal_or_less {
        let equal_or_less_criteria =
            create_fee_per_hour_in_yen_equal_or_less_criteria(equal_or_less);
        params.push(equal_or_less_criteria);
    }
    Ok(generate_query_json(account_id, params))
}

fn create_company_name_criteria(company_name: &str) -> Value {
    json!({
        "nested": {
            "path": "careers",
            "query": {
                "bool": {
                    "must": [
                        {
                            "multi_match": {
                                "query": company_name,
                                "fields": [
                                    "careers.company_name.ngram^1"
                                ],
                                "type": "phrase"
                            }
                        }
                    ],
                    "should": [
                        {
                            "multi_match": {
                                "query": company_name,
                                "fields": [
                                    "careers.company_name^1"
                                ],
                                "type": "phrase"
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn create_department_name_criteria(department_name: &str) -> Value {
    json!({
        "nested": {
            "path": "careers",
            "query": {
                "bool": {
                    "must": [
                        {
                            "multi_match": {
                                "query": department_name,
                                "fields": [
                                    "careers.department_name.ngram^1"
                                ],
                                "type": "phrase"
                            }
                        }
                    ],
                    "should": [
                        {
                            "multi_match": {
                                "query": department_name,
                                "fields": [
                                    "careers.department_name^1"
                                ],
                                "type": "phrase"
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn create_office_criteria(office: &str) -> Value {
    json!({
        "nested": {
            "path": "careers",
            "query": {
                "bool": {
                    "must": [
                        {
                            "multi_match": {
                                "query": office,
                                "fields": [
                                    "careers.office.ngram^1"
                                ],
                                "type": "phrase"
                            }
                        }
                    ],
                    "should": [
                        {
                            "multi_match": {
                                "query": office,
                                "fields": [
                                    "careers.office^1"
                                ],
                                "type": "phrase"
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn create_years_of_service_equal_or_more_criteria(years_of_service: i32) -> Value {
    json!({
        "nested": {
            "path": "careers",
            "query": {
                "bool": {
                    "must": [
                        {
                            "range": {
                                "careers.years_of_service": {
                                    "gte": years_of_service
                                }
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn create_years_of_service_less_than_criteria(years_of_service: i32) -> Value {
    json!({
        "nested": {
            "path": "careers",
            "query": {
                "bool": {
                    "must": [
                        {
                            "range": {
                                "careers.years_of_service": {
                                    "lt": years_of_service
                                }
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn create_employed_criteria(employed: bool) -> Value {
    json!({
        "nested": {
            "path": "careers",
            "query": {
                "bool": {
                    "must": [
                        {
                            "term": {
                                "careers.employed": employed
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn create_contract_type_criteria(contract_type: &str) -> Value {
    json!({
        "nested": {
            "path": "careers",
            "query": {
                "bool": {
                    "must": [
                        {
                            "multi_match": {
                                "query": contract_type,
                                "fields": [
                                    "careers.contract_type.ngram^1"
                                ],
                                "type": "phrase"
                            }
                        }
                    ],
                    "should": [
                        {
                            "multi_match": {
                                "query": contract_type,
                                "fields": [
                                    "careers.contract_type^1"
                                ],
                                "type": "phrase"
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn create_profession_criteria(profession: &str) -> Value {
    json!({
        "nested": {
            "path": "careers",
            "query": {
                "bool": {
                    "must": [
                        {
                            "multi_match": {
                                "query": profession,
                                "fields": [
                                    "careers.profession.ngram^1"
                                ],
                                "type": "phrase"
                            }
                        }
                    ],
                    "should": [
                        {
                            "multi_match": {
                                "query": profession,
                                "fields": [
                                    "careers.profession^1"
                                ],
                                "type": "phrase"
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn create_annual_income_in_man_yen_equal_or_more_criteria(equal_or_more: i32) -> Value {
    json!({
        "nested": {
            "path": "careers",
            "query": {
                "bool": {
                    "must": [
                        {
                            "range": {
                                "careers.years_of_service": {
                                    "gte": equal_or_more
                                }
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn create_annual_income_in_man_yen_equal_or_less_criteria(equal_or_less: i32) -> Value {
    json!({
        "nested": {
            "path": "careers",
            "query": {
                "bool": {
                    "must": [
                        {
                            "range": {
                                "careers.years_of_service": {
                                    "lte": equal_or_less
                                }
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn create_is_manager_criteria(is_manager: bool) -> Value {
    json!({
        "nested": {
            "path": "careers",
            "query": {
                "bool": {
                    "must": [
                        {
                            "term": {
                                "careers.is_manager": is_manager
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn create_position_name_criteria(position_name: &str) -> Value {
    json!({
        "nested": {
            "path": "careers",
            "query": {
                "bool": {
                    "must": [
                        {
                            "multi_match": {
                                "query": position_name,
                                "fields": [
                                    "careers.position_name.ngram^1"
                                ],
                                "type": "phrase"
                            }
                        }
                    ],
                    "should": [
                        {
                            "multi_match": {
                                "query": position_name,
                                "fields": [
                                    "careers.position_name^1"
                                ],
                                "type": "phrase"
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn create_is_new_graduate_criteria(is_new_graduate: bool) -> Value {
    json!({
        "nested": {
            "path": "careers",
            "query": {
                "bool": {
                    "must": [
                        {
                            "term": {
                                "careers.is_new_graduate": is_new_graduate
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn create_note_criteria(note: &str) -> Value {
    json!({
        "nested": {
            "path": "careers",
            "query": {
                "bool": {
                    "must": [
                        {
                            "multi_match": {
                                "query": note,
                                "fields": [
                                    "careers.note.ngram^1"
                                ],
                                "type": "phrase"
                            }
                        }
                    ],
                    "should": [
                        {
                            "multi_match": {
                                "query": note,
                                "fields": [
                                    "careers.note^1"
                                ],
                                "type": "phrase"
                            }
                        }
                    ]
                }
            }
        }
    })
}

fn create_fee_per_hour_in_yen_equal_or_more_criteria(equal_or_more: i32) -> Value {
    json!({
        "range": {
            "fee_per_hour_in_yen": {
                "gte": equal_or_more
            }
        }
    })
}

fn create_fee_per_hour_in_yen_equal_or_less_criteria(equal_or_less: i32) -> Value {
    json!({
        "range": {
            "fee_per_hour_in_yen": {
                "lte": equal_or_less
            }
        }
    })
}

fn generate_query_json(account_id: i64, params: Vec<Value>) -> Value {
    json!({
        "query": {
            "bool": {
                "must": params,
                "filter": [
                    {
                        "range": {
                            "num_of_careers": {
                                "gt": 0
                            }
                        }
                    },
                    {
                        "exists": {
                            "field": "fee_per_hour_in_yen"
                        }
                    },
                    {
                        "term": {
                            "is_bank_account_registered": true
                        }
                    },
                    {
                        "term": {
                            "disabled": false
                        }
                    }
                ],
                "must_not": [
                    {
                        "term": {
                            "user_account_id": account_id
                        }
                    }
                ]
            }
        }
    })
}

fn parse_query_result(query_result: Value) -> RespResult<ConsultantsSearchResult> {
    let took = query_result["took"].as_i64().ok_or_else(|| {
        error!("failed to get processing time: {}", query_result);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    info!("opensearch took {} milliseconds", took);

    let total = query_result["hits"]["total"]["value"]
        .as_i64()
        .ok_or_else(|| {
            error!("failed to get total value: {}", query_result);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: Code::UnexpectedErr as u32,
                }),
            )
        })?;
    let hits = query_result["hits"]["hits"].as_array().ok_or_else(|| {
        error!("failed to get hits: {}", query_result);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    let mut consultants = Vec::with_capacity(VALID_SIZE as usize);
    for hit in hits {
        let consultant_description = create_consultant_description(hit)?;
        consultants.push(consultant_description);
    }
    // NOTE: 将来的にパフォーマンスに影響する場合、ログに出力する内容を制限する
    info!("total: {}, consultants: {:?}", total, consultants);
    let results = ConsultantsSearchResult { total, consultants };
    Ok((StatusCode::OK, Json(results)))
}

fn create_consultant_description(hit: &Value) -> Result<ConsultantDescription, ErrResp> {
    let account_id = hit["_source"]["user_account_id"].as_i64().ok_or_else(|| {
        error!("failed to find account id in _source: {:?}", hit);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    let fee_per_hour_in_yen = hit["_source"]["fee_per_hour_in_yen"]
        .as_i64()
        .ok_or_else(|| {
            error!(
                "failed to find fee_per_hour_in_yen id in _source: {:?}",
                hit
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: Code::UnexpectedErr as u32,
                }),
            )
        })?;
    let rating = hit["_source"]["rating"].as_f64();
    let num_of_rated = hit["_source"]["num_of_rated"].as_i64().unwrap_or(0);
    // 検索条件で num_of_careers > 0 を指定しているので、careersがないケースはエラーとして扱う
    let careers = hit["_source"]["careers"].as_array().ok_or_else(|| {
        error!("failed to find careers id in _source: {:?}", hit);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    let mut consultant_career_descriptions =
        Vec::with_capacity(MAX_NUM_OF_CAREER_PER_USER_ACCOUNT as usize);
    for career in careers {
        let career_description = create_consultant_career_description(career)?;
        consultant_career_descriptions.push(career_description);
    }
    Ok(ConsultantDescription {
        consultant_id: account_id,
        fee_per_hour_in_yen: fee_per_hour_in_yen as i32,
        rating: rating.map(round_rating_to_one_decimal_places),
        num_of_rated: num_of_rated as i32,
        careers: consultant_career_descriptions,
    })
}

fn create_consultant_career_description(
    career: &Value,
) -> Result<ConsultantCareerDescription, ErrResp> {
    let company_name = career["company_name"].as_str().ok_or_else(|| {
        error!("failed to find company_name in career: {:?}", career);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    let profession = career["profession"].as_str();
    let office = career["office"].as_str();
    Ok(ConsultantCareerDescription {
        company_name: company_name.to_string(),
        profession: profession.map(|s| s.to_string()),
        office: office.map(|s| s.to_string()),
    })
}

#[cfg(test)]
mod tests {

    use once_cell::sync::Lazy;

    use crate::handlers::session::authentication::authenticated_handlers::fee_per_hour_in_yen_range::MIN_FEE_PER_HOUR_IN_YEN;

    use super::*;

    #[derive(Clone, Debug)]
    struct ConsultantsSearchOperationMock {
        query_result: Value,
    }

    #[async_trait]
    impl ConsultantsSearchOperation for ConsultantsSearchOperationMock {
        async fn search_documents(
            &self,
            _index_name: &str,
            _from: i64,
            _size: i64,
            _sort: Option<Sort>,
            _query: &Value,
        ) -> Result<Value, ErrResp> {
            Ok(self.query_result.clone())
        }
    }

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<ConsultantsSearchResult>,
    }

    #[derive(Debug)]
    struct Input {
        account_id: i64,
        param: ConsultantSearchParam,
        op: ConsultantsSearchOperationMock,
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        vec![
            TestCase {
                name: "all parameters specified".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: Some("テスト１".to_string()),
                            department_name: Some("テスト２".to_string()),
                            office: Some("テスト３".to_string()),
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: Some(3),
                                less_than: None,
                            },
                            employed: Some(true),
                            contract_type: Some("regular".to_string()),
                            profession: Some("テスト４".to_string()),
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: Some(300),
                                equal_or_less: Some(800),
                            },
                            is_manager: Some(false),
                            position_name: Some("テスト５".to_string()),
                            is_new_graduate: Some(true),
                            note: Some("テスト６".to_string()),
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: Some(3000),
                            equal_or_less: Some(10000),
                        },
                        sort_param: Some(SortParam {
                            key: "rating".to_string(),
                            order: "asc".to_string(),
                        }),
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: json!({
                          "took" : 2,
                          "timed_out" : false,
                          "_shards" : {
                            "total" : 1,
                            "successful" : 1,
                            "skipped" : 0,
                            "failed" : 0
                          },
                          "hits" : {
                            "total" : {
                              "value" : 1,
                              "relation" : "eq"
                            },
                            "max_score" : 1.0,
                            "hits" : [
                                {
                                    "_index" : "users",
                                    "_id" : "2",
                                    "_score" : 1.0,
                                    "_source" : {
                                      "careers" : [
                                        {
                                          "annual_income_in_man_yen" : null,
                                          "career_id" : 1,
                                          "company_name" : "テスト１",
                                          "contract_type" : "regular",
                                          "department_name" : "テスト２",
                                          "employed" : true,
                                          "is_manager" : false,
                                          "is_new_graduate" : true,
                                          "note" : "テスト６",
                                          "office" : "テスト３",
                                          "position_name" : "テスト５",
                                          "profession" : "テスト４",
                                          "years_of_service" : 3
                                        }
                                    ],
                                    "fee_per_hour_in_yen" : 4500,
                                    "is_bank_account_registered" : true,
                                    "disabled" : false,
                                    "num_of_careers" : 1,
                                    "rating" : null,
                                    "num_of_rated": 0,
                                    "user_account_id" : 2
                                  }
                                }
                            ]
                          }
                        }),
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultantsSearchResult {
                        total: 1,
                        consultants: vec![ConsultantDescription {
                            consultant_id: 2,
                            fee_per_hour_in_yen: 4500,
                            rating: None,
                            num_of_rated: 0,
                            careers: vec![ConsultantCareerDescription {
                                company_name: "テスト１".to_string(),
                                profession: Some("テスト４".to_string()),
                                office: Some("テスト３".to_string()),
                            }],
                        }],
                    }),
                )),
            },
            TestCase {
                name: "no parameters specified".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: json!({
                          "took" : 2,
                          "timed_out" : false,
                          "_shards" : {
                            "total" : 1,
                            "successful" : 1,
                            "skipped" : 0,
                            "failed" : 0
                          },
                          "hits" : {
                            "total" : {
                              "value" : 1,
                              "relation" : "eq"
                            },
                            "max_score" : 1.0,
                            "hits" : [
                                {
                                    "_index" : "users",
                                    "_id" : "2",
                                    "_score" : 1.0,
                                    "_source" : {
                                      "careers" : [
                                        {
                                          "annual_income_in_man_yen" : null,
                                          "career_id" : 1,
                                          "company_name" : "テスト１",
                                          "contract_type" : "regular",
                                          "department_name" : "テスト２",
                                          "employed" : true,
                                          "is_manager" : false,
                                          "is_new_graduate" : true,
                                          "note" : "テスト６",
                                          "office" : "テスト３",
                                          "position_name" : "テスト５",
                                          "profession" : "テスト４",
                                          "years_of_service" : 3
                                        }
                                    ],
                                    "fee_per_hour_in_yen" : 4500,
                                    "is_bank_account_registered" : true,
                                    "disabled" : false,
                                    "num_of_careers" : 1,
                                    "rating" : null,
                                    "num_of_rated": 0,
                                    "user_account_id" : 2
                                  }
                                }
                            ]
                          }
                        }),
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(ConsultantsSearchResult {
                        total: 1,
                        consultants: vec![ConsultantDescription {
                            consultant_id: 2,
                            fee_per_hour_in_yen: 4500,
                            rating: None,
                            num_of_rated: 0,
                            careers: vec![ConsultantCareerDescription {
                                company_name: "テスト１".to_string(),
                                profession: Some("テスト４".to_string()),
                                office: Some("テスト３".to_string()),
                            }],
                        }],
                    }),
                )),
            },
            TestCase {
                name: "invalid company name length".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: Some("".to_string()),
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidCompanyNameLength as u32,
                    }),
                )),
            },
            TestCase {
                name: "illegal company name".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: Some("*".to_string()),
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalCharInCompanyName as u32,
                    }),
                )),
            },
            TestCase {
                name: "invalid department name length".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: Some("".to_string()),
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidDepartmentNameLength as u32,
                    }),
                )),
            },
            TestCase {
                name: "illegal department name".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: Some("*".to_string()),
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalCharInDepartmentName as u32,
                    }),
                )),
            },
            TestCase {
                name: "invalid office length".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: Some("".to_string()),
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidOfficeLength as u32,
                    }),
                )),
            },
            TestCase {
                name: "illegal office".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: Some("*".to_string()),
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalCharInOffice as u32,
                    }),
                )),
            },
            TestCase {
                name: "illegal years of service".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: Some(-1),
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalYearsOfService as u32,
                    }),
                )),
            },
            TestCase {
                name: "illegal contract type".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: Some("*".to_string()),
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalContractType as u32,
                    }),
                )),
            },
            TestCase {
                name: "invalid profession length".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: Some("".to_string()),
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidProfessionLength as u32,
                    }),
                )),
            },
            TestCase {
                name: "illegal profession".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: Some("*".to_string()),
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalCharInProfession as u32,
                    }),
                )),
            },
            TestCase {
                name: "illegal equal_or_more annual income in man yen".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: Some(-1),
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalAnnualIncomeInManYen as u32,
                    }),
                )),
            },
            TestCase {
                name: "illegal equal_or_less annual income in man yen".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: Some(-1),
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalAnnualIncomeInManYen as u32,
                    }),
                )),
            },
            TestCase {
                name: "equal_or_more exceeds equal_or_less annual income in man yen".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: Some(1),
                                equal_or_less: Some(0),
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::EqualOrMoreExceedsEqualOrLessInAnnualIncomeInManYen as u32,
                    }),
                )),
            },
            TestCase {
                name: "invalid position name length".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: Some("".to_string()),
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidPositionNameLength as u32,
                    }),
                )),
            },
            TestCase {
                name: "illegal position name".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: Some("*".to_string()),
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalCharInPositionName as u32,
                    }),
                )),
            },
            TestCase {
                name: "invalid note length".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: Some("".to_string()),
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidNoteLength as u32,
                    }),
                )),
            },
            TestCase {
                name: "illegal note".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: Some("*".to_string()),
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalCharInNote as u32,
                    }),
                )),
            },
            TestCase {
                name: "illegal equal_or_more fee_per_hour_in_yen".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: Some(-1),
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalFeePerHourInYen as u32,
                    }),
                )),
            },
            TestCase {
                name: "illegal equal_or_less fee_per_hour_in_yen".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: Some(-1),
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalFeePerHourInYen as u32,
                    }),
                )),
            },
            TestCase {
                name: "equal_or_more exceeds equal_or_less fee_per_hour_in_yen".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: Some(MIN_FEE_PER_HOUR_IN_YEN + 1),
                            equal_or_less: Some(MIN_FEE_PER_HOUR_IN_YEN),
                        },
                        sort_param: None,
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::EqualOrMoreExceedsEqualOrLessInFeePerHourInYen as u32,
                    }),
                )),
            },
            TestCase {
                name: "invalid sort key".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: Some(SortParam {
                            key: "*".to_string(),
                            order: "asc".to_string(),
                        }),
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidSortKey as u32,
                    }),
                )),
            },
            TestCase {
                name: "invalid sort order".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: Some(SortParam {
                            key: "fee_per_hour_in_yen".to_string(),
                            order: "*".to_string(),
                        }),
                        from: 0,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidSortOrder as u32,
                    }),
                )),
            },
            TestCase {
                name: "invalid from".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: -1,
                        size: 20,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidConsultantSearchParamFrom as u32,
                    }),
                )),
            },
            TestCase {
                name: "invalid size".to_string(),
                input: Input {
                    account_id: 1,
                    param: ConsultantSearchParam {
                        career_param: CareerParam {
                            company_name: None,
                            department_name: None,
                            office: None,
                            years_of_service: YearsOfServiceParam {
                                equal_or_more: None,
                                less_than: None,
                            },
                            employed: None,
                            contract_type: None,
                            profession: None,
                            annual_income_in_man_yen: AnnualInComeInManYenParam {
                                equal_or_more: None,
                                equal_or_less: None,
                            },
                            is_manager: None,
                            position_name: None,
                            is_new_graduate: None,
                            note: None,
                        },
                        fee_per_hour_in_yen_param: FeePerHourInYenParam {
                            equal_or_more: None,
                            equal_or_less: None,
                        },
                        sort_param: None,
                        from: 0,
                        size: 21,
                    },
                    op: ConsultantsSearchOperationMock {
                        query_result: create_empty_result(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidConsultantSearchParamSize as u32,
                    }),
                )),
            },
        ]
    });

    fn create_empty_result() -> Value {
        json!({
          "took" : 26,
          "timed_out" : false,
          "_shards" : {
            "total" : 1,
            "successful" : 1,
            "skipped" : 0,
            "failed" : 0
          },
          "hits" : {
            "total" : {
              "value" : 0,
              "relation" : "eq"
            },
            "max_score" : null,
            "hits" : [ ]
          }
        })
    }

    #[tokio::test]
    async fn test_handle_consultants_search() {
        for test_case in TEST_CASE_SET.iter() {
            let resp = handle_consultants_search(
                test_case.input.account_id,
                test_case.input.param.clone(),
                test_case.input.op.clone(),
            )
            .await;

            let message = format!("test case \"{}\" failed", test_case.name.clone());
            if test_case.expected.is_ok() {
                let result = resp.expect("failed to get Ok");
                let expected_result = test_case.expected.as_ref().expect("failed to get Ok");
                assert_eq!(expected_result.0, result.0, "{}", message);
                assert_eq!(expected_result.1 .0, result.1 .0, "{}", message);
            } else {
                let result = resp.expect_err("failed to get Err");
                let expected_result = test_case.expected.as_ref().expect_err("failed to get Err");
                assert_eq!(expected_result.0, result.0, "{}", message);
                assert_eq!(expected_result.1 .0, result.1 .0, "{}", message);
            }
        }
    }
}
