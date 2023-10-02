// Copyright 2023 Ken Miura

use chrono::{DateTime, FixedOffset};
use common::ErrResp;
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;

pub(crate) mod expired_list;
pub(crate) mod list;

#[derive(Clone, Serialize, Debug, PartialEq)]
struct AwaitingPayment {
    consultation_id: i64,
    consultant_id: i64,
    user_account_id: i64,
    meeting_at: String, // RFC 3339形式の文字列
    fee_per_hour_in_yen: i32,
    sender_name: String,
    sender_name_suffix: String,
}

#[derive(Clone)]
struct AwaitingPaymentModel {
    consultation_id: i64,
    consultant_id: i64,
    user_account_id: i64,
    meeting_at: DateTime<FixedOffset>,
    fee_per_hour_in_yen: i32,
}

#[derive(Clone)]
struct Name {
    last_name_furigana: String,
    first_name_furigana: String,
}

async fn find_name_by_user_account_id(
    pool: &DatabaseConnection,
    user_account_id: i64,
) -> Result<Name, ErrResp> {
    let id = entity::identity::Entity::find_by_id(user_account_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find identity (user_account_id: {}): {}",
                user_account_id, e
            );
            unexpected_err_resp()
        })?;
    let id = id.ok_or_else(|| {
        error!("no identity (user_account_id: {}) found", user_account_id);
        unexpected_err_resp()
    })?;
    Ok(Name {
        first_name_furigana: id.first_name_furigana,
        last_name_furigana: id.last_name_furigana,
    })
}
