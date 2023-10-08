// Copyright 2023 Ken Miura

use async_session::async_trait;
use axum::extract::{Query, State};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;
use tracing::error;

use crate::{
    err::unexpected_err_resp,
    handlers::session::authentication::authenticated_handlers::{
        admin::Admin, pagination::Pagination,
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
trait RefundedPaymentsOperation {}

struct RefundedPaymentsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl RefundedPaymentsOperation for RefundedPaymentsOperationImpl {}

async fn handle_refunded_payments(
    page: u64,
    per_page: u64,
    op: impl RefundedPaymentsOperation,
) -> RespResult<RefundedPaymentsResult> {
    if per_page > VALID_PAGE_SIZE {
        error!("invalid per_page ({})", per_page);
        return Err(unexpected_err_resp());
    };

    todo!()
}
