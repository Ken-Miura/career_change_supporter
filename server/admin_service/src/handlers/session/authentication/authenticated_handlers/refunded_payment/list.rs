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

const VALID_PAGE_SIZE: u64 = 20;

pub(crate) async fn get_refunded_payments(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<Pagination>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<RefundedPaymentsResult> {
    let op = RefundedPaymentsOperationImpl { pool };
    handle_refunded_payments(query.page, query.per_page, op).await
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct RefundedPaymentsResult {
    refunded_payments: Vec<RefundedPayment>,
}

#[derive(Clone, Serialize, Debug, PartialEq)]
struct RefundedPayment {
    consultation_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    meeting_at: String, // RFC 3339形式の文字列,
    fee_per_hour_in_yen: i32,
    transfer_fee_in_yen: i32,
    sender_name: String,
    reason: String,
    refund_confirmed_by: String,
    created_at: String, // RFC 3339形式の文字列
}

#[async_trait]
trait RefundedPaymentsOperation {
    async fn get_refunded_payments(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<Vec<RefundedPayment>, ErrResp>;
}

struct RefundedPaymentsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl RefundedPaymentsOperation for RefundedPaymentsOperationImpl {
    async fn get_refunded_payments(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<Vec<RefundedPayment>, ErrResp> {
        let models = entity::refunded_payment::Entity::find()
            .order_by_desc(entity::refunded_payment::Column::CreatedAt)
            .paginate(&self.pool, per_page)
            .fetch_page(page)
            .await
            .map_err(|e| {
                error!(
                    "failed to find refunded_payment (page: {}, per_page: {}): {}",
                    page, per_page, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| RefundedPayment {
                consultation_id: m.consultation_id,
                user_account_id: m.user_account_id,
                consultant_id: m.consultant_id,
                meeting_at: convert_date_time_to_rfc3339_string(m.meeting_at),
                fee_per_hour_in_yen: m.fee_per_hour_in_yen,
                transfer_fee_in_yen: m.transfer_fee_in_yen,
                sender_name: m.sender_name,
                reason: m.reason,
                refund_confirmed_by: m.refund_confirmed_by,
                created_at: convert_date_time_to_rfc3339_string(m.created_at),
            })
            .collect())
    }
}

async fn handle_refunded_payments(
    page: u64,
    per_page: u64,
    op: impl RefundedPaymentsOperation,
) -> RespResult<RefundedPaymentsResult> {
    if per_page > VALID_PAGE_SIZE {
        error!("invalid per_page ({})", per_page);
        return Err(unexpected_err_resp());
    };

    let refunded_payments = op.get_refunded_payments(page, per_page).await?;

    Ok((
        StatusCode::OK,
        Json(RefundedPaymentsResult { refunded_payments }),
    ))
}

#[cfg(test)]
mod tests {

    use chrono::{DateTime, Duration, TimeZone};
    use common::{JAPANESE_TIME_ZONE, LENGTH_OF_MEETING_IN_MINUTE};

    use crate::handlers::session::authentication::authenticated_handlers::{
        generate_sender_name, TRANSFER_FEE_IN_YEN,
        WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS,
    };

    use super::*;

    struct RefundedPaymentsOperationMock {
        page: u64,
        per_page: u64,
        refunded_payments: Vec<RefundedPayment>,
    }

    #[async_trait]
    impl RefundedPaymentsOperation for RefundedPaymentsOperationMock {
        async fn get_refunded_payments(
            &self,
            page: u64,
            per_page: u64,
        ) -> Result<Vec<RefundedPayment>, ErrResp> {
            assert_eq!(self.page, page);
            assert_eq!(self.per_page, per_page);
            let mut refunded_payments: Vec<RefundedPayment> = self.refunded_payments.clone();
            refunded_payments.sort_by(|a, b| {
                DateTime::parse_from_rfc3339(&b.created_at)
                    .expect("failed to get Ok")
                    .cmp(&DateTime::parse_from_rfc3339(&a.created_at).expect("failed to get Ok"))
            });
            let length = refunded_payments.len();
            let page = page as usize;
            let per_page = per_page as usize;
            let start_index = page * per_page;
            let num = if length > per_page { per_page } else { length };
            let end_index = start_index + num;
            Ok(if length <= start_index {
                vec![]
            } else {
                refunded_payments[start_index..end_index].to_vec()
            })
        }
    }

    #[tokio::test]
    async fn test_handle_refunded_payments_success_case1() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE;
        let op = RefundedPaymentsOperationMock {
            page,
            per_page,
            refunded_payments: vec![],
        };

        let result = handle_refunded_payments(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            RefundedPaymentsResult {
                refunded_payments: vec![]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_refunded_payments_success_case2() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE;

        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let rp1 = RefundedPayment {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            reason: "理由1".to_string(),
            refund_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
        };

        let op = RefundedPaymentsOperationMock {
            page,
            per_page,
            refunded_payments: vec![rp1.clone()],
        };

        let result = handle_refunded_payments(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            RefundedPaymentsResult {
                refunded_payments: vec![rp1]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_refunded_payments_success_case3() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE;

        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let rp1 = RefundedPayment {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            reason: "理由1".to_string(),
            refund_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
        };

        let meeting_at2 = meeting_at1 + Duration::hours(1);
        let created_at2 = meeting_at2
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let rp2 = RefundedPayment {
            consultation_id: 4,
            user_account_id: 5,
            consultant_id: 6,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: 4000,
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            sender_name: generate_sender_name(
                "スズキ".to_string(),
                "ジロウ".to_string(),
                meeting_at2,
            )
            .expect("failed to get Ok"),
            reason: "理由2".to_string(),
            refund_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at2),
        };

        let op = RefundedPaymentsOperationMock {
            page,
            per_page,
            refunded_payments: vec![rp1.clone(), rp2.clone()],
        };

        let result = handle_refunded_payments(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            RefundedPaymentsResult {
                refunded_payments: vec![rp2, rp1]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_refunded_payments_success_case4() {
        let page = 0;
        let per_page = 1;

        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let rp1 = RefundedPayment {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            reason: "理由1".to_string(),
            refund_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
        };

        let meeting_at2 = meeting_at1 + Duration::hours(1);
        let created_at2 = meeting_at2
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let rp2 = RefundedPayment {
            consultation_id: 4,
            user_account_id: 5,
            consultant_id: 6,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: 4000,
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            sender_name: generate_sender_name(
                "スズキ".to_string(),
                "ジロウ".to_string(),
                meeting_at2,
            )
            .expect("failed to get Ok"),
            reason: "理由2".to_string(),
            refund_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at2),
        };

        let op = RefundedPaymentsOperationMock {
            page,
            per_page,
            refunded_payments: vec![rp1, rp2.clone()],
        };

        let result = handle_refunded_payments(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            RefundedPaymentsResult {
                refunded_payments: vec![rp2]
            },
            resp.1 .0
        );
    }
}
