// Copyright 2022 Ken Miura

use axum::{Extension, Json};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::util::{
    session::User, validator::bank_account_validator::validate_bank_account, BankAccount,
};

pub(crate) async fn post_bank_account(
    User { account_id: _ }: User,
    Json(bank_account): Json<BankAccount>,
    Extension(_pool): Extension<DatabaseConnection>,
) -> RespResult<BankAccountResult> {
    let _ = validate_bank_account(&bank_account).expect("msg");
    todo!()
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct BankAccountResult {}
