// Copyright 2021 Ken Miura

use axum::extract::{Extension, Query};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::Datelike;
use common::util::{Identity, Ymd};
use common::{ApiError, ErrResp, RespResult};
use entity::identity;
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::Deserialize;

use crate::err::{unexpected_err_resp, Code};
use crate::util::session::Admin;

pub(crate) async fn get_identity_by_user_account_id(
    Admin { account_id: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<GetIdentityQuery>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<Identity> {
    let query = query.0;
    let op = IdentityOperationImpl { pool };
    get_identity_by_user_account_id_internal(query.user_account_id, op).await
}

#[derive(Deserialize)]
pub(crate) struct GetIdentityQuery {
    pub(crate) user_account_id: i64,
}

async fn get_identity_by_user_account_id_internal(
    user_account_id: i64,
    op: impl IdentityOperation,
) -> RespResult<Identity> {
    let identity_option = op.get_identity_by_user_account_id(user_account_id).await?;
    let identity = identity_option.ok_or_else(|| {
        tracing::error!("no identity (user account id: {}) found", user_account_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoIdentityFound as u32,
            }),
        )
    })?;
    Ok((StatusCode::OK, Json(identity)))
}

#[async_trait]
trait IdentityOperation {
    async fn get_identity_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<Identity>, ErrResp>;
}

struct IdentityOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl IdentityOperation for IdentityOperationImpl {
    async fn get_identity_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<Identity>, ErrResp> {
        let model = entity::prelude::Identity::find_by_id(user_account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "failed to find Indentity (account id: {}): {}",
                    user_account_id,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(IdentityOperationImpl::convert_identity_model_to_identity))
    }
}

impl IdentityOperationImpl {
    fn convert_identity_model_to_identity(identity_model: identity::Model) -> Identity {
        let date = identity_model.date_of_birth;
        let ymd = Ymd {
            year: date.year(),
            month: date.month(),
            day: date.day(),
        };
        Identity {
            last_name: identity_model.last_name,
            first_name: identity_model.first_name,
            last_name_furigana: identity_model.last_name_furigana,
            first_name_furigana: identity_model.first_name_furigana,
            date_of_birth: ymd,
            prefecture: identity_model.prefecture,
            city: identity_model.city,
            address_line1: identity_model.address_line1,
            address_line2: identity_model.address_line2,
            telephone_number: identity_model.telephone_number,
        }
    }
}

#[cfg(test)]
mod tests {}
