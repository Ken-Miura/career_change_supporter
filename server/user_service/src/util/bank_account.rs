// Copyright 2022 Ken Miura

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct BankAccount {
    pub(crate) bank_code: String,
    pub(crate) branch_code: String,
    pub(crate) account_type: String,
    pub(crate) account_number: String,
    pub(crate) account_holder_name: String,
}
