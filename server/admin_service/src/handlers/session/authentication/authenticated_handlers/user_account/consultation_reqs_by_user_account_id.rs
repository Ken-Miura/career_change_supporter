// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::error;

use crate::err::unexpected_err_resp;

use super::super::admin::Admin;
use super::{
    validate_account_id_is_positive, ConsultationReq, ConsultationReqsResult, UserAccountIdQuery,
};

pub(crate) async fn get_consultation_reqs_by_user_account_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<UserAccountIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<ConsultationReqsResult> {
    let query = query.0;
    let op = ConsultationReqsOperationImpl { pool };
    get_consultation_reqs_by_user_account_id_internal(query.user_account_id, op).await
}

async fn get_consultation_reqs_by_user_account_id_internal(
    user_account_id: i64,
    op: impl ConsultationReqsOperation,
) -> RespResult<ConsultationReqsResult> {
    validate_account_id_is_positive(user_account_id)?;
    let consultation_reqs = op
        .get_consultation_reqs_by_user_account_id(user_account_id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(ConsultationReqsResult { consultation_reqs }),
    ))
}

#[async_trait]
trait ConsultationReqsOperation {
    async fn get_consultation_reqs_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<ConsultationReq>, ErrResp>;
}

struct ConsultationReqsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationReqsOperation for ConsultationReqsOperationImpl {
    async fn get_consultation_reqs_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<ConsultationReq>, ErrResp> {
        let models = entity::consultation_req::Entity::find()
            .filter(entity::consultation_req::Column::UserAccountId.eq(user_account_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter consultation_req (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| ConsultationReq {
                consultation_req_id: m.consultation_req_id,
                user_account_id: m.user_account_id,
                consultant_id: m.consultant_id,
                first_candidate_date_time: m
                    .first_candidate_date_time
                    .with_timezone(&(*JAPANESE_TIME_ZONE))
                    .to_rfc3339(),
                second_candidate_date_time: m
                    .second_candidate_date_time
                    .with_timezone(&(*JAPANESE_TIME_ZONE))
                    .to_rfc3339(),
                third_candidate_date_time: m
                    .third_candidate_date_time
                    .with_timezone(&(*JAPANESE_TIME_ZONE))
                    .to_rfc3339(),
                latest_candidate_date_time: m
                    .latest_candidate_date_time
                    .with_timezone(&(*JAPANESE_TIME_ZONE))
                    .to_rfc3339(),
                charge_id: m.charge_id,
                fee_per_hour_in_yen: m.fee_per_hour_in_yen,
                platform_fee_rate_in_percentage: m.platform_fee_rate_in_percentage,
                credit_facilities_expired_at: m
                    .credit_facilities_expired_at
                    .with_timezone(&(*JAPANESE_TIME_ZONE))
                    .to_rfc3339(),
            })
            .collect::<Vec<ConsultationReq>>())
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::ErrResp;

    use crate::err::Code;

    use super::*;

    struct ConsultationReqsOperationMock {
        user_account_id: i64,
        consultation_reqs: Vec<ConsultationReq>,
    }

    #[async_trait]
    impl ConsultationReqsOperation for ConsultationReqsOperationMock {
        async fn get_consultation_reqs_by_user_account_id(
            &self,
            user_account_id: i64,
        ) -> Result<Vec<ConsultationReq>, ErrResp> {
            if self.user_account_id != user_account_id {
                return Ok(vec![]);
            }
            Ok(self.consultation_reqs.clone())
        }
    }

    fn create_dummy_consultation_req1(user_account_id: i64) -> ConsultationReq {
        ConsultationReq {
            consultation_req_id: 1,
            user_account_id,
            consultant_id: 510,
            first_candidate_date_time: "2023-04-13T14:00:00.0000+09:00 ".to_string(),
            second_candidate_date_time: "2023-04-14T14:00:00.0000+09:00 ".to_string(),
            third_candidate_date_time: "2023-04-15T14:00:00.0000+09:00 ".to_string(),
            latest_candidate_date_time: "2023-04-15T14:00:00.0000+09:00 ".to_string(),
            charge_id: "ch_6ebea6645ba1bb27307032b23cd5d".to_string(),
            fee_per_hour_in_yen: 4000,
            platform_fee_rate_in_percentage: "30.00".to_string(),
            credit_facilities_expired_at: "2023-06-06T14:50:12.0000+09:00 ".to_string(),
        }
    }

    fn create_dummy_consultation_req2(user_account_id: i64) -> ConsultationReq {
        ConsultationReq {
            consultation_req_id: 2,
            user_account_id,
            consultant_id: 6110,
            first_candidate_date_time: "2023-04-13T16:00:00.0000+09:00 ".to_string(),
            second_candidate_date_time: "2023-04-14T16:00:00.0000+09:00 ".to_string(),
            third_candidate_date_time: "2023-04-15T16:00:00.0000+09:00 ".to_string(),
            latest_candidate_date_time: "2023-04-15T16:00:00.0000+09:00 ".to_string(),
            charge_id: "ch_7ebea6645ba1bb27307032b23cd5d".to_string(),
            fee_per_hour_in_yen: 5000,
            platform_fee_rate_in_percentage: "30.00".to_string(),
            credit_facilities_expired_at: "2023-06-06T14:50:12.0000+09:00 ".to_string(),
        }
    }

    #[tokio::test]

    async fn get_consultation_reqs_by_user_account_id_internal_success_1_result() {
        let user_account_id = 64431;
        let consultation_req1 = create_dummy_consultation_req1(user_account_id);
        let op_mock = ConsultationReqsOperationMock {
            user_account_id,
            consultation_reqs: vec![consultation_req1.clone()],
        };

        let result =
            get_consultation_reqs_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(vec![consultation_req1], resp.1 .0.consultation_reqs);
    }

    #[tokio::test]

    async fn get_consultation_reqs_by_user_account_id_internal_success_2_results() {
        let user_account_id = 64431;
        let consultation_req1 = create_dummy_consultation_req1(user_account_id);
        let consultation_req2 = create_dummy_consultation_req2(user_account_id);
        let op_mock = ConsultationReqsOperationMock {
            user_account_id,
            consultation_reqs: vec![consultation_req1.clone(), consultation_req2.clone()],
        };

        let result =
            get_consultation_reqs_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            vec![consultation_req1, consultation_req2],
            resp.1 .0.consultation_reqs
        );
    }

    #[tokio::test]

    async fn get_consultation_reqs_by_user_account_id_internal_success_no_result() {
        let user_account_id = 64431;
        let op_mock = ConsultationReqsOperationMock {
            user_account_id,
            consultation_reqs: vec![],
        };

        let result =
            get_consultation_reqs_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            Vec::<ConsultationReq>::with_capacity(0),
            resp.1 .0.consultation_reqs
        );
    }

    #[tokio::test]
    async fn get_consultation_reqs_by_user_account_id_internal_fail_user_account_id_is_zero() {
        let user_account_id = 0;
        let op_mock = ConsultationReqsOperationMock {
            user_account_id,
            consultation_reqs: vec![],
        };

        let result =
            get_consultation_reqs_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_consultation_reqs_by_user_account_id_internal_fail_user_account_id_is_negative() {
        let user_account_id = -1;
        let op_mock = ConsultationReqsOperationMock {
            user_account_id,
            consultation_reqs: vec![],
        };

        let result =
            get_consultation_reqs_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }
}
