// Copyright 2022 Ken Miura

use axum::{extract::Query, Extension};
use common::{util::Career, RespResult};
use entity::sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::util::session::User;

pub(crate) async fn career(
    User { account_id }: User,
    param: Query<GetCareerQueryParam>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<Career> {
    let param = param.0;
    todo!()
}

#[derive(Deserialize)]
pub(crate) struct GetCareerQueryParam {
    pub(crate) career_id: i64,
}
