// Copyright 2023 Ken Miura

pub(crate) mod agreements_by_user_account_id;
pub(crate) mod career_creation;
pub(crate) mod careers_by_user_account_id;
pub(crate) mod consultation_reqs_by_consultant_id;
pub(crate) mod consultation_reqs_by_user_account_id;
pub(crate) mod consultations_by_consultant_id;
pub(crate) mod consultations_by_user_account_id;
pub(crate) mod disable_mfa_req;
pub(crate) mod disable_user_account_req;
pub(crate) mod enable_user_account_req;
pub(crate) mod fee_per_hour_in_yen_by_user_account_id;
pub(crate) mod identity_creation;
pub(crate) mod identity_option_by_user_account_id;
pub(crate) mod identity_update;
pub(crate) mod rating_info_by_consultant_id;
pub(crate) mod rating_info_by_user_account_id;
pub(crate) mod tenant_id_by_user_account_id;
pub(crate) mod user_account_retrieval_by_email_address;
pub(crate) mod user_account_retrieval_by_user_account_id;

use axum::{http::StatusCode, Json};
use chrono::NaiveDate;
use common::{
    rating::{calculate_average_rating, round_rating_to_one_decimal_places},
    ApiError, ErrResp,
};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

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

// career_idが必要になるため、共通モジュールのCareerは使わない
struct Career {
    career_id: i64,
    user_account_id: i64,
    company_name: String,
    department_name: Option<String>,
    office: Option<String>,
    career_start_date: NaiveDate,
    career_end_date: Option<NaiveDate>,
    contract_type: String,
    profession: Option<String>,
    annual_income_in_man_yen: Option<i32>,
    is_manager: bool,
    position_name: Option<String>,
    is_new_graduate: bool,
    note: Option<String>,
}

async fn get_careers(
    user_account_id: i64,
    pool: &DatabaseConnection,
) -> Result<Vec<Career>, ErrResp> {
    let careers = entity::career::Entity::find()
        .filter(entity::career::Column::UserAccountId.eq(user_account_id))
        .all(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to filter career (user_account_id: {}): {}",
                user_account_id, e
            );
            unexpected_err_resp()
        })?;
    Ok(careers
        .into_iter()
        .map(|m| Career {
            career_id: m.career_id,
            user_account_id: m.user_account_id,
            company_name: m.company_name,
            department_name: m.department_name,
            office: m.office,
            career_start_date: m.career_start_date,
            career_end_date: m.career_end_date,
            contract_type: m.contract_type,
            profession: m.profession,
            annual_income_in_man_yen: m.annual_income_in_man_yen,
            is_manager: m.is_manager,
            position_name: m.position_name,
            is_new_graduate: m.is_new_graduate,
            note: m.note,
        })
        .collect::<Vec<Career>>())
}

async fn get_fee_per_hour_in_yen(
    user_account_id: i64,
    pool: &DatabaseConnection,
) -> Result<Option<i32>, ErrResp> {
    let result = entity::consulting_fee::Entity::find_by_id(user_account_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find consulting_fee (user_account_id: {}): {}",
                user_account_id, e
            );
            unexpected_err_resp()
        })?;
    Ok(result.map(|m| m.fee_per_hour_in_yen))
}

async fn get_tenant_id(
    user_account_id: i64,
    pool: &DatabaseConnection,
) -> Result<Option<String>, ErrResp> {
    let result = entity::tenant::Entity::find_by_id(user_account_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find tenant (user_account_id: {}): {}",
                user_account_id, e
            );
            unexpected_err_resp()
        })?;
    Ok(result.map(|m| m.tenant_id))
}

async fn get_consultant_rating_info(
    consultant_id: i64,
    pool: &DatabaseConnection,
) -> Result<Vec<i16>, ErrResp> {
    let models = entity::consultation::Entity::find()
        .filter(entity::consultation::Column::ConsultantId.eq(consultant_id))
        .find_with_related(entity::consultant_rating::Entity)
        .filter(entity::consultant_rating::Column::Rating.is_not_null())
        .all(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to filter consultant_rating (consultant_id: {}): {}",
                consultant_id, e
            );
            unexpected_err_resp()
        })?;
    models
        .into_iter()
        .map(|m| {
            // consultationとconsultant_ratingは1対1の設計なので取れない場合は想定外エラーとして扱う
            let ur = m.1.get(0).ok_or_else(|| {
                error!(
                    "failed to find consultant_rating (consultation_id: {})",
                    m.0.consultation_id
                );
                unexpected_err_resp()
            })?;
            let r = ur.rating.ok_or_else(|| {
                error!(
                    "rating is null (consultant_rating_id: {}, consultant_id: {})",
                    ur.consultant_rating_id, m.0.consultant_id
                );
                unexpected_err_resp()
            })?;
            Ok(r)
        })
        .collect::<Result<Vec<i16>, ErrResp>>()
}
