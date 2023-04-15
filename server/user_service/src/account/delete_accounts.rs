// Copyright 2023 Ken Miura

use axum::extract::State;
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use opensearch::OpenSearch;
use serde::Serialize;

use crate::util::session::user::User;

pub(crate) async fn delete_accounts(
    User { user_info }: User,
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
) -> RespResult<DeleteAccountResult> {
    todo!()
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct DeleteAccountResult {}