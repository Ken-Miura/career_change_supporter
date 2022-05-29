// Copyright 2022 Ken Miura

use axum::http::StatusCode;
use axum::{async_trait, Json};
use axum::{extract::Query, Extension};
use chrono::Datelike;
use common::util::Ymd;
use common::{util::Career, RespResult};
use common::{ApiError, ErrResp};
use entity::career;
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::util::session::User;

pub(crate) async fn career(
    User { account_id }: User,
    param: Query<GetCareerQueryParam>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<Career> {
    let param = param.0;
    let op = GetCareerOperationImpl::new(pool);
    handle_career_req(account_id, param.career_id, op).await
}

#[derive(Deserialize)]
pub(crate) struct GetCareerQueryParam {
    pub(crate) career_id: i64,
}

async fn handle_career_req(
    account_id: i64,
    career_id: i64,
    op: impl GetCareerOperation,
) -> RespResult<Career> {
    // 任意の職務経歴の取得を防ぐため、必ずログインユーザーのアカウントIDでフィルターをかけた結果を利用
    let careers = op.filter_careers_by_account_id(account_id).await?;
    for career in careers {
        if career.0 == career_id {
            return Ok((StatusCode::OK, Json(career.1)));
        }
    }
    error!(
        "No career associated with user account found (account_id: {}, career_id: {})",
        account_id, career_id
    );
    Err((
        StatusCode::BAD_REQUEST,
        Json(ApiError {
            code: Code::NoCareerToHandleFound as u32,
        }),
    ))
}

#[async_trait]
trait GetCareerOperation {
    async fn filter_careers_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Vec<(i64, Career)>, ErrResp>;
}

struct GetCareerOperationImpl {
    pool: DatabaseConnection,
}

impl GetCareerOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GetCareerOperation for GetCareerOperationImpl {
    async fn filter_careers_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Vec<(i64, Career)>, ErrResp> {
        let models = career::Entity::find()
            .filter(career::Column::UserAccountId.eq(account_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find career (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| {
                (
                    m.career_id,
                    Career {
                        company_name: m.company_name,
                        department_name: m.department_name,
                        office: m.office,
                        career_start_date: Ymd {
                            year: m.career_start_date.year(),
                            month: m.career_start_date.month(),
                            day: m.career_start_date.day(),
                        },
                        career_end_date: m.career_end_date.map(|date| Ymd {
                            year: date.year(),
                            month: date.month(),
                            day: date.day(),
                        }),
                        contract_type: m.contract_type,
                        profession: m.profession,
                        annual_income_in_man_yen: m.annual_income_in_man_yen,
                        is_manager: m.is_manager,
                        position_name: m.position_name,
                        is_new_graduate: m.is_new_graduate,
                        note: m.note,
                    },
                )
            })
            .collect::<Vec<(i64, Career)>>())
    }
}
