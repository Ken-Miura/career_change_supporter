// Copyright 2023 Ken Miura

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

// DBテーブルの設計上、この回数分だけクエリを呼ぶようになるため、他より少なめな一方で運用上閲覧するのに十分な値を設定する
const VALID_PAGE_SIZE: u64 = 20;

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
    if per_page != VALID_PAGE_SIZE {
        error!("invalid per_page ({})", per_page);
        return Err(unexpected_err_resp());
    };
    todo!()
}

trait WaitingForPaymentsOperation {}

struct WaitingForPaymentsOperationImpl {
    pool: DatabaseConnection,
}

impl WaitingForPaymentsOperation for WaitingForPaymentsOperationImpl {}
