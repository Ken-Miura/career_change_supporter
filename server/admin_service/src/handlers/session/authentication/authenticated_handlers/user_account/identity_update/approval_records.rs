// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;

use super::super::super::admin::Admin;
use super::super::{validate_account_id_is_positive, UserAccountIdQuery};

pub(crate) async fn get_identity_update_approval_records(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<UserAccountIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<ApprovalRecordResult> {
    let query = query.0;
    let op = ApprovalRecordsOperationImpl { pool };
    get_identity_update_approval_records_internal(query.user_account_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ApprovalRecordResult {
    approval_records: Vec<ApprovalRecord>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ApprovalRecord {
    appr_upd_identity_req_id: i64,
    user_account_id: i64,
    last_name: String,
    first_name: String,
    last_name_furigana: String,
    first_name_furigana: String,
    date_of_birth: String, // 2023-05-27 のような形式の文字列
    prefecture: String,
    city: String,
    address_line1: String,
    address_line2: Option<String>,
    telephone_number: String,
    image1_file_name_without_ext: String,
    image2_file_name_without_ext: Option<String>,
    approved_at: String, // RFC 3339形式の文字列
    approved_by: String,
}

async fn get_identity_update_approval_records_internal(
    user_account_id: i64,
    op: impl ApprovalRecordsOperation,
) -> RespResult<ApprovalRecordResult> {
    validate_account_id_is_positive(user_account_id)?;
    let approval_records = op
        .get_identity_update_approval_records(user_account_id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(ApprovalRecordResult { approval_records }),
    ))
}

#[async_trait]
trait ApprovalRecordsOperation {
    async fn get_identity_update_approval_records(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<ApprovalRecord>, ErrResp>;
}

struct ApprovalRecordsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ApprovalRecordsOperation for ApprovalRecordsOperationImpl {
    async fn get_identity_update_approval_records(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<ApprovalRecord>, ErrResp> {
        let models = entity::approved_update_identity_req::Entity::find()
            .filter(entity::approved_update_identity_req::Column::UserAccountId.eq(user_account_id))
            .order_by_desc(entity::approved_update_identity_req::Column::ApprovedAt)
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter approved_update_identity_req (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| ApprovalRecord {
                appr_upd_identity_req_id: m.appr_upd_identity_req_id,
                user_account_id: m.user_account_id,
                last_name: m.last_name,
                first_name: m.first_name,
                last_name_furigana: m.last_name_furigana,
                first_name_furigana: m.first_name_furigana,
                date_of_birth: m.date_of_birth.format("%Y-%m-%d").to_string(),
                prefecture: m.prefecture,
                city: m.city,
                address_line1: m.address_line1,
                address_line2: m.address_line2,
                telephone_number: m.telephone_number,
                image1_file_name_without_ext: m.image1_file_name_without_ext,
                image2_file_name_without_ext: m.image2_file_name_without_ext,
                approved_at: m
                    .approved_at
                    .with_timezone(&(*JAPANESE_TIME_ZONE))
                    .to_rfc3339(),
                approved_by: m.approved_by,
            })
            .collect::<Vec<ApprovalRecord>>())
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::ErrResp;

    use crate::err::Code;

    use super::*;

    struct ApprovalRecordsOperationMock {
        user_account_id: i64,
        approval_records: Vec<ApprovalRecord>,
    }

    #[async_trait]
    impl ApprovalRecordsOperation for ApprovalRecordsOperationMock {
        async fn get_identity_update_approval_records(
            &self,
            user_account_id: i64,
        ) -> Result<Vec<ApprovalRecord>, ErrResp> {
            if self.user_account_id != user_account_id {
                return Ok(vec![]);
            }
            Ok(self.approval_records.clone())
        }
    }

    fn create_dummy_approval_record1(user_account_id: i64) -> ApprovalRecord {
        ApprovalRecord {
            appr_upd_identity_req_id: 1,
            user_account_id,
            last_name: "田中".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "タナカ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: "2000-04-01".to_string(),
            prefecture: "東京都".to_string(),
            city: "八王子市".to_string(),
            address_line1: "元本郷町三丁目24番1号".to_string(),
            address_line2: Some("マンション１０１".to_string()),
            telephone_number: "09012345678".to_string(),
            image1_file_name_without_ext: "7b1e7f857ea04162bc36dba07d085c76".to_string(),
            image2_file_name_without_ext: None,
            approved_at: "2023-04-13T14:12:53.4242+09:00 ".to_string(),
            approved_by: "admin@test.com".to_string(),
        }
    }

    fn create_dummy_approval_record2(user_account_id: i64) -> ApprovalRecord {
        ApprovalRecord {
            appr_upd_identity_req_id: 2,
            user_account_id,
            last_name: "田中".to_string(),
            first_name: "太郎".to_string(),
            last_name_furigana: "タナカ".to_string(),
            first_name_furigana: "タロウ".to_string(),
            date_of_birth: "2000-04-01".to_string(),
            prefecture: "東京都".to_string(),
            city: "八王子市".to_string(),
            address_line1: "元本郷町三丁目24番1号".to_string(),
            address_line2: Some("マンション１０１".to_string()),
            telephone_number: "09012345678".to_string(),
            image1_file_name_without_ext: "6b1e7f857ea04162bc36dba07d085c76".to_string(),
            image2_file_name_without_ext: Some("5b1e7f857ea04162bc36dba07d085c76".to_string()),
            approved_at: "2023-04-23T14:12:53.4242+09:00 ".to_string(),
            approved_by: "admin@test.com".to_string(),
        }
    }

    #[tokio::test]

    async fn get_identity_update_approval_records_internal_success_1_result() {
        let user_account_id = 64431;
        let approval_records = vec![create_dummy_approval_record1(user_account_id)];
        let op_mock = ApprovalRecordsOperationMock {
            user_account_id,
            approval_records: approval_records.clone(),
        };

        let result = get_identity_update_approval_records_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(approval_records, resp.1 .0.approval_records);
    }

    #[tokio::test]

    async fn get_identity_update_approval_records_internal_success_2_results() {
        let user_account_id = 64431;
        let approval_records = vec![
            create_dummy_approval_record1(user_account_id),
            create_dummy_approval_record2(user_account_id),
        ];
        let op_mock = ApprovalRecordsOperationMock {
            user_account_id,
            approval_records: approval_records.clone(),
        };

        let result = get_identity_update_approval_records_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(approval_records, resp.1 .0.approval_records);
    }

    #[tokio::test]

    async fn get_identity_update_approval_records_internal_success_no_result() {
        let user_account_id = 64431;
        let approval_records = vec![
            create_dummy_approval_record1(user_account_id),
            create_dummy_approval_record2(user_account_id),
        ];
        let op_mock = ApprovalRecordsOperationMock {
            user_account_id,
            approval_records: approval_records.clone(),
        };
        let dummy_id = user_account_id + 451;

        let result = get_identity_update_approval_records_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            Vec::<ApprovalRecord>::with_capacity(0),
            resp.1 .0.approval_records
        );
    }

    #[tokio::test]
    async fn get_identity_update_approval_records_internal_fail_user_account_id_is_zero() {
        let user_account_id = 0;
        let approval_records = vec![create_dummy_approval_record1(user_account_id)];
        let op_mock = ApprovalRecordsOperationMock {
            user_account_id,
            approval_records,
        };

        let result = get_identity_update_approval_records_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_identity_update_approval_records_internal_fail_user_account_id_is_negative() {
        let user_account_id = -1;
        let approval_records = vec![create_dummy_approval_record1(user_account_id)];
        let op_mock = ApprovalRecordsOperationMock {
            user_account_id,
            approval_records,
        };

        let result = get_identity_update_approval_records_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }
}
