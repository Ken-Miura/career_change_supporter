// Copyright 2022 Ken Miura

use std::str::FromStr;

use chrono::{DateTime, Datelike, Duration, FixedOffset, TimeZone};
use common::{ErrResp, JAPANESE_TIME_ZONE};
use entity::{
    consultation,
    prelude::Receipt,
    sea_orm::{EntityTrait, QueryFilter},
};
use entity::{
    receipt,
    sea_orm::{ColumnTrait, DatabaseConnection},
};
use rust_decimal::{prelude::FromPrimitive, Decimal, RoundingStrategy};
use tracing::error;

use crate::err::unexpected_err_resp;

/// 支払いに関する情報を保持する構造体
///
/// [crate::common::payment_platform::charge::Chage]と同じデータを保持するキャッシュとしての役割を持つ
#[derive(Clone, Debug)]
pub(super) struct PaymentInfo {
    pub(super) fee_per_hour_in_yen: i32,
    pub(super) platform_fee_rate_in_percentage: String,
}

pub(super) async fn filter_receipts_of_the_duration_by_consultant_id(
    pool: &DatabaseConnection,
    consultant_id: i64,
    start: &DateTime<FixedOffset>,
    end: &DateTime<FixedOffset>,
) -> Result<Vec<PaymentInfo>, ErrResp> {
    let models = consultation::Entity::find()
        .filter(consultation::Column::ConsultantId.eq(consultant_id))
        .find_with_related(Receipt)
        .filter(receipt::Column::SettledAt.between(*start, *end))
        .all(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to filter receipt (consultant_id: {}, start: {}, end: {}): {}",
                consultant_id, start, end, e
            );
            unexpected_err_resp()
        })?;
    // 正確な報酬額を得るためには取得したレコードに記載されているcharge_idを使い、
    // 一つ一つChageオブジェクトをPAYJPから取得して計算をする必要がある。
    // しかし、PAYJPの流量制限に引っかかりやすくなる危険性を考慮し、レコードのキャシュしてある値を使い報酬を計算する
    models
        .into_iter()
        .map(|m| {
            // consultationとreceiptは1対1の設計なので取れない場合は想定外エラーとして扱う
            let r = m.1.get(0).ok_or_else(|| {
                error!(
                    "failed to find receipt (consultation_id: {})",
                    m.0.consultation_id
                );
                unexpected_err_resp()
            })?;
            Ok(PaymentInfo {
                fee_per_hour_in_yen: r.fee_per_hour_in_yen,
                platform_fee_rate_in_percentage: r.platform_fee_rate_in_percentage.clone(),
            })
        })
        .collect::<Result<Vec<PaymentInfo>, ErrResp>>()
}

// [tenantオブジェクト](https://pay.jp/docs/api/#tenant%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88)のpayjp_fee_includedがtrueであるとことを前提として実装
// payjp_fee_includedの値を設定できるのはテナント作成時のみ。そのためテナント作成時のコードで必ずtrueを設定することで、ここでtrueを前提として処理を行う
pub(super) fn calculate_rewards(payment_info: &[PaymentInfo]) -> Result<i32, ErrResp> {
    let rewards = payment_info.iter().try_fold(0, accumulate_rewards)?;
    Ok(rewards)
}

fn accumulate_rewards(sum: i32, payment_info: &PaymentInfo) -> Result<i32, ErrResp> {
    let sales = payment_info.fee_per_hour_in_yen;
    let fee = calculate_fee(sales, payment_info.platform_fee_rate_in_percentage.as_str())?;
    let reward = sales - fee;
    Ok(sum + reward)
}

