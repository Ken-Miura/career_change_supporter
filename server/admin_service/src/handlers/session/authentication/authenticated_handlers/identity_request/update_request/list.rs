// Copyright 2021 Ken Miura

use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset};
use common::{ErrResp, RespResult};

use axum::extract::{Query, State};
use axum::http::StatusCode;
use entity::sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder};
use entity::update_identity_req;
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;
use crate::handlers::session::authentication::authenticated_handlers::admin::Admin;
use crate::handlers::session::authentication::authenticated_handlers::pagination::{
    validate_page_size, Pagination,
};

pub(crate) async fn get_update_identity_requests(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    pagination: Query<Pagination>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<Vec<UpdateIdentityReqItem>> {
    let pagination = pagination.0;
    validate_page_size(pagination.per_page)?;
    let op = UpdateIdentityRequestItemsOperationImpl { pool };
    get_update_identity_request_items(pagination, op).await
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct UpdateIdentityReqItem {
    user_account_id: i64,
    requested_at: DateTime<FixedOffset>,
    name: String,
}

async fn get_update_identity_request_items(
    pagination: Pagination,
    op: impl UpdateIdentityRequestItemsOperation,
) -> RespResult<Vec<UpdateIdentityReqItem>> {
    let items = op.get_items(pagination.page, pagination.per_page).await?;
    Ok((StatusCode::OK, Json(items)))
}

#[async_trait]
trait UpdateIdentityRequestItemsOperation {
    async fn get_items(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<UpdateIdentityReqItem>, ErrResp>;
}

struct UpdateIdentityRequestItemsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl UpdateIdentityRequestItemsOperation for UpdateIdentityRequestItemsOperationImpl {
    async fn get_items(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<UpdateIdentityReqItem>, ErrResp> {
        let items = update_identity_req::Entity::find()
            .order_by_asc(update_identity_req::Column::RequestedAt)
            .paginate(&self.pool, page_size)
            .fetch_page(page)
            .await
            .map_err(|e| {
                error!(
                    "failed to fetch page (page: {}, page_size: {}) in update_identity_req: {}",
                    page, page_size, e
                );
                unexpected_err_resp()
            })?;
        Ok(items
            .iter()
            .map(|model| UpdateIdentityReqItem {
                user_account_id: model.user_account_id,
                requested_at: model.requested_at,
                name: model.last_name.clone() + " " + model.first_name.as_str(),
            })
            .collect::<Vec<UpdateIdentityReqItem>>())
    }
}

// ロジックはDBへのクエリのみでテストは必要ないかもしれないが、
// DBへのクエリ（ORMのAPI）への期待する動作を記すためにテストを記載しておく。
#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::{Duration, TimeZone};
    use common::{ErrResp, JAPANESE_TIME_ZONE};

    use crate::handlers::session::authentication::authenticated_handlers::pagination::Pagination;

    use super::*;

    struct UpdateIdentityRequestItemsOperationMock {
        items: Vec<UpdateIdentityReqItem>,
    }

    #[async_trait]
    impl UpdateIdentityRequestItemsOperation for UpdateIdentityRequestItemsOperationMock {
        async fn get_items(
            &self,
            page: u64,
            page_size: u64,
        ) -> Result<Vec<UpdateIdentityReqItem>, ErrResp> {
            let items = self.items.clone();
            let length = items.len() as u64;
            let start = page * page_size;
            if start >= length {
                return Ok(vec![]);
            }
            let end = if start + page_size > length {
                length
            } else {
                start + page_size
            };
            let items = items
                .get(
                    start.try_into().expect("failed to get Ok")
                        ..end.try_into().expect("failed to get Ok"),
                )
                .expect("failed to get value");
            Ok(items.to_vec())
        }
    }

    #[tokio::test]
    async fn get_update_identity_request_items_success1() {
        let items = create_3_dummy_items();
        let op_mock = UpdateIdentityRequestItemsOperationMock {
            items: items.clone(),
        };
        let pagination = Pagination {
            page: 0,
            per_page: 3,
        };

        let result = get_update_identity_request_items(pagination, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(items, resp.1 .0);
    }

    #[tokio::test]
    async fn get_update_identity_request_items_success2() {
        let items = create_3_dummy_items();
        let op_mock = UpdateIdentityRequestItemsOperationMock {
            items: items.clone(),
        };
        let pagination = Pagination {
            page: 0,
            per_page: 2,
        };

        let result = get_update_identity_request_items(pagination, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(items.get(0..2).expect("failed to get value"), resp.1 .0);
    }

    #[tokio::test]
    async fn get_update_identity_request_items_success3() {
        let items = create_3_dummy_items();
        let op_mock = UpdateIdentityRequestItemsOperationMock {
            items: items.clone(),
        };
        let pagination = Pagination {
            page: 1,
            per_page: 2,
        };

        let result = get_update_identity_request_items(pagination, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        let item = items.get(2).expect("failed to get value");
        assert_eq!(vec![item.clone()], resp.1 .0);
    }

    #[tokio::test]
    async fn get_update_identity_request_items_success4() {
        let items = create_3_dummy_items();
        let op_mock = UpdateIdentityRequestItemsOperationMock {
            items: items.clone(),
        };
        let pagination = Pagination {
            page: 2,
            per_page: 2,
        };

        let result = get_update_identity_request_items(pagination, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Vec::<UpdateIdentityReqItem>::with_capacity(0), resp.1 .0);
    }

    fn create_3_dummy_items() -> Vec<UpdateIdentityReqItem> {
        let mut items = Vec::with_capacity(3);
        let requested_at_1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 9, 11, 15, 30, 45)
            .unwrap();
        let item1 = UpdateIdentityReqItem {
            user_account_id: 1,
            requested_at: requested_at_1,
            name: String::from("山田 太郎"),
        };
        items.push(item1);
        let requested_at_2 = requested_at_1 + Duration::days(1);
        let item2 = UpdateIdentityReqItem {
            user_account_id: 2,
            requested_at: requested_at_2,
            name: String::from("佐藤 次郎"),
        };
        items.push(item2);
        let requested_at_3 = requested_at_2 + Duration::days(1);
        let item3 = UpdateIdentityReqItem {
            user_account_id: 3,
            requested_at: requested_at_3,
            name: String::from("田中 三郎"),
        };
        items.push(item3);
        items
    }
}
