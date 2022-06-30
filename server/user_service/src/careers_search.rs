// Copyright 2022 Ken Miura

use axum::Json;
use common::RespResult;
use serde::{Deserialize, Serialize};

use crate::util::session::User;

pub(crate) async fn post_careers_search(
    User { account_id: _ }: User,
    Json(_req): Json<CareersSearchRequest>,
) -> RespResult<CareersSearchResult> {
    todo!()
}

#[derive(Deserialize)]
pub(crate) struct CareersSearchRequest {}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct CareersSearchResult {}
