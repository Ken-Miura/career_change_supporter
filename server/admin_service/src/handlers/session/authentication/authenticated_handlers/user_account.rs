// Copyright 2023 Ken Miura

pub(crate) mod agreements_by_user_account_id;
pub(crate) mod careers_by_user_account_id;
pub(crate) mod consultation_reqs_by_consultant_id;
pub(crate) mod consultation_reqs_by_user_account_id;
pub(crate) mod fee_per_hour_in_yen_by_user_account_id;
pub(crate) mod identity_option_by_user_account_id;
pub(crate) mod tenant_id_by_user_account_id;
pub(crate) mod user_account_retrieval_by_email_address;
pub(crate) mod user_account_retrieval_by_user_account_id;

use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::Code;

#[derive(Deserialize)]
pub(crate) struct UserAccountIdQuery {
    user_account_id: i64,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct UserAccountRetrievalResult {
    user_account: Option<UserAccount>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
struct UserAccount {
    user_account_id: i64,
    email_address: String,
    last_login_time: Option<String>, // RFC 3339形式の文字列
    created_at: String,              // RFC 3339形式の文字列
    mfa_enabled_at: Option<String>,  // RFC 3339形式の文字列
    disabled_at: Option<String>,     // RFC 3339形式の文字列
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

#[derive(Deserialize)]
pub(crate) struct ConsultantIdQuery {
    consultant_id: i64,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConsultationReqsResult {
    consultation_reqs: Vec<ConsultationReq>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct ConsultationReq {
    consultation_req_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    first_candidate_date_time: String,  // RFC 3339形式の文字列
    second_candidate_date_time: String, // RFC 3339形式の文字列
    third_candidate_date_time: String,  // RFC 3339形式の文字列
    latest_candidate_date_time: String, // RFC 3339形式の文字列
    charge_id: String,
    fee_per_hour_in_yen: i32,
    platform_fee_rate_in_percentage: String,
    credit_facilities_expired_at: String, // RFC 3339形式の文字列
}
