// Copyright 2023 Ken Miura

use serde::Serialize;

pub(crate) mod list;
pub(crate) mod refund_from_awaiting_payment;
pub(crate) mod refund_from_awaiting_withdrawal;
pub(crate) mod refunded_payment_by_consultation_id;

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
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
