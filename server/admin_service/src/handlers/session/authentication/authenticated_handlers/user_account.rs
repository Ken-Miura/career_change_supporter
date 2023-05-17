// Copyright 2023 Ken Miura

pub(crate) mod careers_by_user_account_id;
pub(crate) mod fee_per_hour_in_yen_by_user_account_id;
pub(crate) mod identity_option_by_user_account_id;
pub(crate) mod tenant_by_user_account_id;
pub(crate) mod user_account_retrieval_by_email_address;
pub(crate) mod user_account_retrieval_by_user_account_id;

use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::Code;

#[derive(Deserialize)]
pub(crate) struct UserAccountIdQuery {
    pub(super) user_account_id: i64,
}

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

fn validate_user_account_id_is_positive(user_account_id: i64) -> Result<(), ErrResp> {
    if !user_account_id.is_positive() {
        error!("user_account_id is not positive: {}", user_account_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::UserAccountIdIsNotPositive as u32,
            }),
        ));
    }
    Ok(())
}
