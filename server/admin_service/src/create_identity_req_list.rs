// Copyright 2021 Ken Miura

use chrono::{DateTime, Utc};
use common::RespResult;

use axum::extract::Query;
use serde::Serialize;

use crate::util::{session::Admin, validate_page_size, Pagination};

pub(crate) async fn get_create_identity_requests(
    Admin { account_id: _ }: Admin, // 認証されていることを保証するために必須
    pagination: Query<Pagination>,
) -> RespResult<Vec<CreateIdentityReqItem>> {
    let pagination = pagination.0;
    let _ = validate_page_size(pagination.per_page)?;
    tracing::debug!("{}, {}", pagination.page, pagination.per_page);
    todo!()
}

#[derive(Serialize)]
pub(crate) struct CreateIdentityReqItem {
    id: i32,
    reqested_at: DateTime<Utc>,
    name: String,
}
