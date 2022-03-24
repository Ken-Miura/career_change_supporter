// Copyright 2021 Ken Miura

use axum::async_trait;
use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, FixedOffset, NaiveDate};
use common::{ApiError, ErrResp, RespResult};
use entity::create_identity_info_req;
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};

use crate::err::unexpected_err_resp;
use crate::err::Code::NoCreateIdentityReqDetailFound;
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
    let query = query.0;
    let op = CreateIdentityReqDetailOperationImpl { pool };
    get_create_identity_req_detail(query.user_account_id, op).await
}

#[derive(Deserialize)]
pub(crate) struct CreateIdentityReqDetailQuery {
    pub(crate) user_account_id: i64,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateIdentityReqDetail {
    pub(crate) last_name: String,
    pub(crate) first_name: String,
    pub(crate) last_name_furigana: String,
    pub(crate) first_name_furigana: String,
    pub(crate) date_of_birth: NaiveDate,
    pub(crate) prefecture: String,
    pub(crate) city: String,
    pub(crate) address_line1: String,
    pub(crate) address_line2: Option<String>,
    pub(crate) telephone_number: String,
    pub(crate) image1_file_name_without_ext: String,
    pub(crate) image2_file_name_without_ext: Option<String>,
    pub(crate) requested_at: DateTime<FixedOffset>,
}

async fn get_create_identity_req_detail(
    user_account_id: i64,
    op: impl CreateIdentityReqDetailOperation,
) -> RespResult<CreateIdentityReqDetail> {
    let req_detail_option = op.get_create_identity_req_detail(user_account_id).await?;
    let req_detail = req_detail_option.ok_or_else(|| {
        tracing::error!(
            "no create identity request (user account id: {}) found",
            user_account_id
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: NoCreateIdentityReqDetailFound as u32,
            }),
        )
    })?;
    Ok((StatusCode::OK, Json(req_detail)))
}

#[async_trait]
trait CreateIdentityReqDetailOperation {
    async fn get_create_identity_req_detail(
        &self,
        user_account_id: i64,
    ) -> Result<Option<CreateIdentityReqDetail>, ErrResp>;
}

struct CreateIdentityReqDetailOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl CreateIdentityReqDetailOperation for CreateIdentityReqDetailOperationImpl {
    async fn get_create_identity_req_detail(
        &self,
        user_account_id: i64,
    ) -> Result<Option<CreateIdentityReqDetail>, ErrResp> {
        let result = create_identity_info_req::Entity::find_by_id(user_account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!(
                    "failed to find create identity request (user account id: {}): {}",
                    user_account_id,
                    e
                );
                unexpected_err_resp()
            })?;
        Ok(result.map(|m| CreateIdentityReqDetail {
            last_name: m.last_name,
            first_name: m.first_name,
            last_name_furigana: m.last_name_furigana,
            first_name_furigana: m.first_name_furigana,
            date_of_birth: m.date_of_birth,
            prefecture: m.prefecture,
            city: m.city,
            address_line1: m.address_line1,
            address_line2: m.address_line2,
            telephone_number: m.telephone_number,
            image1_file_name_without_ext: m.image1_file_name_without_ext,
            image2_file_name_without_ext: m.image2_file_name_without_ext,
            requested_at: m.requested_at,
        }))
    }
}

#[cfg(test)]
mod tests {}
