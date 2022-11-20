// Copyright 2022 Ken Miura

use axum::Extension;
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::util::session::User;

pub(crate) async fn get_consultation_requests(
    User { account_id }: User,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<ConsultationRequestsResult> {
    todo!()
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationRequestsResult {
    pub(crate) consultation_requests: Vec<ConsultationRequestsDescription>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationRequestsDescription {
    pub(crate) consultation_req_id: i64,
    pub(crate) user_account_id: i64,
}
