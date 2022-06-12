// Copyright 2022 Ken Miura

use axum::{Extension, Json};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::util::{session::User, BankAccount};

pub(crate) async fn post_bank_account(
    User { account_id }: User,
    Json(bank_account): Json<BankAccount>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<BankAccountResult> {
    todo!()
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct BankAccountResult {}
