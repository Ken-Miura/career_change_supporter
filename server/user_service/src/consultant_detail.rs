// Copyright 2022 Ken Miura

use async_session::serde_json::Value;
use axum::async_trait;
use axum::{extract::Query, Extension};
use common::{ErrResp, RespResult};
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
    let op = ConsultantDetailOperationImpl { pool, index_client };
    handle_consultant_detail(account_id, query.consultant_id, op).await
}

#[derive(Deserialize)]
pub(crate) struct ConsultantDetailQuery {
    pub(crate) consultant_id: i64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ConsultantDetail {}

#[async_trait]
trait ConsultantDetailOperation {
    /// Identityが存在するか確認する。存在する場合、trueを返す。そうでない場合、falseを返す。
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
    async fn search_consultant(&self, index_name: &str, query: &Value) -> Result<Value, ErrResp>;
}

struct ConsultantDetailOperationImpl {
    pool: DatabaseConnection,
    index_client: OpenSearch,
}

#[async_trait]
impl ConsultantDetailOperation for ConsultantDetailOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        todo!()
    }

    async fn search_consultant(&self, index_name: &str, query: &Value) -> Result<Value, ErrResp> {
        todo!()
    }
}

async fn handle_consultant_detail(
    account_id: i64,
    consultant_id: i64,
    op: impl ConsultantDetailOperation,
) -> RespResult<ConsultantDetail> {
    if !consultant_id.is_positive() {
        todo!()
    }
    todo!()
}
