// Copyright 2023 Ken Miura

pub(crate) mod user_account_retrieval_by_email_address;
pub(crate) mod user_account_retrieval_by_user_account_id;

use serde::Serialize;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct UserAccountRetrievalResult {}
