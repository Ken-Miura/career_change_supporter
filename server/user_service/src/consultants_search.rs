// Copyright 2022 Ken Miura

use async_session::async_trait;
use async_session::serde_json::{json, Value};
use axum::http::StatusCode;
use axum::{Extension, Json};
use common::opensearch::{search_documents, Sort, INDEX_NAME};
use common::{ApiError, ErrResp, RespResult, MAX_NUM_OF_CAREER_PER_USER_ACCOUNT};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use opensearch::OpenSearch;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::util::validator::consultant_search_param::fee_per_hour_yen_param_validator::FeePerHourYenParamError;
use crate::util::validator::consultant_search_param::sort_param_validator::SortParamError;
use crate::util::{
    YEARS_OF_SERVICE_FIFTEEN_YEARS_OR_MORE, YEARS_OF_SERVICE_FIVE_YEARS_OR_MORE,
    YEARS_OF_SERVICE_TEN_YEARS_OR_MORE, YEARS_OF_SERVICE_THREE_YEARS_OR_MORE,
    YEARS_OF_SERVICE_TWENTY_YEARS_OR_MORE,
};
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

pub(crate) const VALID_SIZE: i64 = 20;

pub(crate) async fn post_consultants_search(
    User { account_id }: User,
    Json(req): Json<ConsultantSearchParam>,
    Extension(pool): Extension<DatabaseConnection>,
    Extension(index_client): Extension<OpenSearch>,
) -> RespResult<ConsultantsSearchResult> {
    let op = ConsultantsSearchOperationImpl { pool, index_client };
    handle_consultants_search(account_id, req, op).await
}

#[derive(Deserialize)]
pub(crate) struct ConsultantSearchParam {
    pub career_param: CareerParam,
    pub fee_per_hour_yen_param: FeePerHourYenParam,
    pub sort_param: Option<SortParam>,
    pub from: i64,
    pub size: i64,
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

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct SortParam {
    pub key: String,
    pub order: String,
}

#[derive(Serialize, Debug)]
pub(crate) struct ConsultantsSearchResult {
    total: i64,
    consultants: Vec<ConsultantDescription>,
}

#[derive(Serialize, Debug)]
pub(crate) struct ConsultantDescription {
    account_id: i64,
    fee_per_hour_in_yen: i32,
    rating: Option<f64>,
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
    if let Some(sort_param) = param.sort_param.clone() {
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

    info!(
        "query param (account_id: {}, career_param: {:?}, fee_per_hour_yen_param: {:?}, sort_param: {:?})",
        account_id, param.career_param, param.fee_per_hour_yen_param, param.sort_param
    );
    let query = create_query_json(account_id, param.career_param, param.fee_per_hour_yen_param)?;
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
    /// Identityが存在するか確認する。存在する場合、trueを返す。そうでない場合、falseを返す。
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
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
    pool: DatabaseConnection,
    index_client: OpenSearch,
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
        CareerParamValidationError::IllegalContractType(_) => Code::IllegalContractType,
        CareerParamValidationError::InvalidProfessionLength {
            length: _,
            min_length: _,
            max_length: _,
        } => Code::InvalidProfessionLength,
        CareerParamValidationError::IllegalCharInProfession(_) => Code::IllegalCharInProfession,
        CareerParamValidationError::InvalidEqualOrMoreInAnnualIncomInManYen {
            value: _,
            min: _,
            max: _,
        } => Code::IllegalAnnualIncomInManYen,
        CareerParamValidationError::InvalidEqualOrLessInAnnualIncomInManYen {
            value: _,
            min: _,
            max: _,
        } => Code::IllegalAnnualIncomInManYen,
        CareerParamValidationError::EqualOrMoreExceedsEqualOrLessInAnnualIncomInManYen {
            equal_or_more: _,
            equal_or_less: _,
        } => Code::EqualOrMoreExceedsEqualOrLessInAnnualIncomInManYen,
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

fn create_invalid_fee_per_hour_yen_param_err(e: &FeePerHourYenParamError) -> ErrResp {
    let code = match e {
        FeePerHourYenParamError::InvalidEqualOrMore {
            value: _,
            min: _,
            max: _,
        } => Code::IllegalFeePerHourInYen,
        FeePerHourYenParamError::InvalidEqualOrLess {
            value: _,
            min: _,
            max: _,
        } => Code::IllegalFeePerHourInYen,
        FeePerHourYenParamError::EqualOrMoreExceedsEqualOrLess {
            equal_or_more: _,
            equal_or_less: _,
        } => Code::EqualOrMoreExceedsEqualOrLessInFeePerHourYen,
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
    fee_per_hour_yen_param: FeePerHourYenParam,
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
    if let Some(years_of_service) = career_param.years_of_service {
        let value = convert_years_of_service_into_integer_value(years_of_service.as_str())?;
        let years_of_service_criteria = create_years_of_service_criteria(value);
        params.push(years_of_service_criteria);
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
    if let Some(equal_or_more) = fee_per_hour_yen_param.equal_or_more {
        let equal_or_more_criteria = create_fee_per_hour_yen_equal_or_more_criteria(equal_or_more);
        params.push(equal_or_more_criteria);
    }
    if let Some(equal_or_less) = fee_per_hour_yen_param.equal_or_less {
        let equal_or_less_criteria = create_fee_per_hour_yen_equal_or_less_criteria(equal_or_less);
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

fn convert_years_of_service_into_integer_value(years_of_service: &str) -> Result<u32, ErrResp> {
    if years_of_service == YEARS_OF_SERVICE_THREE_YEARS_OR_MORE {
        Ok(3)
    } else if years_of_service == YEARS_OF_SERVICE_FIVE_YEARS_OR_MORE {
        Ok(5)
    } else if years_of_service == YEARS_OF_SERVICE_TEN_YEARS_OR_MORE {
        Ok(10)
    } else if years_of_service == YEARS_OF_SERVICE_FIFTEEN_YEARS_OR_MORE {
        Ok(15)
    } else if years_of_service == YEARS_OF_SERVICE_TWENTY_YEARS_OR_MORE {
        Ok(20)
    } else {
        // 事前にvalidationしているため、ここを通る場合は障害を意味する
        // そのため、500系のステータスコードでレスポンスを返す
        error!("unexpected years_of_service: ({})", years_of_service);
        Err(unexpected_err_resp())
    }
}

fn create_years_of_service_criteria(years_of_service: u32) -> Value {
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

fn create_fee_per_hour_yen_equal_or_more_criteria(equal_or_more: i32) -> Value {
    json!({
        "range": {
            "fee_per_hour_in_yen": {
                "gte": equal_or_more
            }
        }
    })
}

fn create_fee_per_hour_yen_equal_or_less_criteria(equal_or_less: i32) -> Value {
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
                "must": params
            },
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
    info!("took {} milliseconds", took);

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
        account_id,
        fee_per_hour_in_yen: fee_per_hour_in_yen as i32,
        rating,
        num_of_rated: num_of_rated as i32,
        careers: consultant_career_descriptions,
    })
}

fn create_consultant_career_description(
    career: &Value,
) -> Result<ConsultantCareerDescription, ErrResp> {
    let company_name = career["company_name"].as_str().ok_or_else(|| {
        error!("failed to find company_name id in career: {:?}", career);
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
mod tests {}
