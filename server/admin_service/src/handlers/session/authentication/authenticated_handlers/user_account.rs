// Copyright 2023 Ken Miura

pub(crate) mod user_account_retrieval_by_email_address;
pub(crate) mod user_account_retrieval_by_user_account_id;

use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct UserAccountRetrievalResult {
    pub(super) user_account: Option<UserAccount>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(super) struct UserAccount {
    pub(super) user_account_id: i64,
    pub(super) email_address: String,
    pub(super) last_login_time: Option<String>, // RFC 3339形式の文字列
    pub(super) created_at: String,              // RFC 3339形式の文字列
    pub(super) mfa_enabled_at: Option<String>,  // RFC 3339形式の文字列
    pub(super) disabled_at: Option<String>,     // RFC 3339形式の文字列
}
