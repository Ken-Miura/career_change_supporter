// Copyright 2021 Ken Miura

use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset};
use common::{ErrResp, RespResult};

use axum::extract::{Extension, Query};
use entity::{
    create_identity_info_req,
    sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder},
};
use hyper::StatusCode;
use serde::Serialize;

use crate::{
    err::unexpected_err_resp,
    util::{session::Admin, validate_page_size, Pagination},
};

pub(crate) async fn get_create_identity_requests(
    Admin { account_id: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    pagination: Query<Pagination>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<Vec<CreateIdentityReqItem>> {
    let pagination = pagination.0;
    let _ = validate_page_size(pagination.per_page)?;
    let op = CreateIdentityRequestItemsOperationImpl { pool };
    get_create_identity_request_items(pagination, op).await
}

#[derive(Serialize)]
pub(crate) struct CreateIdentityReqItem {
    account_id: i64,
    reqested_at: DateTime<FixedOffset>,
    name: String,
}

async fn get_create_identity_request_items(
    pagination: Pagination,
    op: impl CreateIdentityRequestItemsOperation,
) -> RespResult<Vec<CreateIdentityReqItem>> {
    let items = op.get_items(pagination.page, pagination.per_page).await?;
    Ok((StatusCode::OK, Json(items)))
}

#[async_trait]
trait CreateIdentityRequestItemsOperation {
    async fn get_items(
        &self,
        page: usize,
        page_size: usize,
    ) -> Result<Vec<CreateIdentityReqItem>, ErrResp>;
}

struct CreateIdentityRequestItemsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl CreateIdentityRequestItemsOperation for CreateIdentityRequestItemsOperationImpl {
    async fn get_items(
        &self,
        page: usize,
        page_size: usize,
    ) -> Result<Vec<CreateIdentityReqItem>, ErrResp> {
        let items = create_identity_info_req::Entity::find()
            .order_by_asc(create_identity_info_req::Column::RequestedAt)
            .paginate(&self.pool, page_size)
            .fetch_page(page).await.map_err(|e| {
                tracing::error!("failed to fetch page (page: {}, page_size: {}) in create_identity_info_req: {}", page, page_size, e);
                unexpected_err_resp()
            })?;
        Ok(items
            .iter()
            .map(|model| CreateIdentityReqItem {
                account_id: model.user_account_id,
                reqested_at: model.requested_at,
                name: model.last_name.clone() + " " + model.first_name.as_str(),
            })
            .collect::<Vec<CreateIdentityReqItem>>())
    }
}
