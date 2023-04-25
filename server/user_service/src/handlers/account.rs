// Copyright 2023 Ken Miura

use chrono::{DateTime, FixedOffset};

pub(crate) mod accounts;
pub(crate) mod delete_accounts;
pub(crate) mod temp_accounts;

#[derive(Clone, Debug)]
struct TempAccount {
    user_temp_account_id: String,
    email_address: String,
    hashed_password: Vec<u8>,
    created_at: DateTime<FixedOffset>,
}
