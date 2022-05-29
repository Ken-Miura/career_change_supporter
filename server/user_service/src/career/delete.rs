// Copyright 2022 Ken Miura

use axum::http::StatusCode;
use axum::{async_trait, Json};
use axum::{extract::Query, Extension};
use common::RespResult;
use common::{ApiError, ErrResp};
use entity::career;
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::util::session::User;

pub(crate) async fn career(
    User { account_id }: User,
    param: Query<DeleteCareerQueryParam>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<DeleteCareerResult> {
    let param = param.0;
    let op = DeleteCareerOperationImpl::new(pool);
    handle_career_req(account_id, param.career_id, op).await
}

#[derive(Deserialize)]
pub(crate) struct DeleteCareerQueryParam {
    pub(crate) career_id: i64,
}

#[derive(Serialize)]
pub(crate) struct DeleteCareerResult {}

async fn handle_career_req(
    account_id: i64,
    career_id: i64,
    op: impl DeleteCareerOperation,
) -> RespResult<DeleteCareerResult> {
    // 任意の職務経歴の削除を防ぐため、必ずログインユーザーのアカウントIDに紐付いた職務経歴かチェック
    let career_ids = op.filter_career_ids_by_account_id(account_id).await?;
    if !career_ids.contains(&career_id) {
        error!(
            "No career associated with user account found (account_id: {}, career_id: {})",
            account_id, career_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoCareerToHandleFound as u32,
            }),
        ));
    }
    todo!()
}

#[async_trait]
trait DeleteCareerOperation {
    async fn filter_career_ids_by_account_id(&self, account_id: i64) -> Result<Vec<i64>, ErrResp>;
}

struct DeleteCareerOperationImpl {
    pool: DatabaseConnection,
}

impl DeleteCareerOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DeleteCareerOperation for DeleteCareerOperationImpl {
    async fn filter_career_ids_by_account_id(&self, account_id: i64) -> Result<Vec<i64>, ErrResp> {
        let models = career::Entity::find()
            .filter(career::Column::UserAccountId.eq(account_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter career (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| m.career_id)
            .collect::<Vec<i64>>())
    }
}
