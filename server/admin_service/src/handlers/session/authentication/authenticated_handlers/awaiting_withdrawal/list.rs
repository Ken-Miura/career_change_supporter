// Copyright 2023 Ken Miura

use async_session::async_trait;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE, LENGTH_OF_MEETING_IN_MINUTE};
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;
use tracing::error;

use crate::{
    err::unexpected_err_resp,
    handlers::session::authentication::authenticated_handlers::{
        admin::Admin, pagination::Pagination,
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
    bank_code: String,
    branch_code: String,
    account_type: String,
    account_number: String,
    account_holder_name: String,
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
        todo!()
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
