// Copyright 2023 Ken Miura

use serde::Serialize;

pub(crate) mod list;
pub(crate) mod post;
pub(crate) mod receipt_of_consultation_by_consultation_id;

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ReceiptOfConsultation {
    consultation_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    meeting_at: String, // RFC 3339形式の文字列,
    fee_per_hour_in_yen: i32,
    platform_fee_rate_in_percentage: String,
    transfer_fee_in_yen: i32,
    reward: i32,
    sender_name: String,
    bank_code: String,
    branch_code: String,
    account_type: String,
    account_number: String,
    account_holder_name: String,
    withdrawal_confirmed_by: String,
    created_at: String, // RFC 3339形式の文字列
}
