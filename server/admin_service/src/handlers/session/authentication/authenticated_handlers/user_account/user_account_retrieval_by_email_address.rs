// Copyright 2023 Ken Miura

use axum::{extract::State, Json};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::Deserialize;

use super::{super::admin::Admin, UserAccountRetrievalResult};

pub(crate) async fn post_user_account_retrieval_by_email_address(
    Admin { admin_info }: Admin,
    State(pool): State<DatabaseConnection>,
    Json(req): Json<UserAccountRetrievalByEmailAddressReq>,
) -> RespResult<UserAccountRetrievalResult> {
    todo!()
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct UserAccountRetrievalByEmailAddressReq {
    email_address: String,
}
