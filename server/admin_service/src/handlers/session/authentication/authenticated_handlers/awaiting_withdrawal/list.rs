// Copyright 2023 Ken Miura

use async_session::async_trait;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE, LENGTH_OF_MEETING_IN_MINUTE};
use entity::sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use serde::Serialize;
use tracing::error;

use crate::{
    err::unexpected_err_resp,
    handlers::session::authentication::authenticated_handlers::{
        admin::Admin, calculate_reward, convert_date_time_to_rfc3339_string,
        pagination::Pagination, PLATFORM_FEE_RATE_IN_PERCENTAGE, TRANSFER_FEE_IN_YEN,
        WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS,
    },
};

const VALID_PAGE_SIZE: u64 = 20;

pub(crate) async fn get_awaiting_withdrawals(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<Pagination>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<AwaitingWithdrawalResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = AwaitingWithdrawalsOperationImpl { pool };
    handle_awaiting_withdrawals(query.page, query.per_page, current_date_time, op).await
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct AwaitingWithdrawalResult {
    awaiting_withdrawals: Vec<AwaitingWithdrawal>,
}

#[derive(Clone, Serialize, Debug, PartialEq)]
struct AwaitingWithdrawal {
    consultation_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    meeting_at: String, // RFC 3339形式の文字列,
    fee_per_hour_in_yen: i32,
    sender_name: String,
    payment_confirmed_by: String,
    created_at: String, // RFC 3339形式の文字列
    bank_code: Option<String>,
    branch_code: Option<String>,
    account_type: Option<String>,
    account_number: Option<String>,
    account_holder_name: Option<String>,
    platform_fee_rate_in_percentage: String,
    transfer_fee_in_yen: i32,
    reward: i32,
}

#[async_trait]
trait AwaitingWithdrawalsOperation {
    async fn get_awaiting_withdrawals(
        &self,
        page: u64,
        per_page: u64,
        criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<AwaitingWithdrawal>, ErrResp>;
}

struct AwaitingWithdrawalsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl AwaitingWithdrawalsOperation for AwaitingWithdrawalsOperationImpl {
    async fn get_awaiting_withdrawals(
        &self,
        page: u64,
        per_page: u64,
        criteria: DateTime<FixedOffset>,
    ) -> Result<Vec<AwaitingWithdrawal>, ErrResp> {
        let models = entity::awaiting_withdrawal::Entity::find()
            .filter(entity::awaiting_withdrawal::Column::MeetingAt.lt(criteria))
            .find_also_related(entity::bank_account::Entity)
            .order_by_asc(entity::awaiting_withdrawal::Column::MeetingAt)
            .paginate(&self.pool, per_page)
            .fetch_page(page)
            .await
            .map_err(|e| {
                error!(
                    "failed to find awaiting_withdrawal (page: {}, per_page: {}, criteria: {}): {}",
                    page, per_page, criteria, e
                );
                unexpected_err_resp()
            })?;
        models
            .into_iter()
            .map(|m| {
                let aw = m.0;
                let ba_option = m.1;
                let (bank_code, branch_code, account_type, account_number, account_holder_name) =
                    if let Some(ba) = ba_option {
                        (
                            Some(ba.bank_code),
                            Some(ba.branch_code),
                            Some(ba.account_type),
                            Some(ba.account_number),
                            Some(ba.account_holder_name),
                        )
                    } else {
                        (None, None, None, None, None)
                    };
                let reward = calculate_reward(aw.fee_per_hour_in_yen, &PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(), *TRANSFER_FEE_IN_YEN).map_err(|e|{
                    error!("failed to calculate_reward (fee_per_hour_in_yen: {}, platform_fee_rate_in_percentage: {}, transfer_fee_in_yen: {}): {:?}",
                        aw.fee_per_hour_in_yen, *PLATFORM_FEE_RATE_IN_PERCENTAGE, *TRANSFER_FEE_IN_YEN, e);
                    unexpected_err_resp()
                })?;
                Ok(AwaitingWithdrawal {
                    consultation_id: aw.consultation_id,
                    user_account_id: aw.user_account_id,
                    consultant_id: aw.consultant_id,
                    meeting_at: convert_date_time_to_rfc3339_string(aw.meeting_at),
                    fee_per_hour_in_yen: aw.fee_per_hour_in_yen,
                    sender_name: aw.sender_name,
                    payment_confirmed_by: aw.payment_confirmed_by,
                    created_at: convert_date_time_to_rfc3339_string(aw.created_at),
                    bank_code,
                    branch_code,
                    account_type,
                    account_number,
                    account_holder_name,
                    platform_fee_rate_in_percentage: PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
                    transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
                    reward
                })
            })
            .collect()
    }
}

async fn handle_awaiting_withdrawals(
    page: u64,
    per_page: u64,
    current_date_time: DateTime<FixedOffset>,
    op: impl AwaitingWithdrawalsOperation,
) -> RespResult<AwaitingWithdrawalResult> {
    if per_page > VALID_PAGE_SIZE {
        error!("invalid per_page ({})", per_page);
        return Err(unexpected_err_resp());
    };

    let criteria = current_date_time
        - Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
        - Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64);

    let awaiting_withdrawals = op
        .get_awaiting_withdrawals(page, per_page, criteria)
        .await?;

    Ok((
        StatusCode::OK,
        Json(AwaitingWithdrawalResult {
            awaiting_withdrawals,
        }),
    ))
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;

    use crate::{
        err::Code, handlers::session::authentication::authenticated_handlers::generate_sender_name,
    };

    use super::*;

    struct AwaitingWithdrawalsOperationMock {
        page: u64,
        per_page: u64,
        current_date_time: DateTime<FixedOffset>,
        awaiting_withdrawals: Vec<AwaitingWithdrawal>,
    }

    #[async_trait]
    impl AwaitingWithdrawalsOperation for AwaitingWithdrawalsOperationMock {
        async fn get_awaiting_withdrawals(
            &self,
            page: u64,
            per_page: u64,
            criteria: DateTime<FixedOffset>,
        ) -> Result<Vec<AwaitingWithdrawal>, ErrResp> {
            assert_eq!(self.page, page);
            assert_eq!(self.per_page, per_page);
            assert_eq!(
                self.current_date_time
                    - Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
                    - Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64),
                criteria
            );
            let awaiting_withdrawals: Vec<AwaitingWithdrawal> = self
                .awaiting_withdrawals
                .clone()
                .into_iter()
                .filter(|aw| {
                    DateTime::parse_from_rfc3339(&aw.meeting_at).expect("failed to get Ok")
                        < criteria
                })
                .collect();
            let length = awaiting_withdrawals.len();
            let page = page as usize;
            let per_page = per_page as usize;
            let start_index = page * per_page;
            let num = if length > per_page { per_page } else { length };
            let end_index = start_index + num;
            Ok(if length <= start_index {
                vec![]
            } else {
                awaiting_withdrawals[start_index..end_index].to_vec()
            })
        }
    }

    #[tokio::test]
    async fn test_handle_awaiting_withdrawals_success_case1() {
        let page = 0;
        let per_page = VALID_PAGE_SIZE;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = AwaitingWithdrawalsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_withdrawals: vec![],
        };

        let result = handle_awaiting_withdrawals(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            AwaitingWithdrawalResult {
                awaiting_withdrawals: vec![]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_awaiting_withdrawals_success_case2() {
        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1 - Duration::days(5);
        let fee_per_hour_in_yen1 = 3000;
        let awaiting_withdrawal1 = AwaitingWithdrawal {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: fee_per_hour_in_yen1,
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            payment_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
            bank_code: Some("0001".to_string()),
            branch_code: Some("001".to_string()),
            account_type: Some("普通".to_string()),
            account_number: Some("1234567".to_string()),
            account_holder_name: Some("スズキ　ジロウ".to_string()),
            platform_fee_rate_in_percentage: PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(
                fee_per_hour_in_yen1,
                &PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
                *TRANSFER_FEE_IN_YEN,
            )
            .expect("failed to get Ok"),
        };

        let page = 0;
        let per_page = VALID_PAGE_SIZE;
        let current_date_time = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::seconds(1);

        let op = AwaitingWithdrawalsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_withdrawals: vec![awaiting_withdrawal1.clone()],
        };

        let result = handle_awaiting_withdrawals(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            AwaitingWithdrawalResult {
                awaiting_withdrawals: vec![awaiting_withdrawal1]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_awaiting_withdrawals_success_case3() {
        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1 - Duration::days(5);
        let fee_per_hour_in_yen1 = 4000;
        let awaiting_withdrawal1 = AwaitingWithdrawal {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: fee_per_hour_in_yen1,
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            payment_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
            bank_code: Some("0001".to_string()),
            branch_code: Some("001".to_string()),
            account_type: Some("普通".to_string()),
            account_number: Some("1234567".to_string()),
            account_holder_name: Some("スズキ　ジロウ".to_string()),
            platform_fee_rate_in_percentage: PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(
                fee_per_hour_in_yen1,
                &PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
                *TRANSFER_FEE_IN_YEN,
            )
            .expect("failed to get Ok"),
        };

        let meeting_at2 = meeting_at1 + Duration::days(1);
        let created_at2 = meeting_at2 - Duration::days(5);
        let fee_per_hour_in_yen2 = 5000;
        let awaiting_withdrawal2 = AwaitingWithdrawal {
            consultation_id: 4,
            user_account_id: 5,
            consultant_id: 6,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: fee_per_hour_in_yen2,
            sender_name: generate_sender_name(
                "サトウ".to_string(),
                "サブロウ".to_string(),
                meeting_at2,
            )
            .expect("failed to get Ok"),
            payment_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at2),
            bank_code: Some("0005".to_string()),
            branch_code: Some("004".to_string()),
            account_type: Some("普通".to_string()),
            account_number: Some("7654321".to_string()),
            account_holder_name: Some("タカハシ　シロウ".to_string()),
            platform_fee_rate_in_percentage: PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(
                fee_per_hour_in_yen2,
                &PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
                *TRANSFER_FEE_IN_YEN,
            )
            .expect("failed to get Ok"),
        };

        let page = 0;
        let per_page = VALID_PAGE_SIZE;
        let current_date_time = std::cmp::max(meeting_at1, meeting_at2)
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::seconds(1);

        let op = AwaitingWithdrawalsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_withdrawals: vec![awaiting_withdrawal1.clone(), awaiting_withdrawal2.clone()],
        };

        let result = handle_awaiting_withdrawals(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            AwaitingWithdrawalResult {
                awaiting_withdrawals: vec![awaiting_withdrawal1, awaiting_withdrawal2]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_awaiting_withdrawals_success_case4() {
        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let fee_per_hour_in_yen1 = 4000;
        let created_at1 = meeting_at1 - Duration::days(5);
        let awaiting_withdrawal1 = AwaitingWithdrawal {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: fee_per_hour_in_yen1,
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            payment_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
            bank_code: Some("0001".to_string()),
            branch_code: Some("001".to_string()),
            account_type: Some("普通".to_string()),
            account_number: Some("1234567".to_string()),
            account_holder_name: Some("スズキ　ジロウ".to_string()),
            platform_fee_rate_in_percentage: PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(
                fee_per_hour_in_yen1,
                &PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
                *TRANSFER_FEE_IN_YEN,
            )
            .expect("failed to get Ok"),
        };

        let meeting_at2 = meeting_at1 + Duration::days(1);
        let created_at2 = meeting_at2 - Duration::days(5);
        let fee_per_hour_in_yen2 = 5000;
        let awaiting_withdrawal2 = AwaitingWithdrawal {
            consultation_id: 4,
            user_account_id: 5,
            consultant_id: 6,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: fee_per_hour_in_yen2,
            sender_name: generate_sender_name(
                "サトウ".to_string(),
                "サブロウ".to_string(),
                meeting_at2,
            )
            .expect("failed to get Ok"),
            payment_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at2),
            bank_code: Some("0005".to_string()),
            branch_code: Some("004".to_string()),
            account_type: Some("普通".to_string()),
            account_number: Some("7654321".to_string()),
            account_holder_name: Some("タカハシ　シロウ".to_string()),
            platform_fee_rate_in_percentage: PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(
                fee_per_hour_in_yen2,
                &PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
                *TRANSFER_FEE_IN_YEN,
            )
            .expect("failed to get Ok"),
        };

        let page = 0;
        let per_page = 1;
        let current_date_time = std::cmp::max(meeting_at1, meeting_at2)
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::seconds(1);

        let op = AwaitingWithdrawalsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_withdrawals: vec![awaiting_withdrawal1.clone(), awaiting_withdrawal2],
        };

        let result = handle_awaiting_withdrawals(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            AwaitingWithdrawalResult {
                awaiting_withdrawals: vec![awaiting_withdrawal1]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_awaiting_withdrawals_success_case5() {
        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1 - Duration::days(5);
        let fee_per_hour_in_yen1 = 4000;
        let awaiting_withdrawal1 = AwaitingWithdrawal {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: fee_per_hour_in_yen1,
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            payment_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
            bank_code: Some("0001".to_string()),
            branch_code: Some("001".to_string()),
            account_type: Some("普通".to_string()),
            account_number: Some("1234567".to_string()),
            account_holder_name: Some("スズキ　ジロウ".to_string()),
            platform_fee_rate_in_percentage: PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(
                fee_per_hour_in_yen1,
                &PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
                *TRANSFER_FEE_IN_YEN,
            )
            .expect("failed to get Ok"),
        };

        let meeting_at2 = meeting_at1 + Duration::days(1);
        let created_at2 = meeting_at2 - Duration::days(5);
        let fee_per_hour_in_yen2 = 5000;
        let awaiting_withdrawal2 = AwaitingWithdrawal {
            consultation_id: 4,
            user_account_id: 5,
            consultant_id: 6,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: fee_per_hour_in_yen2,
            sender_name: generate_sender_name(
                "サトウ".to_string(),
                "サブロウ".to_string(),
                meeting_at2,
            )
            .expect("failed to get Ok"),
            payment_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at2),
            bank_code: Some("0005".to_string()),
            branch_code: Some("004".to_string()),
            account_type: Some("普通".to_string()),
            account_number: Some("7654321".to_string()),
            account_holder_name: Some("タカハシ　シロウ".to_string()),
            platform_fee_rate_in_percentage: PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(
                fee_per_hour_in_yen2,
                &PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
                *TRANSFER_FEE_IN_YEN,
            )
            .expect("failed to get Ok"),
        };

        let page = 1;
        let per_page = 1;
        let current_date_time = std::cmp::max(meeting_at1, meeting_at2)
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::seconds(1);

        let op = AwaitingWithdrawalsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_withdrawals: vec![awaiting_withdrawal1, awaiting_withdrawal2.clone()],
        };

        let result = handle_awaiting_withdrawals(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            AwaitingWithdrawalResult {
                awaiting_withdrawals: vec![awaiting_withdrawal2]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_awaiting_withdrawals_success_case6() {
        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1 - Duration::days(5);
        let fee_per_hour_in_yen1 = 4000;
        let awaiting_withdrawal1 = AwaitingWithdrawal {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: 4000,
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            payment_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
            bank_code: Some("0001".to_string()),
            branch_code: Some("001".to_string()),
            account_type: Some("普通".to_string()),
            account_number: Some("1234567".to_string()),
            account_holder_name: Some("スズキ　ジロウ".to_string()),
            platform_fee_rate_in_percentage: PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(
                fee_per_hour_in_yen1,
                &PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
                *TRANSFER_FEE_IN_YEN,
            )
            .expect("failed to get Ok"),
        };

        let meeting_at2 = meeting_at1 + Duration::days(1);
        let created_at2 = meeting_at2 - Duration::days(5);
        let fee_per_hour_in_yen2 = 5000;
        let awaiting_withdrawal2 = AwaitingWithdrawal {
            consultation_id: 4,
            user_account_id: 5,
            consultant_id: 6,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: fee_per_hour_in_yen2,
            sender_name: generate_sender_name(
                "サトウ".to_string(),
                "サブロウ".to_string(),
                meeting_at2,
            )
            .expect("failed to get Ok"),
            payment_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at2),
            bank_code: Some("0005".to_string()),
            branch_code: Some("004".to_string()),
            account_type: Some("普通".to_string()),
            account_number: Some("7654321".to_string()),
            account_holder_name: Some("タカハシ　シロウ".to_string()),
            platform_fee_rate_in_percentage: PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(
                fee_per_hour_in_yen2,
                &PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
                *TRANSFER_FEE_IN_YEN,
            )
            .expect("failed to get Ok"),
        };

        let page = 2;
        let per_page = 1;
        let current_date_time = std::cmp::max(meeting_at1, meeting_at2)
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::seconds(1);

        let op = AwaitingWithdrawalsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_withdrawals: vec![awaiting_withdrawal1, awaiting_withdrawal2],
        };

        let result = handle_awaiting_withdrawals(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            AwaitingWithdrawalResult {
                awaiting_withdrawals: vec![]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_awaiting_withdrawals_success_case7() {
        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1 - Duration::days(5);
        let fee_per_hour_in_yen1 = 4000;
        let awaiting_withdrawal1 = AwaitingWithdrawal {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: fee_per_hour_in_yen1,
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            payment_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
            bank_code: Some("0001".to_string()),
            branch_code: Some("001".to_string()),
            account_type: Some("普通".to_string()),
            account_number: Some("1234567".to_string()),
            account_holder_name: Some("スズキ　ジロウ".to_string()),
            platform_fee_rate_in_percentage: PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(
                fee_per_hour_in_yen1,
                &PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
                *TRANSFER_FEE_IN_YEN,
            )
            .expect("failed to get Ok"),
        };

        let meeting_at2 = meeting_at1 + Duration::days(15);
        let created_at2 = meeting_at2 - Duration::days(5);
        let fee_per_hour_in_yen2 = 5000;
        let awaiting_withdrawal2 = AwaitingWithdrawal {
            consultation_id: 4,
            user_account_id: 5,
            consultant_id: 6,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at2),
            fee_per_hour_in_yen: fee_per_hour_in_yen2,
            sender_name: generate_sender_name(
                "サトウ".to_string(),
                "サブロウ".to_string(),
                meeting_at2,
            )
            .expect("failed to get Ok"),
            payment_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at2),
            bank_code: Some("0005".to_string()),
            branch_code: Some("004".to_string()),
            account_type: Some("普通".to_string()),
            account_number: Some("7654321".to_string()),
            account_holder_name: Some("タカハシ　シロウ".to_string()),
            platform_fee_rate_in_percentage: PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(
                fee_per_hour_in_yen2,
                &PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
                *TRANSFER_FEE_IN_YEN,
            )
            .expect("failed to get Ok"),
        };

        let page = 0;
        let per_page = VALID_PAGE_SIZE;
        let current_date_time = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::seconds(1);

        let op = AwaitingWithdrawalsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_withdrawals: vec![awaiting_withdrawal1.clone(), awaiting_withdrawal2],
        };

        let result = handle_awaiting_withdrawals(page, per_page, current_date_time, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            AwaitingWithdrawalResult {
                awaiting_withdrawals: vec![awaiting_withdrawal1]
            },
            resp.1 .0
        );
    }

    #[tokio::test]
    async fn test_handle_awaiting_withdrawals_fail_case1() {
        let meeting_at1 = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 25, 21, 0, 0)
            .unwrap();
        let created_at1 = meeting_at1 - Duration::days(5);
        let fee_per_hour_in_yen1 = 3000;
        let awaiting_withdrawal1 = AwaitingWithdrawal {
            consultation_id: 1,
            user_account_id: 2,
            consultant_id: 3,
            meeting_at: convert_date_time_to_rfc3339_string(meeting_at1),
            fee_per_hour_in_yen: fee_per_hour_in_yen1,
            sender_name: generate_sender_name(
                "タナカ".to_string(),
                "タロウ".to_string(),
                meeting_at1,
            )
            .expect("failed to get Ok"),
            payment_confirmed_by: "admin@test.com".to_string(),
            created_at: convert_date_time_to_rfc3339_string(created_at1),
            bank_code: Some("0001".to_string()),
            branch_code: Some("001".to_string()),
            account_type: Some("普通".to_string()),
            account_number: Some("1234567".to_string()),
            account_holder_name: Some("スズキ　ジロウ".to_string()),
            platform_fee_rate_in_percentage: PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            reward: calculate_reward(
                fee_per_hour_in_yen1,
                &PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
                *TRANSFER_FEE_IN_YEN,
            )
            .expect("failed to get Ok"),
        };

        let page = 0;
        let per_page = VALID_PAGE_SIZE + 1;
        let current_date_time = meeting_at1
            + Duration::days(WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS)
            + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64)
            + Duration::seconds(1);

        let op = AwaitingWithdrawalsOperationMock {
            page,
            per_page,
            current_date_time,
            awaiting_withdrawals: vec![awaiting_withdrawal1.clone()],
        };

        let result = handle_awaiting_withdrawals(page, per_page, current_date_time, op).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.0);
        assert_eq!(Code::UnexpectedErr as u32, resp.1 .0.code);
    }
}
