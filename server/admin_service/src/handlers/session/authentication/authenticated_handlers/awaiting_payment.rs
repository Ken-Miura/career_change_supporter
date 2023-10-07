// Copyright 2023 Ken Miura

use chrono::{DateTime, Datelike, FixedOffset, Timelike};
use common::ErrResp;
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
    sender_name: Option<String>,
}

fn convertDateTimeToRfc3339String(date_time: DateTime<FixedOffset>) -> String {
    date_time.to_rfc3339()
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

#[cfg(test)]
mod tests {
    use super::*;
}
