// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;

use super::super::super::admin::Admin;
use super::super::{validate_account_id_is_positive, UserAccountIdQuery};

pub(crate) async fn get_identity_creation_rejection_records(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<UserAccountIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<RejectionRecordResult> {
    let query = query.0;
    let op = RejectionRecordsOperationImpl { pool };
    get_identity_creation_rejection_records_internal(query.user_account_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct RejectionRecordResult {
    rejection_records: Vec<RejectionRecord>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct RejectionRecord {
    rjd_cre_identity_id: i64,
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
    reason: String,
    image1_file_name_without_ext: Option<String>,
    image2_file_name_without_ext: Option<String>,
    rejected_at: String, // RFC 3339形式の文字列
    rejected_by: String,
}

async fn get_identity_creation_rejection_records_internal(
    user_account_id: i64,
    op: impl RejectionRecordsOperation,
) -> RespResult<RejectionRecordResult> {
    validate_account_id_is_positive(user_account_id)?;
    let rejection_records = op
        .get_identity_creation_rejection_records(user_account_id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(RejectionRecordResult { rejection_records }),
    ))
}

#[async_trait]
trait RejectionRecordsOperation {
    async fn get_identity_creation_rejection_records(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<RejectionRecord>, ErrResp>;
}

struct RejectionRecordsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl RejectionRecordsOperation for RejectionRecordsOperationImpl {
    async fn get_identity_creation_rejection_records(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<RejectionRecord>, ErrResp> {
        let models = entity::rejected_create_identity_req::Entity::find()
            .filter(entity::rejected_create_identity_req::Column::UserAccountId.eq(user_account_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter rejected_create_identity_req (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| RejectionRecord {
                rjd_cre_identity_id: m.rjd_cre_identity_id,
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
                reason: m.reason,
                image1_file_name_without_ext: m.image1_file_name_without_ext,
                image2_file_name_without_ext: m.image2_file_name_without_ext,
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
        async fn get_identity_creation_rejection_records(
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
            rjd_cre_identity_id: 1,
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
            reason: "理由１".to_string(),
            image1_file_name_without_ext: Some("6faa6b9468cc4e8fadde117b0fa4d690".to_string()),
            image2_file_name_without_ext: Some("5ef5bc0add3545898c51f3b9b8554b43".to_string()),
            rejected_at: "2023-04-13T14:12:53.4242+09:00 ".to_string(),
            rejected_by: "admin@test.com".to_string(),
        }
    }

    fn create_dummy_rejection_record2(user_account_id: i64) -> RejectionRecord {
        RejectionRecord {
            rjd_cre_identity_id: 2,
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
            reason: "理由２".to_string(),
            image1_file_name_without_ext: Some("7faa6b9468cc4e8fadde117b0fa4d690".to_string()),
            image2_file_name_without_ext: Some("3ef5bc0add3545898c51f3b9b8554b43".to_string()),
            rejected_at: "2023-04-23T14:12:53.4242+09:00 ".to_string(),
            rejected_by: "admin@test.com".to_string(),
        }
    }

    #[tokio::test]

    async fn get_identity_creation_rejection_records_internal_success_1_result() {
        let user_account_id = 64431;
        let rejection_records = vec![create_dummy_rejection_record1(user_account_id)];
        let op_mock = RejectionRecordsOperationMock {
            user_account_id,
            rejection_records: rejection_records.clone(),
        };

        let result =
            get_identity_creation_rejection_records_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(rejection_records, resp.1 .0.rejection_records);
    }

    #[tokio::test]

    async fn get_identity_creation_rejection_records_internal_success_2_results() {
        let user_account_id = 64431;
        let rejection_records = vec![
            create_dummy_rejection_record1(user_account_id),
            create_dummy_rejection_record2(user_account_id),
        ];
        let op_mock = RejectionRecordsOperationMock {
            user_account_id,
            rejection_records: rejection_records.clone(),
        };

        let result =
            get_identity_creation_rejection_records_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(rejection_records, resp.1 .0.rejection_records);
    }

    #[tokio::test]

    async fn get_identity_creation_rejection_records_internal_success_no_result() {
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

        let result = get_identity_creation_rejection_records_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            Vec::<RejectionRecord>::with_capacity(0),
            resp.1 .0.rejection_records
        );
    }

    #[tokio::test]
    async fn get_identity_creation_rejection_records_internal_fail_user_account_id_is_zero() {
        let user_account_id = 0;
        let rejection_records = vec![create_dummy_rejection_record1(user_account_id)];
        let op_mock = RejectionRecordsOperationMock {
            user_account_id,
            rejection_records,
        };

        let result =
            get_identity_creation_rejection_records_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_identity_creation_rejection_records_internal_fail_user_account_id_is_negative() {
        let user_account_id = -1;
        let rejection_records = vec![create_dummy_rejection_record1(user_account_id)];
        let op_mock = RejectionRecordsOperationMock {
            user_account_id,
            rejection_records,
        };

        let result =
            get_identity_creation_rejection_records_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }
}
