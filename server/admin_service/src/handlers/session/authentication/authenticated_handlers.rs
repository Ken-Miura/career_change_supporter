// Copyright 2023 Ken Miura

use std::str::FromStr;

use axum::{http::StatusCode, Json};
use chrono::{DateTime, Datelike, FixedOffset, Timelike};
use common::{
    util::{Identity, Ymd},
    ApiError, ErrResp, ErrRespStruct,
};
use entity::sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QuerySelect, Set, TransactionError, TransactionTrait,
};
use once_cell::sync::Lazy;
use rust_decimal::{prelude::FromPrimitive, Decimal, RoundingStrategy};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

pub(crate) mod admin;
pub(crate) mod awaiting_payment;
pub(crate) mod awaiting_withdrawal;
pub(crate) mod career_request;
pub(crate) mod consultation;
mod document_operation;
pub(crate) mod identity_by_user_account_id;
pub(crate) mod identity_request;
pub(crate) mod maintenance;
pub(crate) mod neglected_payment;
pub(crate) mod news;
pub(crate) mod pagination;
mod reason_validator;
pub(crate) mod refresh;
pub(crate) mod refunded_payment;
pub(crate) mod user_account;
mod user_account_operation;

/// コンサルタントへ報酬を振り込むまで待機する期間（単位：日）
const WAITING_PERIOD_BEFORE_WITHDRAWAL_TO_CONSULTANT_IN_DAYS: i64 = 8;

pub(crate) const KEY_TO_TRANSFER_FEE_IN_YEN: &str = "TRANSFER_FEE_IN_YEN";
static TRANSFER_FEE_IN_YEN: Lazy<i32> = Lazy::new(|| {
    let transfer_fee_in_yen = std::env::var(KEY_TO_TRANSFER_FEE_IN_YEN).unwrap_or_else(|_| {
        // 単体テスト実行時に環境変数がないため、初期値として記載しておく。
        // サービスとして起動するときは環境変数を記載することは必須。
        "250".to_string()
    });
    transfer_fee_in_yen
        .parse()
        .expect("failed to parse TRANSFER_FEE_IN_YEN")
});

pub(crate) const KEY_TO_PLATFORM_FEE_RATE_IN_PERCENTAGE: &str = "PLATFORM_FEE_RATE_IN_PERCENTAGE";
static PLATFORM_FEE_RATE_IN_PERCENTAGE: Lazy<String> = Lazy::new(|| {
    std::env::var(KEY_TO_PLATFORM_FEE_RATE_IN_PERCENTAGE).unwrap_or_else(|_| {
        // 単体テスト実行時に環境変数がないため、初期値として記載しておく。
        // サービスとして起動するときは環境変数を記載することは必須。
        "50.0".to_string()
    })
});

fn calculate_reward(
    sale_in_yen: i32,
    platform_fee_rate_in_percentage: &str,
    transfer_fee_in_yen: i32,
) -> Result<i32, ErrResp> {
    let platform_fee_in_yen =
        calculate_platform_fee_in_yen(sale_in_yen, platform_fee_rate_in_percentage)?;
    let reward = sale_in_yen - platform_fee_in_yen - transfer_fee_in_yen;
    Ok(reward)
}

// platform_fee_rate_in_percentageはパーセンテージを示す少数の文字列。返り値は、sale_in_yen * (platform_fee_rate_in_percentage/100) の結果の少数部分を切り捨てた値。
fn calculate_platform_fee_in_yen(
    sale_in_yen: i32,
    platform_fee_rate_in_percentage: &str,
) -> Result<i32, ErrResp> {
    let platform_fee_rate_in_percentage_decimal =
        Decimal::from_str(platform_fee_rate_in_percentage).map_err(|e| {
            error!(
                "failed to parse platform_fee_rate_in_percentage ({}): {}",
                platform_fee_rate_in_percentage, e
            );
            unexpected_err_resp()
        })?;
    let one_handred_decimal = Decimal::from_str("100").map_err(|e| {
        error!("failed to parse str literal: {}", e);
        unexpected_err_resp()
    })?;
    let sale_in_yen_decimal = match Decimal::from_i32(sale_in_yen) {
        Some(s) => s,
        None => {
            error!("failed to parse sale_in_yen value ({})", sale_in_yen);
            return Err(unexpected_err_resp());
        }
    };
    let platform_fee_in_yen_decimal = (sale_in_yen_decimal
        * (platform_fee_rate_in_percentage_decimal / one_handred_decimal))
        .round_dp_with_strategy(0, RoundingStrategy::ToZero);
    let platform_fee_in_yen = platform_fee_in_yen_decimal
        .to_string()
        .parse::<i32>()
        .map_err(|e| {
            error!(
                "failed to parse platform_fee_in_yen_decimal ({}): {}",
                platform_fee_in_yen_decimal, e
            );
            unexpected_err_resp()
        })?;
    Ok(platform_fee_in_yen)
}

