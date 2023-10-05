// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use chrono::Utc;
use common::{RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::handlers::session::authentication::authenticated_handlers::{
    admin::Admin, pagination::Pagination,
};

// DBテーブルの設計上、この回数分だけクエリを呼ぶようになるため、他より少なめな一方で運用上閲覧するのに十分な値を設定する
const VALID_PAGE_SIZE: u64 = 20;

pub(crate) async fn get_awaiting_withdrawals(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<Pagination>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<AwaitingWithdrawResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    todo!()
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct AwaitingWithdrawResult {
    awaiting_withdrawals: Vec<AwaitingWithdraw>,
}

#[derive(Clone, Serialize, Debug, PartialEq)]
struct AwaitingWithdraw {
    consultation_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    meeting_at: String, // RFC 3339形式の文字列,
    fee_per_hour_in_yen: i32,
    payment_confirmed_by: String,
    created_at: String, // RFC 3339形式の文字列
                        // TODO: 本人氏名、口座情報
}
