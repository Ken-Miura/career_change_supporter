// Copyright 2022 Ken Miura

use std::env;

use once_cell::sync::Lazy;

const KEY_TO_MAX_ANNUAL_REWARDS_IN_YEN: &str = "MAX_ANNUAL_REWARDS_IN_YEN";
/// 年間で稼ぐことが可能な最大報酬額（単位：円）
///
/// 動作確認時の利便性のために環境変数をセットする選択肢を用意しているただけで、原則、環境変数をセットせず、デフォルト値を用いる。
pub(super) static MAX_ANNUAL_REWARDS_IN_YEN: Lazy<i32> = Lazy::new(|| {
    let max_annual_rewards =
        env::var(KEY_TO_MAX_ANNUAL_REWARDS_IN_YEN).unwrap_or_else(|_| "470000".to_string());
    let max_annual_rewards = max_annual_rewards
        .parse()
        .expect("failed to parse MAX_ANNUAL_REWARDS_IN_YEN");
    if max_annual_rewards <= 0 {
        panic!(
            "MAX_ANNUAL_REWARDS_IN_YEN must be positive: {}",
            max_annual_rewards
        );
    }
    max_annual_rewards
});

const KEY_TO_MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS: &str =
    "MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS";
/// 相談者が相談依頼を行った日時を起点とし、相談開始日時までの秒単位での最小期間
///
/// 動作確認時に待機時間を減らすために環境変数をセットする選択肢を用意しているただけで、原則、環境変数をセットせず、デフォルト値を用いる。
pub(super) static MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS: Lazy<i64> = Lazy::new(|| {
    let min_duration_in_seconds = env::var(KEY_TO_MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS)
        .unwrap_or_else(|_| {
            "259200".to_string() // 3 days
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
    min_duration_in_seconds
});

const KEY_TO_MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS: &str =
    "MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS";
/// 相談者が相談依頼を行った日時を起点とし、相談開始日時までの秒単位での最大期間
///
/// 動作確認時に待機時間を減らすために環境変数をセットする選択肢を用意しているただけで、原則、環境変数をセットせず、デフォルト値を用いる。
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

const KEY_TO_EXPIRY_DAYS_OF_CHARGE: &str = "EXPIRY_DAYS_OF_CHARGE";
/// 相談者が相談依頼を行った日時を起点とし、決済の認証が切れるまでの有効期限（単位：日）
///
/// 動作確認時の利便性のために環境変数をセットする選択肢を用意しているただけで、原則、環境変数をセットせず、デフォルト値を用いる。
pub(super) static EXPIRY_DAYS_OF_CHARGE: Lazy<u32> = Lazy::new(|| {
    let expiry_days_of_charge =
        env::var(KEY_TO_EXPIRY_DAYS_OF_CHARGE).unwrap_or_else(|_| "59".to_string());
    let expiry_days_of_charge = expiry_days_of_charge
        .parse()
        .expect("failed to parse EXPIRY_DAYS_OF_CHARGE");
    // https://pay.jp/docs/api/#%E6%94%AF%E6%89%95%E3%81%84%E3%82%92%E4%BD%9C%E6%88%90
    // APIドキュメントでは60まで許容されているが、60を指定したときの挙動が奇妙なので59までしか使わないようにする
    if !(1..=59).contains(&expiry_days_of_charge) {
        panic!(
            "EXPIRY_DAYS_OF_CHARGE ({}) must be between 1 and 59",
            expiry_days_of_charge
        );
    };
    let expiry_days_of_charge_in_seconds = expiry_days_of_charge as i64 * 24 * 60 * 60;
    // TODO: 相談終了後、相談者が相談相手の評価をせず、自動決済の対象となる期間も考慮した制約として書き直す。
    if expiry_days_of_charge_in_seconds < *MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS {
        panic!(
            "EXPIRY_DAYS_OF_CHARGE in seconds ({}) must be MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS ({}) or more",
            expiry_days_of_charge, *MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS
        );
    };
    expiry_days_of_charge
});

const KEY_TO_MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE: &str =
    "MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE";
/// 受け付けた相談を承認する際、相談開始日時までに空いていなければならない最小期間（単位：時間）
///
/// 動作確認時に待機時間を減らすために環境変数をセットする選択肢を用意しているただけで、原則、環境変数をセットせず、デフォルト値を用いる。
pub(super) static MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE: Lazy<u32> =
    Lazy::new(|| {
        let min_duration_in_hour =
            env::var(KEY_TO_MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE)
                .unwrap_or_else(|_| "6".to_string());
        min_duration_in_hour
            .parse()
            .expect("failed to parse MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE")
    });

const KEY_TO_FIRST_START_HOUR_OF_CONSULTATION: &str = "FIRST_START_HOUR_OF_CONSULTATION";
/// 1日の内、最も早い相談開始時刻
///
/// 動作確認時に待機時間を減らすために環境変数をセットする選択肢を用意しているただけで、原則、環境変数をセットせず、デフォルト値を用いる。
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
///
/// 動作確認時に待機時間を減らすために環境変数をセットする選択肢を用意しているただけで、原則、環境変数をセットせず、デフォルト値を用いる。
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
