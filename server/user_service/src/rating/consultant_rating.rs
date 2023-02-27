// Copyright 2023 Ken Miura

use axum::{extract::State, Json};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use opensearch::OpenSearch;
use serde::{Deserialize, Serialize};

use crate::util::session::User;

pub(crate) async fn post_consultant_rating(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
    Json(req): Json<ConsultantRatingParam>,
) -> RespResult<ConsultantRatingResult> {
    todo!()
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ConsultantRatingParam {}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct ConsultantRatingResult {}
