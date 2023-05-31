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

pub(crate) async fn get_career_creation_approval_records(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<UserAccountIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<ApprovalRecordResult> {
    let query = query.0;
    let op = ApprovalRecordsOperationImpl { pool };
    get_career_creation_approval_records_internal(query.user_account_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ApprovalRecordResult {
    approval_records: Vec<ApprovalRecord>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ApprovalRecord {
    appr_cre_career_req_id: i64,
    user_account_id: i64,
    company_name: String,
    department_name: Option<String>,
    office: Option<String>,
    career_start_date: String,       // 2023-05-27 のような形式の文字列
    career_end_date: Option<String>, // 2023-05-27 のような形式の文字列
    contract_type: String,           // 'regular' or 'contract' or 'other'
    profession: Option<String>,
    annual_income_in_man_yen: Option<i32>,
    is_manager: bool,
    position_name: Option<String>,
    is_new_graduate: bool,
    note: Option<String>,
    image1_file_name_without_ext: String,
    image2_file_name_without_ext: Option<String>,
    approved_at: String, // RFC 3339形式の文字列
    approved_by: String,
}

async fn get_career_creation_approval_records_internal(
    user_account_id: i64,
    op: impl ApprovalRecordsOperation,
) -> RespResult<ApprovalRecordResult> {
    validate_account_id_is_positive(user_account_id)?;
    let approval_records = op
        .get_career_creation_approval_records(user_account_id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(ApprovalRecordResult { approval_records }),
    ))
}

#[async_trait]
trait ApprovalRecordsOperation {
    async fn get_career_creation_approval_records(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<ApprovalRecord>, ErrResp>;
}

struct ApprovalRecordsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ApprovalRecordsOperation for ApprovalRecordsOperationImpl {
    async fn get_career_creation_approval_records(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<ApprovalRecord>, ErrResp> {
        let models = entity::approved_create_career_req::Entity::find()
            .filter(entity::approved_create_career_req::Column::UserAccountId.eq(user_account_id))
            .order_by_desc(entity::approved_create_career_req::Column::ApprovedAt)
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter approved_create_career_req (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| ApprovalRecord {
                appr_cre_career_req_id: m.appr_cre_career_req_id,
                user_account_id: m.user_account_id,
                company_name: m.company_name,
                department_name: m.department_name,
                office: m.office,
                career_start_date: m.career_start_date.format("%Y-%m-%d").to_string(),
                career_end_date: m
                    .career_end_date
                    .map(|dt| dt.format("%Y-%m-%d").to_string()),
                contract_type: m.contract_type,
                profession: m.profession,
                annual_income_in_man_yen: m.annual_income_in_man_yen,
                is_manager: m.is_manager,
                position_name: m.position_name,
                is_new_graduate: m.is_new_graduate,
                note: m.note,
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
        async fn get_career_creation_approval_records(
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
            appr_cre_career_req_id: 1,
            user_account_id,
            company_name: "テスト１株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date: "2016-04-01".to_string(),
            career_end_date: Some("2017-12-01".to_string()),
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
            image1_file_name_without_ext: "7b1e7f857ea04162bc36dba07d085c76".to_string(),
            image2_file_name_without_ext: None,
            approved_at: "2023-04-13T14:12:53.4242+09:00 ".to_string(),
            approved_by: "admin@test.com".to_string(),
        }
    }

    fn create_dummy_approval_record2(user_account_id: i64) -> ApprovalRecord {
        ApprovalRecord {
            appr_cre_career_req_id: 2,
            user_account_id,
            company_name: "テスト２株式会社".to_string(),
            department_name: Some("開発部署".to_string()),
            office: Some("和歌山事業所".to_string()),
            career_start_date: "2018-01-01".to_string(),
            career_end_date: None,
            contract_type: "other".to_string(),
            profession: Some("SE".to_string()),
            annual_income_in_man_yen: Some(600),
            is_manager: true,
            position_name: Some("係長".to_string()),
            is_new_graduate: false,
            note: Some(
                r"理由１
            理由２
            理由３"
                    .to_string(),
            ),
            image1_file_name_without_ext: "5b1e7f857ea04162bc36dba07d085c76".to_string(),
            image2_file_name_without_ext: Some("4b1e7f857ea04162bc36dba07d085c76".to_string()),
            approved_at: "2023-04-15T14:12:53.4242+09:00 ".to_string(),
            approved_by: "admin@test.com".to_string(),
        }
    }

    #[tokio::test]

    async fn get_career_creation_approval_records_internal_success_1_result() {
        let user_account_id = 64431;
        let approval_records = vec![create_dummy_approval_record1(user_account_id)];
        let op_mock = ApprovalRecordsOperationMock {
            user_account_id,
            approval_records: approval_records.clone(),
        };

        let result = get_career_creation_approval_records_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(approval_records, resp.1 .0.approval_records);
    }

    #[tokio::test]

    async fn get_career_creation_approval_records_internal_success_2_results() {
        let user_account_id = 64431;
        let approval_records = vec![
            create_dummy_approval_record1(user_account_id),
            create_dummy_approval_record2(user_account_id),
        ];
        let op_mock = ApprovalRecordsOperationMock {
            user_account_id,
            approval_records: approval_records.clone(),
        };

        let result = get_career_creation_approval_records_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(approval_records, resp.1 .0.approval_records);
    }

    #[tokio::test]

    async fn get_career_creation_approval_records_internal_success_no_result() {
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

        let result = get_career_creation_approval_records_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            Vec::<ApprovalRecord>::with_capacity(0),
            resp.1 .0.approval_records
        );
    }

    #[tokio::test]
    async fn get_career_creation_approval_records_internal_fail_user_account_id_is_zero() {
        let user_account_id = 0;
        let approval_records = vec![create_dummy_approval_record1(user_account_id)];
        let op_mock = ApprovalRecordsOperationMock {
            user_account_id,
            approval_records,
        };

        let result = get_career_creation_approval_records_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_career_creation_approval_records_internal_fail_user_account_id_is_negative() {
        let user_account_id = -1;
        let approval_records = vec![create_dummy_approval_record1(user_account_id)];
        let op_mock = ApprovalRecordsOperationMock {
            user_account_id,
            approval_records,
        };

        let result = get_career_creation_approval_records_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }
}
