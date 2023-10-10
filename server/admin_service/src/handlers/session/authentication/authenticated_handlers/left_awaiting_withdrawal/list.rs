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

pub(crate) async fn get_left_awaiting_withdrawals(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<Pagination>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<LeftAwaitingWithdrawalsResult> {
    let op = LeftAwaitingWithdrawalsOperationImpl { pool };
    handle_left_awaiting_withdrawals(query.page, query.per_page, op).await
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct LeftAwaitingWithdrawalsResult {
    left_awaiting_withdrawals: Vec<ReceiptOfConsultation>,
}

#[derive(Clone, Serialize, Debug, PartialEq)]
struct ReceiptOfConsultation {
    consultation_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    meeting_at: String, // RFC 3339形式の文字列,
    fee_per_hour_in_yen: i32,
    platform_fee_rate_in_percentage: String,
    transfer_fee_in_yen: i32,
    reward: i32,
    sender_name: String,
    bank_code: String,
    branch_code: String,
    account_type: String,
    account_number: String,
    account_holder_name: String,
    withdrawal_confirmed_by: String,
    created_at: String, // RFC 3339形式の文字列
}

#[async_trait]
trait LeftAwaitingWithdrawalsOperation {
    async fn get_left_awaiting_withdrawals(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<Vec<ReceiptOfConsultation>, ErrResp>;
}

struct LeftAwaitingWithdrawalsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl LeftAwaitingWithdrawalsOperation for LeftAwaitingWithdrawalsOperationImpl {
    async fn get_left_awaiting_withdrawals(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<Vec<ReceiptOfConsultation>, ErrResp> {
        let models = entity::receipt_of_consultation::Entity::find()
            .order_by_desc(entity::receipt_of_consultation::Column::CreatedAt)
            .paginate(&self.pool, per_page)
            .fetch_page(page)
            .await
            .map_err(|e| {
                error!(
                    "failed to find receipt_of_consultation (page: {}, per_page: {}): {}",
                    page, per_page, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| ReceiptOfConsultation {
                consultation_id: m.consultation_id,
                user_account_id: m.user_account_id,
                consultant_id: m.consultant_id,
                meeting_at: convert_date_time_to_rfc3339_string(m.meeting_at),
                fee_per_hour_in_yen: m.fee_per_hour_in_yen,
                platform_fee_rate_in_percentage: m.platform_fee_rate_in_percentage,
                transfer_fee_in_yen: m.transfer_fee_in_yen,
                reward: m.reward,
                sender_name: m.sender_name,
                bank_code: m.bank_code,
                branch_code: m.branch_code,
                account_type: m.account_type,
                account_number: m.account_number,
                account_holder_name: m.account_holder_name,
                withdrawal_confirmed_by: m.withdrawal_confirmed_by,
                created_at: convert_date_time_to_rfc3339_string(m.created_at),
            })
            .collect())
    }
}

async fn handle_left_awaiting_withdrawals(
    page: u64,
    per_page: u64,
    op: impl LeftAwaitingWithdrawalsOperation,
) -> RespResult<LeftAwaitingWithdrawalsResult> {
    if per_page > VALID_PAGE_SIZE {
        error!("invalid per_page ({})", per_page);
        return Err(unexpected_err_resp());
    };

    let left_awaiting_withdrawals = op.get_left_awaiting_withdrawals(page, per_page).await?;

    Ok((
        StatusCode::OK,
        Json(LeftAwaitingWithdrawalsResult {
            left_awaiting_withdrawals,
        }),
    ))
}

#[cfg(test)]
mod tests {

    use chrono::{DateTime, Duration, TimeZone};
    use common::{JAPANESE_TIME_ZONE, LENGTH_OF_MEETING_IN_MINUTE};

    use crate::{
        err::Code,
        handlers::session::authentication::authenticated_handlers::{
            calculate_reward, generate_sender_name, TRANSFER_FEE_IN_YEN,
            WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS,
        },
    };

    use super::*;

    struct LeftAwaitingWithdrawalsOperationMock {
        page: u64,
        per_page: u64,
        left_awaiting_withdrawals: Vec<ReceiptOfConsultation>,
    }

    #[async_trait]
    impl LeftAwaitingWithdrawalsOperation for LeftAwaitingWithdrawalsOperationMock {
        async fn get_left_awaiting_withdrawals(
            &self,
            page: u64,
            per_page: u64,
        ) -> Result<Vec<ReceiptOfConsultation>, ErrResp> {
            assert_eq!(self.page, page);
            assert_eq!(self.per_page, per_page);
            let mut left_awaiting_withdrawals: Vec<ReceiptOfConsultation> =
                self.left_awaiting_withdrawals.clone();
            left_awaiting_withdrawals.sort_by(|a, b| {
                DateTime::parse_from_rfc3339(&b.created_at)
                    .expect("failed to get Ok")
                    .cmp(&DateTime::parse_from_rfc3339(&a.created_at).expect("failed to get Ok"))
            });
            let length = left_awaiting_withdrawals.len();
            let page = page as usize;
            let per_page = per_page as usize;
            let start_index = page * per_page;
            let num = if length > per_page { per_page } else { length };
            let end_index = start_index + num;
            Ok(if length <= start_index {
                vec![]
            } else {
                left_awaiting_withdrawals[start_index..end_index].to_vec()
            })
        }
    }

    #[tokio::test]
    async fn test_handle_left_awaiting_withdrawals_success_case1() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE;
        let op = LeftAwaitingWithdrawalsOperationMock {
            page,
            per_page,
            left_awaiting_withdrawals: vec![],
        };

        let result = handle_left_awaiting_withdrawals(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            LeftAwaitingWithdrawalsResult {
                left_awaiting_withdrawals: vec![]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_left_awaiting_withdrawals_success_case2() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE;

        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let rp1 = ReceiptOfConsultation {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            platform_fee_rate_in_percentage: "50.0".to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(4000, "50.0", *TRANSFER_FEE_IN_YEN).expect("failed to get Ok"),
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "スズキ　ジロウ".to_string(),
            withdrawal_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
        };

        let op = LeftAwaitingWithdrawalsOperationMock {
            page,
            per_page,
            left_awaiting_withdrawals: vec![rp1.clone()],
        };

        let result = handle_left_awaiting_withdrawals(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            LeftAwaitingWithdrawalsResult {
                left_awaiting_withdrawals: vec![rp1]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_left_awaiting_withdrawals_success_case3() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE;

        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let rp1 = ReceiptOfConsultation {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            platform_fee_rate_in_percentage: "50.0".to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(4000, "50.0", *TRANSFER_FEE_IN_YEN).expect("failed to get Ok"),
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "スズキ　ジロウ".to_string(),
            withdrawal_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
        };

        let meeting_at2 = meeting_at1 + Duration::hours(1);
        let created_at2 = meeting_at2
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let rp2 = ReceiptOfConsultation {
            consultation_id: 4,
            user_account_id: 5,
            consultant_id: 6,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: 4000,
            platform_fee_rate_in_percentage: "50.0".to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(4000, "50.0", *TRANSFER_FEE_IN_YEN).expect("failed to get Ok"),
            sender_name: generate_sender_name(
                "スズキ".to_string(),
                "ジロウ".to_string(),
                meeting_at2,
            )
            .expect("failed to get Ok"),
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "7654321".to_string(),
            account_holder_name: "サトウ　サブロウ".to_string(),
            withdrawal_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at2),
        };

        let op = LeftAwaitingWithdrawalsOperationMock {
            page,
            per_page,
            left_awaiting_withdrawals: vec![rp1.clone(), rp2.clone()],
        };

        let result = handle_left_awaiting_withdrawals(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            LeftAwaitingWithdrawalsResult {
                left_awaiting_withdrawals: vec![rp2, rp1]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_left_awaiting_withdrawals_success_case4() {
        let page = 0;
        let per_page = 1;

        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let rp1 = ReceiptOfConsultation {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            platform_fee_rate_in_percentage: "50.0".to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(4000, "50.0", *TRANSFER_FEE_IN_YEN).expect("failed to get Ok"),
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "スズキ　ジロウ".to_string(),
            withdrawal_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
        };

        let meeting_at2 = meeting_at1 + Duration::hours(1);
        let created_at2 = meeting_at2
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let rp2 = ReceiptOfConsultation {
            consultation_id: 4,
            user_account_id: 5,
            consultant_id: 6,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: 4000,
            platform_fee_rate_in_percentage: "50.0".to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(4000, "50.0", *TRANSFER_FEE_IN_YEN).expect("failed to get Ok"),
            sender_name: generate_sender_name(
                "スズキ".to_string(),
                "ジロウ".to_string(),
                meeting_at2,
            )
            .expect("failed to get Ok"),
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "7654321".to_string(),
            account_holder_name: "サトウ　サブロウ".to_string(),
            withdrawal_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at2),
        };

        let op = LeftAwaitingWithdrawalsOperationMock {
            page,
            per_page,
            left_awaiting_withdrawals: vec![rp1, rp2.clone()],
        };

        let result = handle_left_awaiting_withdrawals(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            LeftAwaitingWithdrawalsResult {
                left_awaiting_withdrawals: vec![rp2]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_left_awaiting_withdrawals_success_case5() {
        let page = 1;
        let per_page = 1;

        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let rp1 = ReceiptOfConsultation {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            platform_fee_rate_in_percentage: "50.0".to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(4000, "50.0", *TRANSFER_FEE_IN_YEN).expect("failed to get Ok"),
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "スズキ　ジロウ".to_string(),
            withdrawal_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
        };

        let meeting_at2 = meeting_at1 + Duration::hours(1);
        let created_at2 = meeting_at2
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let rp2 = ReceiptOfConsultation {
            consultation_id: 4,
            user_account_id: 5,
            consultant_id: 6,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: 4000,
            platform_fee_rate_in_percentage: "50.0".to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(4000, "50.0", *TRANSFER_FEE_IN_YEN).expect("failed to get Ok"),
            sender_name: generate_sender_name(
                "スズキ".to_string(),
                "ジロウ".to_string(),
                meeting_at2,
            )
            .expect("failed to get Ok"),
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "7654321".to_string(),
            account_holder_name: "サトウ　サブロウ".to_string(),
            withdrawal_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at2),
        };

        let op = LeftAwaitingWithdrawalsOperationMock {
            page,
            per_page,
            left_awaiting_withdrawals: vec![rp1.clone(), rp2],
        };

        let result = handle_left_awaiting_withdrawals(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            LeftAwaitingWithdrawalsResult {
                left_awaiting_withdrawals: vec![rp1]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_left_awaiting_withdrawals_success_case6() {
        let page = 2;
        let per_page = 1;

        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let rp1 = ReceiptOfConsultation {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            platform_fee_rate_in_percentage: "50.0".to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(4000, "50.0", *TRANSFER_FEE_IN_YEN).expect("failed to get Ok"),
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "スズキ　ジロウ".to_string(),
            withdrawal_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
        };

        let meeting_at2 = meeting_at1 + Duration::hours(1);
        let created_at2 = meeting_at2
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let rp2 = ReceiptOfConsultation {
            consultation_id: 4,
            user_account_id: 5,
            consultant_id: 6,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: 4000,
            platform_fee_rate_in_percentage: "50.0".to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(4000, "50.0", *TRANSFER_FEE_IN_YEN).expect("failed to get Ok"),
            sender_name: generate_sender_name(
                "スズキ".to_string(),
                "ジロウ".to_string(),
                meeting_at2,
            )
            .expect("failed to get Ok"),
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "7654321".to_string(),
            account_holder_name: "サトウ　サブロウ".to_string(),
            withdrawal_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at2),
        };

        let op = LeftAwaitingWithdrawalsOperationMock {
            page,
            per_page,
            left_awaiting_withdrawals: vec![rp1, rp2],
        };

        let result = handle_left_awaiting_withdrawals(page, per_page, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            LeftAwaitingWithdrawalsResult {
                left_awaiting_withdrawals: vec![]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_left_awaiting_withdrawals_fail_case1() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE + 1;

        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::days(1);
        let rp1 = ReceiptOfConsultation {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            platform_fee_rate_in_percentage: "50.0".to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(4000, "50.0", *TRANSFER_FEE_IN_YEN).expect("failed to get Ok"),
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            bank_code: "0001".to_string(),
            branch_code: "001".to_string(),
            account_type: "普通".to_string(),
            account_number: "1234567".to_string(),
            account_holder_name: "スズキ　ジロウ".to_string(),
            withdrawal_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
        };

        let op = LeftAwaitingWithdrawalsOperationMock {
            page,
            per_page,
            left_awaiting_withdrawals: vec![rp1.clone()],
        };

        let result = handle_left_awaiting_withdrawals(page, per_page, op).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.0);
        assert_eq!(Code::UnexpectedErr as u32, resp.1 .0.code);
    }
}
