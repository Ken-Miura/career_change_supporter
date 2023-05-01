// Copyright 2022 Ken Miura

use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset};
use common::{ErrResp, RespResult};

use axum::extract::{Query, State};
use axum::http::StatusCode;
use entity::create_career_req;
use entity::sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;
use crate::handlers::session::authentication::authenticated_handlers::admin::Admin;
use crate::handlers::session::authentication::authenticated_handlers::pagination::{
    validate_page_size, Pagination,
};

pub(crate) async fn get_create_career_requests(
    Admin { account_id: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    pagination: Query<Pagination>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<Vec<CreateCareerReqItem>> {
    let pagination = pagination.0;
    validate_page_size(pagination.per_page)?;
    let op = CreateCareerRequestItemsOperationImpl { pool };
    get_create_career_request_items(pagination, op).await
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateCareerReqItem {
    create_career_req_id: i64,
    company_name: String,
    requested_at: DateTime<FixedOffset>,
}

async fn get_create_career_request_items(
    pagination: Pagination,
    op: impl CreateCareerRequestItemsOperation,
) -> RespResult<Vec<CreateCareerReqItem>> {
    let items = op.get_items(pagination.page, pagination.per_page).await?;
    Ok((StatusCode::OK, Json(items)))
}

#[async_trait]
trait CreateCareerRequestItemsOperation {
    async fn get_items(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<CreateCareerReqItem>, ErrResp>;
}

struct CreateCareerRequestItemsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl CreateCareerRequestItemsOperation for CreateCareerRequestItemsOperationImpl {
    async fn get_items(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<Vec<CreateCareerReqItem>, ErrResp> {
        let items = create_career_req::Entity::find()
            .order_by_asc(create_career_req::Column::RequestedAt)
            .paginate(&self.pool, page_size)
            .fetch_page(page)
            .await
            .map_err(|e| {
                error!(
                    "failed to fetch page (page: {}, page_size: {}) in create_career_req: {}",
                    page, page_size, e
                );
                unexpected_err_resp()
            })?;
        Ok(items
            .iter()
            .map(|model| CreateCareerReqItem {
                create_career_req_id: model.create_career_req_id,
                company_name: model.company_name.to_string(),
                requested_at: model.requested_at,
            })
            .collect::<Vec<CreateCareerReqItem>>())
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

    struct CreateCareerRequestItemsOperationMock {
        items: Vec<CreateCareerReqItem>,
    }

    #[async_trait]
    impl CreateCareerRequestItemsOperation for CreateCareerRequestItemsOperationMock {
        async fn get_items(
            &self,
            page: u64,
            page_size: u64,
        ) -> Result<Vec<CreateCareerReqItem>, ErrResp> {
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
    async fn get_create_career_request_items_success1() {
        let items = create_3_dummy_items();
        let op_mock = CreateCareerRequestItemsOperationMock {
            items: items.clone(),
        };
        let pagination = Pagination {
            page: 0,
            per_page: 3,
        };

        let result = get_create_career_request_items(pagination, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(items, resp.1 .0);
    }

    #[tokio::test]
    async fn get_create_career_request_items_success2() {
        let items = create_3_dummy_items();
        let op_mock = CreateCareerRequestItemsOperationMock {
            items: items.clone(),
        };
        let pagination = Pagination {
            page: 0,
            per_page: 2,
        };

        let result = get_create_career_request_items(pagination, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(items.get(0..2).expect("failed to get value"), resp.1 .0);
    }

    #[tokio::test]
    async fn get_create_career_request_items_success3() {
        let items = create_3_dummy_items();
        let op_mock = CreateCareerRequestItemsOperationMock {
            items: items.clone(),
        };
        let pagination = Pagination {
            page: 1,
            per_page: 2,
        };

        let result = get_create_career_request_items(pagination, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        let item = items.get(2).expect("failed to get value");
        assert_eq!(vec![item.clone()], resp.1 .0);
    }

    #[tokio::test]
    async fn get_create_career_request_items_success4() {
        let items = create_3_dummy_items();
        let op_mock = CreateCareerRequestItemsOperationMock {
            items: items.clone(),
        };
        let pagination = Pagination {
            page: 2,
            per_page: 2,
        };

        let result = get_create_career_request_items(pagination, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Vec::<CreateCareerReqItem>::with_capacity(0), resp.1 .0);
    }

    fn create_3_dummy_items() -> Vec<CreateCareerReqItem> {
        let mut items = Vec::with_capacity(3);
        let requested_at_1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 9, 11, 15, 30, 45)
            .unwrap();
        let item1 = CreateCareerReqItem {
            create_career_req_id: 1,
            company_name: String::from("テスト１株式会社"),
            requested_at: requested_at_1,
        };
        items.push(item1);
        let requested_at_2 = requested_at_1 + Duration::days(1);
        let item2 = CreateCareerReqItem {
            create_career_req_id: 2,
            company_name: String::from("テスト２株式会社"),
            requested_at: requested_at_2,
        };
        items.push(item2);
        let requested_at_3 = requested_at_2 + Duration::days(1);
        let item3 = CreateCareerReqItem {
            create_career_req_id: 3,
            company_name: String::from("テスト３株式会社"),
            requested_at: requested_at_3,
        };
        items.push(item3);
        items
    }
}
