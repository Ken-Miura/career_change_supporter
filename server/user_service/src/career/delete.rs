// Copyright 2022 Ken Miura

use axum::http::StatusCode;
use axum::{async_trait, Json};
use axum::{extract::Query, Extension};
use common::{ApiError, ErrResp};
use common::{ErrRespStruct, RespResult};
use entity::sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QuerySelect, TransactionTrait,
};
use entity::{career, document};
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
    let _ = op.delete_career(account_id, career_id).await?;
    Ok((StatusCode::OK, Json(DeleteCareerResult {})))
}

#[async_trait]
trait DeleteCareerOperation {
    async fn filter_career_ids_by_account_id(&self, account_id: i64) -> Result<Vec<i64>, ErrResp>;
    async fn delete_career(&self, account_id: i64, career_id: i64) -> Result<(), ErrResp>;
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

    async fn delete_career(&self, account_id: i64, career_id: i64) -> Result<(), ErrResp> {
        let _ = self.pool.transaction::<_, (), ErrRespStruct>(|txn| {
            Box::pin(async move {
                let document_option = document::Entity::find_by_id(account_id)
                    .lock_exclusive()
                    .one(txn)
                    .await
                    .map_err(|e| {
                        error!(
                            "failed to find document (user_account_id: {}): {}",
                            account_id, e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;
                let document = document_option.ok_or_else(|| {
                    error!("no document found (user_account_id: {})", account_id);
                    ErrRespStruct {
                        err_resp: unexpected_err_resp(),
                    }
                })?;

                let _ = career::Entity::delete_by_id(career_id)
                    .exec(txn)
                    .await
                    .map_err(|e| {
                        error!("failed to delete career (career_id: {}): {}", career_id, e);
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                Ok(())
            })
        });
        Ok(())
    }
}
