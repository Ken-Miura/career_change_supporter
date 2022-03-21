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

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateIdentityReqItem {
    account_id: i64,
    requested_at: DateTime<FixedOffset>,
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
                requested_at: model.requested_at,
                name: model.last_name.clone() + " " + model.first_name.as_str(),
            })
            .collect::<Vec<CreateIdentityReqItem>>())
    }
}

// ロジックはDBへのクエリのみでテストは必要ないかもしれないが、
// DBへのクエリ（ORMのAPI）への期待する動作を記すためにテストを記載しておく。
#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::{Duration, TimeZone, Utc};
    use common::{ErrResp, JAPANESE_TIME_ZONE};

    use crate::util::Pagination;

    use super::{
        get_create_identity_request_items, CreateIdentityReqItem,
        CreateIdentityRequestItemsOperation,
    };

    struct CreateIdentityRequestItemsOperationMock {
        items: Vec<CreateIdentityReqItem>,
    }

    #[async_trait]
    impl CreateIdentityRequestItemsOperation for CreateIdentityRequestItemsOperationMock {
        async fn get_items(
            &self,
            page: usize,
            page_size: usize,
        ) -> Result<Vec<CreateIdentityReqItem>, ErrResp> {
            let items = self.items.clone();
            let length = items.len();
            let start = page * page_size;
            if start >= length {
                return Ok(vec![]);
            }
            let end = if start + page_size > length {
                length
            } else {
                start + page_size
            };
            let items = items.get(start..end).expect("failed to get value");
            Ok(items.to_vec())
        }
    }

    #[tokio::test]
    async fn get_create_identity_request_items_success1() {
        let items = create_dummy_items();
        let op_mock = CreateIdentityRequestItemsOperationMock {
            items: items.clone(),
        };
        let pagination = Pagination {
            page: 0,
            per_page: items.len(),
        };

        let result = get_create_identity_request_items(pagination, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(items, resp.1 .0);
    }

    #[tokio::test]
    async fn get_create_identity_request_items_success2() {
        let items = create_dummy_items();
        let op_mock = CreateIdentityRequestItemsOperationMock {
            items: items.clone(),
        };
        let pagination = Pagination {
            page: 0,
            per_page: items.len() - 1,
        };

        let result = get_create_identity_request_items(pagination, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(items.get(0..2).expect("failed to get value"), resp.1 .0);
    }

    #[tokio::test]
    async fn get_create_identity_request_items_success3() {
        let items = create_dummy_items();
        let op_mock = CreateIdentityRequestItemsOperationMock {
            items: items.clone(),
        };
        let pagination = Pagination {
            page: 1,
            per_page: items.len() - 1,
        };

        let result = get_create_identity_request_items(pagination, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        let item = items.get(2).expect("failed to get value");
        assert_eq!(vec![item.clone()], resp.1 .0);
    }

    #[tokio::test]
    async fn get_create_identity_request_items_success4() {
        let items = create_dummy_items();
        let op_mock = CreateIdentityRequestItemsOperationMock {
            items: items.clone(),
        };
        let pagination = Pagination {
            page: 2,
            per_page: items.len() - 1,
        };

        let result = get_create_identity_request_items(pagination, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Vec::<CreateIdentityReqItem>::with_capacity(0), resp.1 .0);
    }

    fn create_dummy_items() -> Vec<CreateIdentityReqItem> {
        let mut items = Vec::with_capacity(3);
        let requested_at_1 = Utc
            .ymd(2021, 9, 11)
            .and_hms(15, 30, 45)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned());
        let item1 = CreateIdentityReqItem {
            account_id: 1,
            requested_at: requested_at_1,
            name: String::from("山田 太郎"),
        };
        items.push(item1);
        let requested_at_2 = requested_at_1 + Duration::days(1);
        let item2 = CreateIdentityReqItem {
            account_id: 2,
            requested_at: requested_at_2,
            name: String::from("佐藤 次郎"),
        };
        items.push(item2);
        let requested_at_3 = requested_at_2 + Duration::days(1);
        let item3 = CreateIdentityReqItem {
            account_id: 3,
            requested_at: requested_at_3,
            name: String::from("田中 三郎"),
        };
        items.push(item3);
        items
    }
}
