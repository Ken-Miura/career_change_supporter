// Copyright 2023 Ken Miura

pub(crate) mod agreements_by_user_account_id;
pub(crate) mod careers_by_user_account_id;
pub(crate) mod consultation_reqs_by_consultant_id;
pub(crate) mod consultation_reqs_by_user_account_id;
pub(crate) mod consultations_by_consultant_id;
pub(crate) mod consultations_by_user_account_id;
pub(crate) mod fee_per_hour_in_yen_by_user_account_id;
pub(crate) mod identity_option_by_user_account_id;
pub(crate) mod rating_info_by_user_account_id;
pub(crate) mod tenant_id_by_user_account_id;
pub(crate) mod user_account_retrieval_by_email_address;
pub(crate) mod user_account_retrieval_by_user_account_id;

use axum::{http::StatusCode, Json};
use common::{
    rating::{calculate_average_rating, round_rating_to_one_decimal_places},
    ApiError, ErrResp,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::Code;

#[derive(Deserialize)]
pub(crate) struct UserAccountIdQuery {
    user_account_id: i64,
}

#[derive(Deserialize)]
pub(crate) struct ConsultantIdQuery {
    consultant_id: i64,
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

fn validate_account_id_is_positive(account_id: i64) -> Result<(), ErrResp> {
    if !account_id.is_positive() {
        error!("account_id is not positive: {}", account_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::AccountIdIsNotPositive as u32,
            }),
        ));
    }
    Ok(())
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

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConsultationsResult {
    consultations: Vec<Consultation>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct Consultation {
    consultation_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    meeting_at: String, // RFC 3339形式の文字列
    room_name: String,
    user_account_entered_at: Option<String>, // RFC 3339形式の文字列
    consultant_entered_at: Option<String>,   // RFC 3339形式の文字列
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct RatingInfoResult {
    average_rating: Option<String>,
    count: i32,
}

fn calculate_rating_and_count(ratings: Vec<i16>) -> (Option<String>, i32) {
    let count = ratings.len();
    let rating_option = calculate_average_rating(ratings);
    if let Some(rating) = rating_option {
        let rating_str = round_rating_to_one_decimal_places(rating);
        (Some(rating_str), count as i32)
    } else {
        (None, 0)
    }
}
