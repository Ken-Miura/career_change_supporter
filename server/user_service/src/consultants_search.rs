// Copyright 2022 Ken Miura

use axum::Json;
use common::RespResult;
use serde::{Deserialize, Serialize};

use crate::util::session::User;

pub(crate) async fn post_consultants_search(
    User { account_id: _ }: User,
    Json(_req): Json<ConsultantsSearchRequest>,
) -> RespResult<ConsultantsSearchResult> {
    todo!()
}

#[derive(Deserialize)]
pub(crate) struct ConsultantsSearchRequest {
    pub career_param: CareerParam,
    pub fee_per_hour_yen_param: FeePerHourYenParam,
    pub sort: Option<Sort>,
    pub from: usize,
    pub size: usize,
}

#[derive(Deserialize)]
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

#[derive(Deserialize)]
pub(crate) struct AnnualInComeInManYenParam {
    pub equal_or_more: Option<i32>,
    pub equal_or_less: Option<i32>,
}

#[derive(Deserialize)]
pub(crate) struct FeePerHourYenParam {
    pub equal_or_more: Option<i32>,
    pub equal_or_less: Option<i32>,
}

#[derive(Deserialize)]
pub(crate) struct Sort {
    pub key: String,
    pub order: String,
}

#[derive(Serialize, Debug)]
pub(crate) struct ConsultantsSearchResult {
    total: u32,
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
