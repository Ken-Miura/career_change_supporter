// Copyright 2022 Ken Miura

use async_session::serde_json::Value;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use axum::{extract::Query, Extension};
use common::{ApiError, ErrResp, RespResult};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use opensearch::OpenSearch;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
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
    pub consultant_id: i64,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ConsultantDetail {
    pub consultant_id: i64,
    pub fee_per_hour_in_yen: i32,
    pub rating: Option<f64>,
    pub num_of_rated: i32,
    pub careers: Vec<ConsultantCareerDetail>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultantCareerDetail {
    pub career_id: i64,
    pub company_name: String,
    pub department_name: Option<String>,
    pub office: Option<String>,
    pub years_of_service: String,
    pub employed: bool,
    pub contract_type: String,
    pub profession: Option<String>,
    pub annual_income_in_man_yen: Option<i32>,
    pub is_manager: bool,
    pub position_name: Option<String>,
    pub is_new_graduate: bool,
    pub note: Option<String>,
}

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
        let model = entity::prelude::Identity::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find identity (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.is_some())
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
        error!(
            "consultant_id is not positive (consultant id: {})",
            consultant_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonPositiveConsultantId as u32,
            }),
        ));
    }
    let identity_exists = op.check_if_identity_exists(account_id).await?;
    if !identity_exists {
        error!("identity is not registered (account id: {})", account_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoIdentityRegistered as u32,
            }),
        ));
    }
    // UserAccountの存在のチェック
    // Detail取得
    todo!()
}
