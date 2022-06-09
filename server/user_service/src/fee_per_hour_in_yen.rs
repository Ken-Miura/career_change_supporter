// Copyright 2022 Ken Miura

use axum::{Extension, Json};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::util::session::User;

pub(crate) async fn post_fee_per_hour_in_yen(
    User { account_id }: User,
    Json(temp_account): Json<Fee>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<FeePerHourInYenResult> {
    // Identityチェック
    // 最低金額チェック
    todo!()
}

#[derive(Deserialize)]
pub(crate) struct Fee {
    #[serde(rename = "fee-per-hour-in-yen")]
    fee_per_hour_in_yen: i32,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct FeePerHourInYenResult {}
