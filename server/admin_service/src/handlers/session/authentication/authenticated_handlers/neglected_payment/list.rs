// Copyright 2023 Ken Miura

use async_session::async_trait;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use common::{ErrResp, RespResult};
use entity::sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder};
use serde::Serialize;
use tracing::error;

use crate::{
    err::unexpected_err_resp,
    handlers::session::authentication::authenticated_handlers::{
        admin::Admin, convert_date_time_to_rfc3339_string, pagination::Pagination,
    },
};

use super::NeglectedPayment;

const VALID_PAGE_SIZE: u64 = 20;

pub(crate) async fn get_neglected_payments(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<Pagination>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<NeglectedPaymentsResult> {
    let op = NeglectedPaymentsOperationImpl { pool };
    handle_neglected_payments(query.page, query.per_page, op).await
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct NeglectedPaymentsResult {
    neglected_payments: Vec<NeglectedPayment>,
}

#[async_trait]
trait NeglectedPaymentsOperation {
    async fn get_neglected_payments(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<Vec<NeglectedPayment>, ErrResp>;
}

struct NeglectedPaymentsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl NeglectedPaymentsOperation for NeglectedPaymentsOperationImpl {
    async fn get_neglected_payments(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<Vec<NeglectedPayment>, ErrResp> {
        let models = entity::neglected_payment::Entity::find()
            .order_by_desc(entity::neglected_payment::Column::CreatedAt)
            .paginate(&self.pool, per_page)
            .fetch_page(page)
            .await
            .map_err(|e| {
                error!(
                    "failed to find neglected_payment (page: {}, per_page: {}): {}",
                    page, per_page, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| NeglectedPayment {
                consultation_id: m.consultation_id,
                user_account_id: m.user_account_id,
                consultant_id: m.consultant_id,
                meeting_at: convert_date_time_to_rfc3339_string(m.meeting_at),
                fee_per_hour_in_yen: m.fee_per_hour_in_yen,
                neglect_confirmed_by: m.neglect_confirmed_by,
                created_at: convert_date_time_to_rfc3339_string(m.created_at),
            })
            .collect())
    }
}

async fn handle_neglected_payments(
    page: u64,
    per_page: u64,
    op: impl NeglectedPaymentsOperation,
) -> RespResult<NeglectedPaymentsResult> {
    if per_page > VALID_PAGE_SIZE {
        error!("invalid per_page ({})", per_page);
        return Err(unexpected_err_resp());
    };

    let neglected_payments = op.get_neglected_payments(page, per_page).await?;

    Ok((
        StatusCode::OK,
        Json(NeglectedPaymentsResult { neglected_payments }),
    ))
}

#[cfg(test)]
mod tests {

    use chrono::{DateTime, Duration, TimeZone};
    use common::{JAPANESE_TIME_ZONE, LENGTH_OF_MEETING_IN_MINUTE};

    use crate::{
        err::Code,
        handlers::session::authentication::authenticated_handlers::WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS,
    };

    use super::*;

    struct NeglectedPaymentsOperationMock {
        page: u64,
        per_page: u64,
        neglected_payments: Vec<NeglectedPayment>,
    }

    #[async_trait]
    impl NeglectedPaymentsOperation for NeglectedPaymentsOperationMock {
        async fn get_neglected_payments(
            &self,
            page: u64,
            per_page: u64,
        ) -> Result<Vec<NeglectedPayment>, ErrResp> {
            assert_eq!(self.page, page);
            assert_eq!(self.per_page, per_page);
            let mut neglected_payments: Vec<NeglectedPayment> = self.neglected_payments.clone();
            neglected_payments.sort_by(|a, b| {
                DateTime::parse_from_rfc3339(&b.created_at)
                    .expect("failed to get Ok")
                    .cmp(&DateTime::parse_from_rfc3339(&a.created_at).expect("failed to get Ok"))
            });
            let length = neglected_payments.len();
            let page = page as usize;
            let per_page = per_page as usize;
            let start_index = page * per_page;
            let num = if length > per_page { per_page } else { length };
            let end_index = start_index + num;
            Ok(if length <= start_index {
                vec![]
            } else {
                neglected_payments[start_index..end_index].to_vec()
            })
        }
    }

    #[tokio::test]
    async fn test_handle_neglected_payments_success_case1() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE;
        let op = NeglectedPaymentsOperationMock {
            page,
            per_page,
            neglected_payments: vec![],
        };

        let result = handle_neglected_payments(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            NeglectedPaymentsResult {
                neglected_payments: vec![]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_neglected_payments_success_case2() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE;

        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let np1 = NeglectedPayment {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            neglect_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
        };

        let op = NeglectedPaymentsOperationMock {
            page,
            per_page,
            neglected_payments: vec![np1.clone()],
        };

        let result = handle_neglected_payments(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            NeglectedPaymentsResult {
                neglected_payments: vec![np1]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_neglected_payments_success_case3() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE;

        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let np1 = NeglectedPayment {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            neglect_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
        };

        let meeting_at2 = meeting_at1 + Duration::hours(1);
        let created_at2 = meeting_at2
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let np2 = NeglectedPayment {
            consultation_id: 4,
            user_account_id: 5,
            consultant_id: 6,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: 4000,
            neglect_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at2),
        };

        let op = NeglectedPaymentsOperationMock {
            page,
            per_page,
            neglected_payments: vec![np1.clone(), np2.clone()],
        };

        let result = handle_neglected_payments(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            NeglectedPaymentsResult {
                neglected_payments: vec![np2, np1]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_neglected_payments_success_case4() {
        let page = 0;
        let per_page = 1;

        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let np1 = NeglectedPayment {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            neglect_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
        };

        let meeting_at2 = meeting_at1 + Duration::hours(1);
        let created_at2 = meeting_at2
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let np2 = NeglectedPayment {
            consultation_id: 4,
            user_account_id: 5,
            consultant_id: 6,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: 4000,
            neglect_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at2),
        };

        let op = NeglectedPaymentsOperationMock {
            page,
            per_page,
            neglected_payments: vec![np1, np2.clone()],
        };

        let result = handle_neglected_payments(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            NeglectedPaymentsResult {
                neglected_payments: vec![np2]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_neglected_payments_success_case5() {
        let page = 1;
        let per_page = 1;

        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let np1 = NeglectedPayment {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            neglect_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
        };

        let meeting_at2 = meeting_at1 + Duration::hours(1);
        let created_at2 = meeting_at2
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let np2 = NeglectedPayment {
            consultation_id: 4,
            user_account_id: 5,
            consultant_id: 6,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: 4000,
            neglect_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at2),
        };

        let op = NeglectedPaymentsOperationMock {
            page,
            per_page,
            neglected_payments: vec![np1.clone(), np2],
        };

        let result = handle_neglected_payments(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            NeglectedPaymentsResult {
                neglected_payments: vec![np1]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_neglected_payments_success_case6() {
        let page = 2;
        let per_page = 1;

        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let np1 = NeglectedPayment {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            neglect_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
        };

        let meeting_at2 = meeting_at1 + Duration::hours(1);
        let created_at2 = meeting_at2
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let np2 = NeglectedPayment {
            consultation_id: 4,
            user_account_id: 5,
            consultant_id: 6,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: 4000,
            neglect_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at2),
        };

        let op = NeglectedPaymentsOperationMock {
            page,
            per_page,
            neglected_payments: vec![np1, np2],
        };

        let result = handle_neglected_payments(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            NeglectedPaymentsResult {
                neglected_payments: vec![]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_neglected_payments_fail_case1() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE + 1;

        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let np1 = NeglectedPayment {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            neglect_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
        };

        let op = NeglectedPaymentsOperationMock {
            page,
            per_page,
            neglected_payments: vec![np1.clone()],
        };

        let result = handle_neglected_payments(page, per_page, op).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.0);
        assert_eq!(Code::UnexpectedErr as u32, resp.1 .0.code);
    }
}
