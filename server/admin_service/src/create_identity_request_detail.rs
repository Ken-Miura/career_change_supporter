// Copyright 2021 Ken Miura

use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    Json,
};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::util::session::Admin;

// リクエスト
// クエリでuser account idを受け取る
// レスポンス
// 本人確認依頼（新規）の詳細
// （生年月日が同じユーザーのリストは別途リクエストを発行する）
pub(crate) async fn get_create_identity_request_detail(
    Admin { account_id: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<CreateIdentityReqDetailQuery>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<CreateIdentityReqDetail> {
    Ok((StatusCode::OK, Json(CreateIdentityReqDetail {})))
}

#[derive(Deserialize)]
pub(crate) struct CreateIdentityReqDetailQuery {
    pub(crate) user_account_id: i64,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateIdentityReqDetail {}

#[cfg(test)]
mod tests {}
