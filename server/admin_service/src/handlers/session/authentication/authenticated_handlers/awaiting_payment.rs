// Copyright 2023 Ken Miura

use serde::Serialize;

pub(crate) mod awaiting_payment_by_consultation_id;
pub(crate) mod expired_list;
pub(crate) mod list;

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct AwaitingPayment {
    consultation_id: i64,
    consultant_id: i64,
    user_account_id: i64,
    meeting_at: String, // RFC 3339形式の文字列
    fee_per_hour_in_yen: i32,
    sender_name: Option<String>,
}
