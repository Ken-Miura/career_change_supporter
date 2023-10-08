// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::handlers::session::authentication::authenticated_handlers::{
    admin::Admin, pagination::Pagination,
};

const VALID_PAGE_SIZE: u64 = 20;

pub(crate) async fn get_refunded_payments(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<Pagination>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<RefundedPaymentsResult> {
    // let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    // let op = AwaitingWithdrawalsOperationImpl { pool };
    // handle_awaiting_withdrawals(query.page, query.per_page, current_date_time, op).await
    todo!()
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
