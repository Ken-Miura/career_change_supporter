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

pub(crate) async fn get_career_creation_rejection_records(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<UserAccountIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<RejectionRecordResult> {
    let query = query.0;
    let op = RejectionRecordsOperationImpl { pool };
    get_career_creation_rejection_records_internal(query.user_account_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct RejectionRecordResult {
    rejection_records: Vec<RejectionRecord>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RejectionRecord {
    rjd_cre_career_req_id: i64,
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
    reason: String,
    rejected_at: String, // RFC 3339形式の文字列
    rejected_by: String,
}

async fn get_career_creation_rejection_records_internal(
    user_account_id: i64,
    op: impl RejectionRecordsOperation,
) -> RespResult<RejectionRecordResult> {
    validate_account_id_is_positive(user_account_id)?;
    let rejection_records = op
        .get_career_creation_rejection_records(user_account_id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(RejectionRecordResult { rejection_records }),
    ))
}

#[async_trait]
trait RejectionRecordsOperation {
    async fn get_career_creation_rejection_records(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<RejectionRecord>, ErrResp>;
}

struct RejectionRecordsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl RejectionRecordsOperation for RejectionRecordsOperationImpl {
    async fn get_career_creation_rejection_records(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<RejectionRecord>, ErrResp> {
        let models = entity::rejected_create_career_req::Entity::find()
            .filter(entity::rejected_create_career_req::Column::UserAccountId.eq(user_account_id))
            .order_by_desc(entity::rejected_create_career_req::Column::RejectedAt)
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter rejected_create_career_req (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| RejectionRecord {
                rjd_cre_career_req_id: m.rjd_cre_career_req_id,
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
                reason: m.reason,
                rejected_at: m
                    .rejected_at
                    .with_timezone(&(*JAPANESE_TIME_ZONE))
                    .to_rfc3339(),
                rejected_by: m.rejected_by,
            })
            .collect::<Vec<RejectionRecord>>())
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::ErrResp;

    use crate::err::Code;

    use super::*;

    struct RejectionRecordsOperationMock {
        user_account_id: i64,
        rejection_records: Vec<RejectionRecord>,
    }

    #[async_trait]
    impl RejectionRecordsOperation for RejectionRecordsOperationMock {
        async fn get_career_creation_rejection_records(
            &self,
            user_account_id: i64,
        ) -> Result<Vec<RejectionRecord>, ErrResp> {
            if self.user_account_id != user_account_id {
                return Ok(vec![]);
            }
            Ok(self.rejection_records.clone())
        }
    }

    fn create_dummy_rejection_record1(user_account_id: i64) -> RejectionRecord {
        RejectionRecord {
            rjd_cre_career_req_id: 1,
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
            reason: "理由１".to_string(),
            rejected_at: "2023-04-13T14:12:53.4242+09:00 ".to_string(),
            rejected_by: "admin@test.com".to_string(),
        }
    }

    fn create_dummy_rejection_record2(user_account_id: i64) -> RejectionRecord {
        RejectionRecord {
            rjd_cre_career_req_id: 2,
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
            reason: "理由２".to_string(),
            rejected_at: "2023-04-23T14:12:53.4242+09:00 ".to_string(),
            rejected_by: "admin@test.com".to_string(),
        }
    }

    #[tokio::test]

    async fn get_career_creation_rejection_records_internal_success_1_result() {
        let user_account_id = 64431;
        let rejection_records = vec![create_dummy_rejection_record1(user_account_id)];
        let op_mock = RejectionRecordsOperationMock {
            user_account_id,
            rejection_records: rejection_records.clone(),
        };

        let result = get_career_creation_rejection_records_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(rejection_records, resp.1 .0.rejection_records);
    }

    #[tokio::test]

    async fn get_career_creation_rejection_records_internal_success_2_results() {
        let user_account_id = 64431;
        let rejection_records = vec![
            create_dummy_rejection_record1(user_account_id),
            create_dummy_rejection_record2(user_account_id),
        ];
        let op_mock = RejectionRecordsOperationMock {
            user_account_id,
            rejection_records: rejection_records.clone(),
        };

        let result = get_career_creation_rejection_records_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(rejection_records, resp.1 .0.rejection_records);
    }

    #[tokio::test]

    async fn get_career_creation_rejection_records_internal_success_no_result() {
        let user_account_id = 64431;
        let rejection_records = vec![
            create_dummy_rejection_record1(user_account_id),
            create_dummy_rejection_record2(user_account_id),
        ];
        let op_mock = RejectionRecordsOperationMock {
            user_account_id,
            rejection_records: rejection_records.clone(),
        };
        let dummy_id = user_account_id + 451;

        let result = get_career_creation_rejection_records_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            Vec::<RejectionRecord>::with_capacity(0),
            resp.1 .0.rejection_records
        );
    }

    #[tokio::test]
    async fn get_career_creation_rejection_records_internal_fail_user_account_id_is_zero() {
        let user_account_id = 0;
        let rejection_records = vec![create_dummy_rejection_record1(user_account_id)];
        let op_mock = RejectionRecordsOperationMock {
            user_account_id,
            rejection_records,
        };

        let result = get_career_creation_rejection_records_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_career_creation_rejection_records_internal_fail_user_account_id_is_negative() {
        let user_account_id = -1;
        let rejection_records = vec![create_dummy_rejection_record1(user_account_id)];
        let op_mock = RejectionRecordsOperationMock {
            user_account_id,
            rejection_records,
        };

        let result = get_career_creation_rejection_records_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }
}
