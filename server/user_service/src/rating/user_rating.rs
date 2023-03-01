// Copyright 2023 Ken Miura

use axum::{extract::State, Json};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::util::session::User;

pub(crate) async fn post_user_rating(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
    Json(req): Json<UserRatingParam>,
) -> RespResult<UserRatingResult> {
    todo!()
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct UserRatingParam {
    user_rating_id: i64,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct UserRatingResult {}
