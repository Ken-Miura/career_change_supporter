// Copyright 2023 Ken Miura

use chrono::{DateTime, FixedOffset};
use serde::Serialize;

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

fn convert_date_time_to_rfc3339_string(date_time: DateTime<FixedOffset>) -> String {
    date_time.to_rfc3339()
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use common::JAPANESE_TIME_ZONE;

    use super::*;

    #[test]
    fn test_convert_date_time_to_rfc3339_string() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();

        let result = convert_date_time_to_rfc3339_string(current_date_time);

        assert_eq!(current_date_time.to_rfc3339(), result);
    }
}
