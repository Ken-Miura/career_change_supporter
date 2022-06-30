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
    // pub company_name: Option<String>,
// pub department_name: Option<String>,
// pub office: Option<String>,
// pub years_of_service: Option<String>,
// pub employed: Option<bool>,
// pub contract_type: Option<String>,
// pub profession: Option<String>,
// pub annual_income_in_man_yen: Option<i32>,
// pub is_manager: Option<bool>,
// pub position_name: Option<String>,
// pub is_new_graduate: Option<bool>,
// pub note: Option<String>,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct ConsultantsSearchResult {}
