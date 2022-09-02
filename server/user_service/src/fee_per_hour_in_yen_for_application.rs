// Copyright 2022 Ken Miura

use async_redis_session::RedisSessionStore;
use axum::{extract::Query, Extension};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::util::session::User;

pub(crate) async fn get_fee_per_hour_in_yen_for_application(
    User { account_id }: User,
    query: Query<FeePerHourInYenForApplicationQuery>,
    Extension(store): Extension<RedisSessionStore>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<FeePerHourInYenForApplication> {
    todo!()
}

#[derive(Deserialize)]
pub(crate) struct FeePerHourInYenForApplicationQuery {
    pub consultant_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct FeePerHourInYenForApplication {
    pub fee_per_hour_in_yen: i32,
}
