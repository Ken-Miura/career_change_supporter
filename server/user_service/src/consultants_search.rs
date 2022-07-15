// Copyright 2022 Ken Miura

use axum::{Extension, Json};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::util::{
    session::User,
    validator::consultant_search_param::{
        career_param_validator::validate_career_param,
        fee_per_hour_yen_param_validator::validate_fee_per_hour_yen_param,
        sort_param_validator::validate_sort_param,
    },
};

pub(crate) async fn post_consultants_search(
    User { account_id: _ }: User,
    Json(req): Json<ConsultantSearchParam>,
    Extension(_pool): Extension<DatabaseConnection>,
) -> RespResult<ConsultantsSearchResult> {
    let _ = validate_fee_per_hour_yen_param(&req.fee_per_hour_yen_param).expect("failed to get Ok");
    if let Some(sort_param) = req.sort_param {
        let _ = validate_sort_param(&sort_param).expect("failed to get Ok");
    }
    let _ = validate_career_param(&req.career_param).expect("failed to get Ok");
    todo!()
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