// percentageはパーセンテージを示す少数の文字列。feeは、sales * (percentage/100) の結果の少数部分を切り捨てた値。
fn calculate_fee(sales: i32, percentage: &str) -> Result<i32, ErrResp> {
    let percentage_decimal = Decimal::from_str(percentage).map_err(|e| {
        error!("failed to parse percentage ({}): {}", percentage, e);
        unexpected_err_resp()
    })?;
    let one_handred_decimal = Decimal::from_str("100").map_err(|e| {
        error!("failed to parse str literal: {}", e);
        unexpected_err_resp()
    })?;
    let sales_decimal = match Decimal::from_i32(sales) {
        Some(s) => s,
        None => {
            error!("failed to parse sales value ({})", sales);
            return Err(unexpected_err_resp());
        }
    };
    let fee_decimal = (sales_decimal * (percentage_decimal / one_handred_decimal))
        .round_dp_with_strategy(0, RoundingStrategy::ToZero);
    let fee = fee_decimal.to_string().parse::<i32>().map_err(|e| {
        error!("failed to parse fee_decimal ({}): {}", fee_decimal, e);
        unexpected_err_resp()
    })?;
    Ok(fee)
}

/// 渡された日時に対して、その年の日本時間における1月1日0時0分0秒と12月31日23時59分59秒を示す日時を返す。
pub(super) fn create_start_and_end_date_time_of_current_year(
    current_date_time: &DateTime<FixedOffset>,
) -> Result<(DateTime<FixedOffset>, DateTime<FixedOffset>), ErrResp> {
    let current_year = current_date_time.year();

    // 日本のタイムゾーンにおいて、（サマータイム等による）タイムゾーンの遷移は発生しないので一意にならない場合はすべてエラー
    let start = match JAPANESE_TIME_ZONE.with_ymd_and_hms(current_year, 1, 1, 0, 0, 0) {
        chrono::LocalResult::None => {
            error!("failed to get start (current_year: {})", current_year);
            return Err(unexpected_err_resp());
        }
        chrono::LocalResult::Single(s) => s,
        chrono::LocalResult::Ambiguous(a1, a2) => {
            error!(
                "failed to get start (current_year: {}, ambiguous1: {}, ambiguous2: {})",
                current_year, a1, a2
            );
            return Err(unexpected_err_resp());
        }
    };

    // 日本のタイムゾーンにおいて、（サマータイム等による）タイムゾーンの遷移は発生しないので一意にならない場合はすべてエラー
    let end = match JAPANESE_TIME_ZONE.with_ymd_and_hms(current_year, 12, 31, 23, 59, 59) {
        chrono::LocalResult::None => {
            error!("failed to get end (current_year: {})", current_year);
            return Err(unexpected_err_resp());
        }
        chrono::LocalResult::Single(s) => s,
        chrono::LocalResult::Ambiguous(a1, a2) => {
            error!(
                "failed to get end (current_year: {}, ambiguous1: {}, ambiguous2: {})",
                current_year, a1, a2
            );
            return Err(unexpected_err_resp());
        }
    };

    Ok((start, end))
}

