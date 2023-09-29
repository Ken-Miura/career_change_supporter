// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::handlers::session::authentication::authenticated_handlers::{
    admin::Admin, pagination::Pagination,
};

pub(crate) async fn get_waiting_for_payments(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<Pagination>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<WaitingForPaymentsResults> {
    let op = WaitingForPaymentsOperationImpl { pool };
    handle_waiting_for_payments(query.page, query.per_page, op).await
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct WaitingForPaymentsResults {
    total: i64,
    waiting_for_payments: Vec<WaitingForPayment>,
}

#[derive(Clone, Serialize, Debug, PartialEq)]
pub(crate) struct WaitingForPayment {
    consultation_id: i64,
    consultant_id: i64,
    user_account_id: i64,
    meeting_at: String, // RFC 3339形式の文字列
    fee_per_hour_in_yen: i32,
    sender_name: String,
}

async fn handle_waiting_for_payments(
    page: u64,
    per_page: u64,
    op: impl WaitingForPaymentsOperation,
) -> RespResult<WaitingForPaymentsResults> {
    todo!()
}

trait WaitingForPaymentsOperation {}

struct WaitingForPaymentsOperationImpl {
    pool: DatabaseConnection,
}

impl WaitingForPaymentsOperation for WaitingForPaymentsOperationImpl {}
