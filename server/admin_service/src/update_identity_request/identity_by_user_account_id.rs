// Copyright 2021 Ken Miura

use axum::extract::{Extension, Query};
use common::util::Identity;
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::util::session::Admin;

pub(crate) async fn get_identity_by_user_account_id(
    Admin { account_id: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    _query: Query<GetIdentityQuery>,
    Extension(_pool): Extension<DatabaseConnection>,
) -> RespResult<Identity> {
    todo!()
    // let query = query.0;
    // let op = UsersByDateOfBirthOperationImpl { pool };
    // get_identity_by_user_account_id_internal(query.user_account_id, op).await
}

#[derive(Deserialize)]
pub(crate) struct GetIdentityQuery {
    pub(crate) user_account_id: i64,
}

#[cfg(test)]
mod tests {}