/// 渡された日時に対して、その月の日本時間における1日0時0分0秒と最終日23時59分59秒を示す日時を返す。
pub(super) fn create_start_and_end_date_time_of_current_month(
    current_date_time: &DateTime<FixedOffset>,
) -> Result<(DateTime<FixedOffset>, DateTime<FixedOffset>), ErrResp> {
    let current_year = current_date_time.year();
    let current_month = current_date_time.month();

    // 日本のタイムゾーンにおいて、（サマータイム等による）タイムゾーンの遷移は発生しないので一意にならない場合はすべてエラー
    let start = match JAPANESE_TIME_ZONE.with_ymd_and_hms(current_year, current_month, 1, 0, 0, 0) {
        chrono::LocalResult::None => {
            error!(
                "failed to get start (current_year: {}, current_month: {})",
                current_year, current_month
            );
            return Err(unexpected_err_resp());
        }
        chrono::LocalResult::Single(s) => s,
        chrono::LocalResult::Ambiguous(a1, a2) => {
            error!("failed to get start (current_year: {}, current_month: {}, ambiguous1: {}, ambiguous2: {})", current_year, current_month, a1 ,a2);
            return Err(unexpected_err_resp());
        }
    };

    let (year, month) = if current_month >= 12 {
        (current_year + 1, 1)
    } else {
        (current_year, current_month + 1)
    };

    // 日本のタイムゾーンにおいて、（サマータイム等による）タイムゾーンの遷移は発生しないので一意にならない場合はすべてエラー
    let end = match JAPANESE_TIME_ZONE.with_ymd_and_hms(year, month, 1, 23, 59, 59) {
        chrono::LocalResult::None => {
            error!("failed to get end (year: {}, month: {})", year, month);
            return Err(unexpected_err_resp());
        }
        chrono::LocalResult::Single(s) => s,
        chrono::LocalResult::Ambiguous(a1, a2) => {
            error!(
                "failed to get end (year: {}, month: {}, ambiguous1: {}, ambiguous2: {})",
                year, month, a1, a2
            );
            return Err(unexpected_err_resp());
        }
    } - Duration::days(1);

    Ok((start, end))
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;
    use common::JAPANESE_TIME_ZONE;
    use once_cell::sync::Lazy;

    use crate::handlers::session::authentication::authenticated_handlers::rewards_info::{
        create_start_and_end_date_time_of_current_month,
        create_start_and_end_date_time_of_current_year,
    };

    use super::{calculate_rewards, PaymentInfo};

    #[derive(Debug)]
    struct CalculateRewardsTestCase {
        name: String,
        input: Vec<PaymentInfo>,
        expected: i32,
    }

    static CALCULATE_REWARDS_TEST_CASE_SET: Lazy<Vec<CalculateRewardsTestCase>> = Lazy::new(|| {
        vec![
            CalculateRewardsTestCase {
                name: "1 payment".to_string(),
                input: vec![PaymentInfo {
                    fee_per_hour_in_yen: 5000,
                    platform_fee_rate_in_percentage: "30.0".to_string(),
                }],
                expected: 3500,
            },
            CalculateRewardsTestCase {
                name: "2 payments".to_string(),
                input: vec![
                    PaymentInfo {
                        fee_per_hour_in_yen: 5000,
                        platform_fee_rate_in_percentage: "30.0".to_string(),
                    },
                    PaymentInfo {
                        fee_per_hour_in_yen: 4000,
                        platform_fee_rate_in_percentage: "30.0".to_string(),
                    },
                ],
                expected: 3500 + 2800,
            },
            CalculateRewardsTestCase {
                name: "3 payments".to_string(),
                input: vec![
                    PaymentInfo {
                        fee_per_hour_in_yen: 5000,
                        platform_fee_rate_in_percentage: "30.0".to_string(),
                    },
                    PaymentInfo {
                        fee_per_hour_in_yen: 4000,
                        platform_fee_rate_in_percentage: "30.0".to_string(),
                    },
                    PaymentInfo {
                        fee_per_hour_in_yen: 3000,
                        platform_fee_rate_in_percentage: "30.0".to_string(),
                    },
                ],
                expected: 3500 + 2800 + 2100,
            },
            CalculateRewardsTestCase {
                name: "decimal number in fee round to zero case 1".to_string(),
                input: vec![PaymentInfo {
                    fee_per_hour_in_yen: 5003,
                    platform_fee_rate_in_percentage: "30.0".to_string(),
                }],
                expected: 3503,
            },
            CalculateRewardsTestCase {
                name: "decimal number in fee round to zero case 2".to_string(),
                input: vec![PaymentInfo {
                    fee_per_hour_in_yen: 4008,
                    platform_fee_rate_in_percentage: "30.0".to_string(),
                }],
                expected: 2806,
            },
            CalculateRewardsTestCase {
                name: "decimal number in fee round to zero case 3".to_string(),
                input: vec![
                    PaymentInfo {
                        fee_per_hour_in_yen: 5003,
                        platform_fee_rate_in_percentage: "30.0".to_string(),
                    },
                    PaymentInfo {
                        fee_per_hour_in_yen: 4008,
                        platform_fee_rate_in_percentage: "30.0".to_string(),
                    },
                ],
                expected: 3503 + 2806,
            },
        ]
    });

    #[test]
    fn test_calculate_rewards() {
        for test_case in CALCULATE_REWARDS_TEST_CASE_SET.iter() {
            let result = calculate_rewards(&test_case.input).expect("failed to get Ok");
            let message = format!("test case \"{}\" failed", test_case.name.clone());
            assert_eq!(result, test_case.expected, "{}", message);
        }
    }

    #[test]
    fn test_case_normal_year_create_start_and_end_date_time_of_current_year() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 7, 15, 15, 43, 39)
            .unwrap();
        let (start, end) = create_start_and_end_date_time_of_current_year(&current_date_time)
            .expect("failed to get Ok");
        assert_eq!(
            JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2022, 1, 1, 0, 0, 0)
                .unwrap(),
            start
        );
        assert_eq!(
            JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2022, 12, 31, 23, 59, 59)
                .unwrap(),
            end
        );
    }

    #[test]
    fn test_case_leap_year_create_start_and_end_date_time_of_current_year() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2020, 12, 15, 15, 43, 39)
            .unwrap();
        let (start, end) = create_start_and_end_date_time_of_current_year(&current_date_time)
            .expect("failed to get Ok");
        assert_eq!(
            JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2020, 1, 1, 0, 0, 0)
                .unwrap(),
            start
        );
        assert_eq!(
            JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2020, 12, 31, 23, 59, 59)
                .unwrap(),
            end
        );
    }

    #[test]
    fn test_case_normal_month_create_start_and_end_date_time_of_current_month() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 7, 15, 15, 43, 39)
            .unwrap();
        let (start, end) = create_start_and_end_date_time_of_current_month(&current_date_time)
            .expect("failed to get Ok");
        assert_eq!(
            JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2022, 7, 1, 0, 0, 0)
                .unwrap(),
            start
        );
        assert_eq!(
            JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2022, 7, 31, 23, 59, 59)
                .unwrap(),
            end
        );
    }

    #[test]
    fn test_case_feb_in_normal_year_create_start_and_end_date_time_of_current_month() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2021, 2, 15, 15, 43, 39)
            .unwrap();
        let (start, end) = create_start_and_end_date_time_of_current_month(&current_date_time)
            .expect("failed to get Ok");
        assert_eq!(
            JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2021, 2, 1, 0, 0, 0)
                .unwrap(),
            start
        );
        assert_eq!(
            JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2021, 2, 28, 23, 59, 59)
                .unwrap(),
            end
        );
    }

    #[test]
    fn test_case_feb_in_leap_year_create_start_and_end_date_time_of_current_month() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2020, 2, 15, 15, 43, 39)
            .unwrap();
        let (start, end) = create_start_and_end_date_time_of_current_month(&current_date_time)
            .expect("failed to get Ok");
        assert_eq!(
            JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2020, 2, 1, 0, 0, 0)
                .unwrap(),
            start
        );
        assert_eq!(
            JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2020, 2, 29, 23, 59, 59)
                .unwrap(),
            end
        );
    }

    #[test]
    fn test_case_dec_create_start_and_end_date_time_of_current_month() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 12, 15, 15, 43, 39)
            .unwrap();
        let (start, end) = create_start_and_end_date_time_of_current_month(&current_date_time)
            .expect("failed to get Ok");
        assert_eq!(
            JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2022, 12, 1, 0, 0, 0)
                .unwrap(),
            start
        );
        assert_eq!(
            JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2022, 12, 31, 23, 59, 59)
                .unwrap(),
            end
        );
    }
}