#[derive(Deserialize)]
pub(crate) struct ConsultationIdQuery {
    consultation_id: i64,
}

#[derive(Deserialize)]
pub(crate) struct ConsultationIdBody {
    consultation_id: i64,
}

fn validate_consultation_id_is_positive(consultation_id: i64) -> Result<(), ErrResp> {
    if !consultation_id.is_positive() {
        error!("consultation_id is not positive: {}", consultation_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultationIdIsNotPositive as u32,
            }),
        ));
    }
    Ok(())
}

async fn find_awaiting_payment_with_exclusive_lock(
    consultation_id: i64,
    txn: &DatabaseTransaction,
) -> Result<Option<entity::awaiting_payment::Model>, ErrRespStruct> {
    let model_option = entity::awaiting_payment::Entity::find_by_id(consultation_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find awaiting_payment (consultation_id: {}): {}",
                consultation_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(model_option)
}

async fn delete_awaiting_payment(
    consultation_id: i64,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    let _ = entity::awaiting_payment::Entity::delete_by_id(consultation_id)
        .exec(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to delete awaiting_payment (consultation_id: {}): {}",
                consultation_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(())
}

async fn find_awaiting_withdrawal_with_exclusive_lock(
    consultation_id: i64,
    txn: &DatabaseTransaction,
) -> Result<Option<entity::awaiting_withdrawal::Model>, ErrRespStruct> {
    let model_option = entity::awaiting_withdrawal::Entity::find_by_id(consultation_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find awaiting_withdrawal (consultation_id: {}): {}",
                consultation_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(model_option)
}

async fn delete_awaiting_withdrawal(
    consultation_id: i64,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    let _ = entity::awaiting_withdrawal::Entity::delete_by_id(consultation_id)
        .exec(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to delete awaiting_withdrawal (consultation_id: {}): {}",
                consultation_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(())
}

async fn find_identity_by_user_account_id(
    pool: &DatabaseConnection,
    user_account_id: i64,
) -> Result<Option<Identity>, ErrResp> {
    let model = entity::identity::Entity::find_by_id(user_account_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find identity (user_account_id: {}): {}",
                user_account_id, e
            );
            unexpected_err_resp()
        })?;
    Ok(model.map(convert_identity))
}

fn convert_identity(identity_model: entity::identity::Model) -> Identity {
    let date = identity_model.date_of_birth;
    let ymd = Ymd {
        year: date.year(),
        month: date.month(),
        day: date.day(),
    };
    Identity {
        last_name: identity_model.last_name,
        first_name: identity_model.first_name,
        last_name_furigana: identity_model.last_name_furigana,
        first_name_furigana: identity_model.first_name_furigana,
        date_of_birth: ymd,
        prefecture: identity_model.prefecture,
        city: identity_model.city,
        address_line1: identity_model.address_line1,
        address_line2: identity_model.address_line2,
        telephone_number: identity_model.telephone_number,
    }
}

async fn find_identity_by_user_account_id_in_transaction(
    txn: &DatabaseTransaction,
    user_account_id: i64,
) -> Result<Option<Identity>, ErrRespStruct> {
    let model = entity::identity::Entity::find_by_id(user_account_id)
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find identity (user_account_id: {}): {}",
                user_account_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(model.map(convert_identity))
}

fn generate_sender_name(
    last_name_furigana: String,
    first_name_furigana: String,
    meeting_at: DateTime<FixedOffset>,
) -> Result<String, ErrResp> {
    let name = format!("{}　{}", last_name_furigana, first_name_furigana);
    let suffix = generate_suffix(meeting_at)?;
    Ok(format!("{}　{}", name, suffix))
}

fn generate_suffix(meeting_at: DateTime<FixedOffset>) -> Result<String, ErrResp> {
    let suffix: Vec<char> = format!(
        "{:0>2}{:0>2}{:0>2}",
        meeting_at.month(),
        meeting_at.day(),
        meeting_at.hour()
    )
    .chars()
    .collect();

    suffix
        .into_iter()
        .map(|c| match c {
            '0' => Ok('０'),
            '1' => Ok('１'),
            '2' => Ok('２'),
            '3' => Ok('３'),
            '4' => Ok('４'),
            '5' => Ok('５'),
            '6' => Ok('６'),
            '7' => Ok('７'),
            '8' => Ok('８'),
            '9' => Ok('９'),
            _ => {
                error!("not a number ({})", c);
                Err(unexpected_err_resp())
            }
        })
        .collect()
}

fn convert_date_time_to_rfc3339_string(date_time: DateTime<FixedOffset>) -> String {
    date_time.to_rfc3339()
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

async fn move_to_stopped_settlement(
    pool: &DatabaseConnection,
    settlement_id: i64,
    current_date_time: DateTime<FixedOffset>,
) -> Result<(), ErrResp> {
    pool.transaction::<_, (), ErrRespStruct>(|txn| {
        Box::pin(async move {
            let s = find_settlement_with_exclusive_lock(settlement_id, txn).await?;

            let ss = entity::stopped_settlement::ActiveModel {
                stopped_settlement_id: NotSet,
                consultation_id: Set(s.consultation_id),
                charge_id: Set(s.charge_id.clone()),
                fee_per_hour_in_yen: Set(s.fee_per_hour_in_yen),
                platform_fee_rate_in_percentage: Set(s.platform_fee_rate_in_percentage.clone()),
                credit_facilities_expired_at: Set(s.credit_facilities_expired_at),
                stopped_at: Set(current_date_time),
            };
            let _ = ss.insert(txn).await.map_err(|e| {
                error!(
                    "failed to insert stopped_settlement (settlement: {:?}): {}",
                    s, e,
                );
                ErrRespStruct {
                    err_resp: unexpected_err_resp(),
                }
            })?;

            let _ = entity::settlement::Entity::delete_by_id(settlement_id)
                .exec(txn)
                .await
                .map_err(|e| {
                    error!(
                        "failed to delete settlement (settlement_id: {}): {}",
                        settlement_id, e,
                    );
                    ErrRespStruct {
                        err_resp: unexpected_err_resp(),
                    }
                })?;

            Ok(())
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(db_err) => {
            error!("connection error: {}", db_err);
            unexpected_err_resp()
        }
        TransactionError::Transaction(err_resp_struct) => {
            error!("failed to move_to_stopped_settlement: {}", err_resp_struct);
            err_resp_struct.err_resp
        }
    })?;
    Ok(())
}

async fn find_settlement_with_exclusive_lock(
    settlement_id: i64,
    txn: &DatabaseTransaction,
) -> Result<entity::settlement::Model, ErrRespStruct> {
    let result = entity::settlement::Entity::find_by_id(settlement_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find settlement (settlement_id: {}): {}",
                settlement_id, e,
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    let model = result.ok_or_else(|| {
        error!("no settlement (settlement_id: {}) found", settlement_id,);
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(model)
}

#[cfg(test)]
pub(super) mod tests {

    use axum::async_trait;

    use chrono::TimeZone;
    use common::{smtp::SendMail, ErrResp, JAPANESE_TIME_ZONE};

    use super::*;

    pub(super) struct SendMailMock {
        to: String,
        from: String,
        subject: String,
        text: String,
    }

    impl SendMailMock {
        pub(super) fn new(to: String, from: String, subject: String, text: String) -> Self {
            Self {
                to,
                from,
                subject,
                text,
            }
        }
    }

    #[async_trait]
    impl SendMail for SendMailMock {
        async fn send_mail(
            &self,
            to: &str,
            from: &str,
            subject: &str,
            text: &str,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.to, to);
            assert_eq!(self.from, from);
            assert_eq!(self.subject, subject);
            assert_eq!(self.text, text);
            Ok(())
        }
    }

    #[test]
    fn test_generate_sender_name_case1() {
        let last_name_furigana = "スズキ".to_string();
        let first_name_furigana = "ジロウ".to_string();
        let meeting_at = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 11, 15, 18, 0, 0)
            .unwrap();

        let result = generate_sender_name(last_name_furigana, first_name_furigana, meeting_at)
            .expect("failed to get Ok");

        assert_eq!("スズキ　ジロウ　１１１５１８", result);
    }

    #[test]
    fn test_generate_sender_name_case2() {
        let last_name_furigana = "タナカ".to_string();
        let first_name_furigana = "タロウ".to_string();
        let meeting_at = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 8, 0, 0)
            .unwrap();

        let result = generate_sender_name(last_name_furigana, first_name_furigana, meeting_at)
            .expect("failed to get Ok");

        assert_eq!("タナカ　タロウ　０９０５０８", result);
    }

    #[test]
    fn test_convert_date_time_to_rfc3339_string() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();

        let result = convert_date_time_to_rfc3339_string(current_date_time);

        assert_eq!(current_date_time.to_rfc3339(), result);
    }

    #[test]
    fn test_calculate_reward_case1() {
        let fee = 3000;
        let platform_fee_rate_in_percentage = "50.0";
        let transfer_fee = 250;

        let result = calculate_reward(fee, platform_fee_rate_in_percentage, transfer_fee)
            .expect("failed to get Ok");

        assert_eq!(1250, result);
    }

    #[test]
    fn test_calculate_reward_case2() {
        let fee = 3001;
        let platform_fee_rate_in_percentage = "50.0";
        let transfer_fee = 250;

        let result = calculate_reward(fee, platform_fee_rate_in_percentage, transfer_fee)
            .expect("failed to get Ok");

        assert_eq!(1251, result);
    }

    #[test]
    fn test_calculate_reward_case3() {
        let fee = 3004;
        let platform_fee_rate_in_percentage = "60.0";
        let transfer_fee = 250;

        let result = calculate_reward(fee, platform_fee_rate_in_percentage, transfer_fee)
            .expect("failed to get Ok");

        assert_eq!(952, result);
    }
}
