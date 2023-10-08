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
        admin::Admin, convert_date_time_to_rfc3339_string, pagination::Pagination,
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
        Ok(models
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
                AwaitingWithdrawal {
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
                }
            })
            .collect())
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

    // use crate::err::Code;

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
}
