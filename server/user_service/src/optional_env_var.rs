// Copyright 2022 Ken Miura
//! デバッグ時に利便性のために用意したオプションの環境変数の集合
//!
//! 本番環境ではこのモジュールの環境変数を使ってはいけない

use std::env;

use once_cell::sync::Lazy;

const KEY_TO_MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS: &str =
    "MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS";
/// 相談者が相談依頼を行った日時を起点とし、相談開始日時までの秒単位での最小期間
pub(super) static MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS: Lazy<i64> = Lazy::new(|| {
    let min_duration_in_seconds = env::var(KEY_TO_MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS)
        .unwrap_or_else(|_| {
            "864000".to_string() // 10 days
        });
    let min_duration_in_seconds = min_duration_in_seconds
        .parse()
        .expect("failed to parse MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS");
    if min_duration_in_seconds < 0 {
        panic!(
            "MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS must be 0 or positive ({})",
            min_duration_in_seconds
        );
    };
    if min_duration_in_seconds < *MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64 {
        panic!(
            "MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS must be MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS ({}) or more ({})",
            *MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS, min_duration_in_seconds
        );
    }
    min_duration_in_seconds
});

const KEY_TO_MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS: &str =
    "MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS";
/// 相談者が相談依頼を行った日時を起点とし、相談開始日時までの秒単位での最大期間
pub(super) static MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS: Lazy<i64> = Lazy::new(|| {
    let max_duration_in_seconds = env::var(KEY_TO_MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS)
        .unwrap_or_else(|_| {
            "1814400".to_string() // 21 days
        });
    let max_duration_in_seconds = max_duration_in_seconds
        .parse()
        .expect("failed to parse MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS");
    if max_duration_in_seconds <= *MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS {
        panic!("MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS ({}) must be more than MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS ({})", 
            max_duration_in_seconds, *MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS);
    };
    max_duration_in_seconds
});

const KEY_TO_MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS: &str =
    "MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS";
/// 受け付けた相談を承認する際、相談開始日時までに空いていなければならない最小期間（単位：秒）
pub(super) static MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS: Lazy<u32> =
    Lazy::new(|| {
        let min_duration_in_hour =
            env::var(KEY_TO_MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS).unwrap_or_else(
                |_| common::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS.to_string(),
            ); // 7日間
        min_duration_in_hour
            .parse()
            .expect("failed to parse MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS")
    });

const KEY_TO_FIRST_START_HOUR_OF_CONSULTATION: &str = "FIRST_START_HOUR_OF_CONSULTATION";
/// 1日の内、最も早い相談開始時刻
pub(super) static FIRST_START_HOUR_OF_CONSULTATION: Lazy<u32> = Lazy::new(|| {
    let first_start_hour =
        env::var(KEY_TO_FIRST_START_HOUR_OF_CONSULTATION).unwrap_or_else(|_| "7".to_string());
    let first_start_hour = first_start_hour
        .parse()
        .expect("failed to parse FIRST_START_HOUR_OF_CONSULTATION");
    if !(0..=23).contains(&first_start_hour) {
        panic!(
            "FIRST_START_HOUR_OF_CONSULTATION must be between 0 to 23: {}",
            first_start_hour
        );
    };
    first_start_hour
});

const KEY_TO_LAST_START_HOUR_OF_CONSULTATION: &str = "LAST_START_HOUR_OF_CONSULTATION";
/// 1日の内、最も遅い相談開始時刻
pub(super) static LAST_START_HOUR_OF_CONSULTATION: Lazy<u32> = Lazy::new(|| {
    let last_start_hour =
        env::var(KEY_TO_LAST_START_HOUR_OF_CONSULTATION).unwrap_or_else(|_| "23".to_string());
    let last_start_hour = last_start_hour
        .parse()
        .expect("failed to parse LAST_START_HOUR_OF_CONSULTATION");
    if !(0..=23).contains(&last_start_hour) {
        panic!(
            "LAST_START_HOUR_OF_CONSULTATION must be between 0 to 23: {}",
            last_start_hour
        );
    };
    if last_start_hour <= *FIRST_START_HOUR_OF_CONSULTATION {
        panic!("LAST_START_HOUR_OF_CONSULTATION ({}) must be more than FIRST_START_HOUR_OF_CONSULTATION ({})", last_start_hour, *FIRST_START_HOUR_OF_CONSULTATION);
    };
    last_start_hour
});

const KEY_TO_CHECK_IF_CONSULTATION_ROOM_IS_OPENED: &str = "CHECK_IF_CONSULTATION_ROOM_IS_OPENED";
/// 相談室に入室する際、現在時刻が入室可能な範囲の時刻かどうか（＝相談室が開いているかどうか）
pub(super) static CHECK_IF_CONSULTATION_ROOM_IS_OPENED: Lazy<bool> = Lazy::new(|| {
    let check_if_consultation_room_is_opened =
        env::var(KEY_TO_CHECK_IF_CONSULTATION_ROOM_IS_OPENED)
            .unwrap_or_else(|_| "true".to_string());

    check_if_consultation_room_is_opened
        .parse()
        .expect("failed to parse KEY_TO_CHECK_IF_CONSULTATION_ROOM_IS_OPENED")
});
