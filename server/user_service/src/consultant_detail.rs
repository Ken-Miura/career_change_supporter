// Copyright 2022 Ken Miura

use axum::{extract::Query, Extension};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use opensearch::OpenSearch;
use serde::{Deserialize, Serialize};

use crate::util::session::User;

pub(crate) async fn get_consultant_detail(
    User { account_id }: User,
    query: Query<ConsultantDetailQuery>,
    Extension(pool): Extension<DatabaseConnection>,
    Extension(index_client): Extension<OpenSearch>,
) -> RespResult<ConsultantDetail> {
    let query = query.0;
    if !query.consultant_id.is_positive() {
        todo!()
    }
    todo!()
}

#[derive(Deserialize)]
pub(crate) struct ConsultantDetailQuery {
    pub(crate) consultant_id: i64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ConsultantDetail {}
