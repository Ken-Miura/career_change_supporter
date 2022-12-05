// Copyright 2022 Ken Miura

use axum::{extract::State, Json};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::util::session::User;

pub(crate) async fn post_consultation_request_rejection(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
    Json(param): Json<ConsultationRequestRejectionParam>,
) -> RespResult<ConsultationRequestRejectionResult> {
    todo!()
}

#[derive(Deserialize)]
pub(crate) struct ConsultationRequestRejectionParam {
    pub(crate) consultation_req_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationRequestRejectionResult {}
