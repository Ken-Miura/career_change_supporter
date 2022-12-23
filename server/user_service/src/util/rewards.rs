// Copyright 2022 Ken Miura

use std::str::FromStr;

use chrono::{DateTime, Datelike, Duration, FixedOffset, TimeZone};
use common::{ErrResp, JAPANESE_TIME_ZONE};
use entity::{
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
pub(crate) struct PaymentInfo {
    pub(crate) fee_per_hour_in_yen: i32,
    pub(crate) platform_fee_rate_in_percentage: String,
}

pub(crate) async fn filter_receipts_of_the_duration_by_consultant_id(
    pool: &DatabaseConnection,
    consultant_id: i64,
    start: &DateTime<FixedOffset>,
    end: &DateTime<FixedOffset>,
) -> Result<Vec<PaymentInfo>, ErrResp> {
    let models = Receipt::find()
        .filter(receipt::Column::ConsultantId.eq(consultant_id))
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
    Ok(models
        .into_iter()
        .map(|m| PaymentInfo {
            fee_per_hour_in_yen: m.fee_per_hour_in_yen,
            platform_fee_rate_in_percentage: m.platform_fee_rate_in_percentage,
        })
        .collect::<Vec<PaymentInfo>>())
}

// [tenantオブジェクト](https://pay.jp/docs/api/#tenant%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88)のpayjp_fee_includedがtrueであるとことを前提として実装
// payjp_fee_includedの値を設定できるのはテナント作成時のみ。そのためテナント作成時のコードで必ずtrueを設定することで、ここでtrueを前提として処理を行う
pub(crate) fn calculate_rewards(payment_info: &[PaymentInfo]) -> Result<i32, ErrResp> {
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
pub(crate) fn create_start_and_end_date_time_of_current_year(
    current_date_time: &DateTime<FixedOffset>,
) -> (DateTime<FixedOffset>, DateTime<FixedOffset>) {
    let current_year = current_date_time.year();

    let start = JAPANESE_TIME_ZONE.ymd(current_year, 1, 1).and_hms(0, 0, 0);

    let end = JAPANESE_TIME_ZONE
        .ymd(current_year, 12, 31)
        .and_hms(23, 59, 59);

    (start, end)
}

/// 渡された日時に対して、その月の日本時間における1日0時0分0秒と最終日23時59分59秒を示す日時を返す。
pub(crate) fn create_start_and_end_date_time_of_current_month(
    current_date_time: &DateTime<FixedOffset>,
) -> (DateTime<FixedOffset>, DateTime<FixedOffset>) {
    let current_year = current_date_time.year();
    let current_month = current_date_time.month();
    let start = JAPANESE_TIME_ZONE
        .ymd(current_date_time.year(), current_month, 1)
        .and_hms(0, 0, 0);

    let (year, month) = if current_month >= 12 {
        (current_year + 1, 1)
    } else {
        (current_year, current_month + 1)
    };
    let end = JAPANESE_TIME_ZONE.ymd(year, month, 1).and_hms(23, 59, 59) - Duration::days(1);

    (start, end)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use chrono::{DateTime, FixedOffset, TimeZone};
    use common::payment_platform::Metadata;
    use common::JAPANESE_TIME_ZONE;
    // use chrono::TimeZone;
    use common::payment_platform::charge::RefundQuery;
    use common::{
        payment_platform::{
            charge::{Charge, ChargeOperation, CreateCharge, Query},
            customer::Card,
            ErrorDetail, ErrorInfo, List,
        },
        ErrResp,
    };
    use once_cell::sync::Lazy;
    use rust_decimal::prelude::FromPrimitive;
    use rust_decimal::{Decimal, RoundingStrategy};

    // use crate::util::rewards::calculate_expected_rewards;
    // use crate::util::KEY_TO_MEETING_DATE_TIME_IN_JST_ON_CHARGE_OBJ;

    use crate::util::rewards::create_start_and_end_date_time_of_current_year;

    use super::calculate_rewards;
    // use common::{ApiError, JAPANESE_TIME_ZONE};
    // use once_cell::sync::Lazy;
    // use rust_decimal::{prelude::FromPrimitive, Decimal, RoundingStrategy};

    // use crate::err::Code;

    // #[derive(Debug)]
    // struct TestCase {
    //     name: String,
    //     input: Input,
    //     expected: Result<i32, ErrResp>,
    // }

    // #[derive(Debug)]
    // struct Input {
    //     charge_op: ChargeOperationMock,
    //     num_of_charges_per_req: u32,
    //     since_timestamp: i64,
    //     until_timestamp: i64,
    //     tenant_id: String,
    // }

    // static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
    //     vec![
    //         TestCase {
    //             name: "empty results".to_string(),
    //             input: Input {
    //                 charge_op: ChargeOperationMock {
    //                     num_of_charges_per_req: 1,
    //                     since_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 1)
    //                         .and_hms(0, 0, 0)
    //                         .timestamp(),
    //                     until_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 30)
    //                         .and_hms(23, 59, 59)
    //                         .timestamp(),
    //                     tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //                     num_of_search_trial: 0,
    //                     lists: vec![List {
    //                         object: "list".to_string(),
    //                         has_more: false,
    //                         url: "/v1/charges".to_string(),
    //                         data: vec![],
    //                         count: 0,
    //                     }],
    //                     too_many_requests: false,
    //                 },
    //                 num_of_charges_per_req: 1,
    //                 since_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 1)
    //                     .and_hms(0, 0, 0)
    //                     .timestamp(),
    //                 until_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 30)
    //                     .and_hms(23, 59, 59)
    //                     .timestamp(),
    //                 tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //             },
    //             expected: Ok(0),
    //         },
    //         TestCase {
    //             name: "one result one request".to_string(),
    //             input: Input {
    //                 charge_op: ChargeOperationMock {
    //                     num_of_charges_per_req: 1,
    //                     since_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 1)
    //                         .and_hms(0, 0, 0)
    //                         .timestamp(),
    //                     until_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 30)
    //                         .and_hms(23, 59, 59)
    //                         .timestamp(),
    //                     tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //                     num_of_search_trial: 0,
    //                     lists: vec![List {
    //                         object: "list".to_string(),
    //                         has_more: false,
    //                         url: "/v1/charges".to_string(),
    //                         data: vec![create_dummy_charge(
    //                             "ch_7fb5aea258910da9a756985cbe51f",
    //                             "336e7d16726246b69636d58bec7a3a30",
    //                             4000,
    //                             0,
    //                             "30.0",
    //                             true,
    //                         )],
    //                         count: 1,
    //                     }],
    //                     too_many_requests: false,
    //                 },
    //                 num_of_charges_per_req: 1,
    //                 since_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 1)
    //                     .and_hms(0, 0, 0)
    //                     .timestamp(),
    //                 until_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 30)
    //                     .and_hms(23, 59, 59)
    //                     .timestamp(),
    //                 tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //             },
    //             expected: Ok(2800),
    //         },
    //         TestCase {
    //             name: "two results one request".to_string(),
    //             input: Input {
    //                 charge_op: ChargeOperationMock {
    //                     num_of_charges_per_req: 2,
    //                     since_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 1)
    //                         .and_hms(0, 0, 0)
    //                         .timestamp(),
    //                     until_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 30)
    //                         .and_hms(23, 59, 59)
    //                         .timestamp(),
    //                     tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //                     num_of_search_trial: 0,
    //                     lists: vec![List {
    //                         object: "list".to_string(),
    //                         has_more: false,
    //                         url: "/v1/charges".to_string(),
    //                         data: vec![
    //                             create_dummy_charge(
    //                                 "ch_7fb5aea258910da9a756985cbe51f",
    //                                 "336e7d16726246b69636d58bec7a3a30",
    //                                 4000,
    //                                 0,
    //                                 "30.0",
    //                                 true,
    //                             ),
    //                             create_dummy_charge(
    //                                 "ch_7fb5aea258910da9a756985cbe511",
    //                                 "336e7d16726246b69636d58bec7a3a30",
    //                                 3000,
    //                                 0,
    //                                 "30.0",
    //                                 true,
    //                             ),
    //                         ],
    //                         count: 2,
    //                     }],
    //                     too_many_requests: false,
    //                 },
    //                 num_of_charges_per_req: 2,
    //                 since_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 1)
    //                     .and_hms(0, 0, 0)
    //                     .timestamp(),
    //                 until_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 30)
    //                     .and_hms(23, 59, 59)
    //                     .timestamp(),
    //                 tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //             },
    //             expected: Ok(4900),
    //         },
    //         TestCase {
    //             name: "three results two requests".to_string(),
    //             input: Input {
    //                 charge_op: ChargeOperationMock {
    //                     num_of_charges_per_req: 2,
    //                     since_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 1)
    //                         .and_hms(0, 0, 0)
    //                         .timestamp(),
    //                     until_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 30)
    //                         .and_hms(23, 59, 59)
    //                         .timestamp(),
    //                     tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //                     num_of_search_trial: 0,
    //                     lists: vec![
    //                         List {
    //                             object: "list".to_string(),
    //                             has_more: true,
    //                             url: "/v1/charges".to_string(),
    //                             data: vec![
    //                                 create_dummy_charge(
    //                                     "ch_7fb5aea258910da9a756985cbe51f",
    //                                     "336e7d16726246b69636d58bec7a3a30",
    //                                     4000,
    //                                     0,
    //                                     "30.0",
    //                                     true,
    //                                 ),
    //                                 create_dummy_charge(
    //                                     "ch_7fb5aea258910da9a756985cbe511",
    //                                     "336e7d16726246b69636d58bec7a3a30",
    //                                     3000,
    //                                     0,
    //                                     "30.0",
    //                                     true,
    //                                 ),
    //                             ],
    //                             count: 2,
    //                         },
    //                         List {
    //                             object: "list".to_string(),
    //                             has_more: false,
    //                             url: "/v1/charges".to_string(),
    //                             data: vec![create_dummy_charge(
    //                                 "ch_7fb5aea258910da9a756985cbe512",
    //                                 "336e7d16726246b69636d58bec7a3a30",
    //                                 5000,
    //                                 0,
    //                                 "30.0",
    //                                 true,
    //                             )],
    //                             count: 1,
    //                         },
    //                     ],
    //                     too_many_requests: false,
    //                 },
    //                 num_of_charges_per_req: 2,
    //                 since_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 1)
    //                     .and_hms(0, 0, 0)
    //                     .timestamp(),
    //                 until_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 30)
    //                     .and_hms(23, 59, 59)
    //                     .timestamp(),
    //                 tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //             },
    //             expected: Ok(8400),
    //         },
    //         TestCase {
    //             name: "refunded".to_string(),
    //             input: Input {
    //                 charge_op: ChargeOperationMock {
    //                     num_of_charges_per_req: 1,
    //                     since_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 1)
    //                         .and_hms(0, 0, 0)
    //                         .timestamp(),
    //                     until_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 30)
    //                         .and_hms(23, 59, 59)
    //                         .timestamp(),
    //                     tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //                     num_of_search_trial: 0,
    //                     lists: vec![List {
    //                         object: "list".to_string(),
    //                         has_more: false,
    //                         url: "/v1/charges".to_string(),
    //                         data: vec![create_dummy_charge(
    //                             "ch_7fb5aea258910da9a756985cbe51f",
    //                             "336e7d16726246b69636d58bec7a3a30",
    //                             4000,
    //                             4000,
    //                             "30.0",
    //                             true,
    //                         )],
    //                         count: 1,
    //                     }],
    //                     too_many_requests: false,
    //                 },
    //                 num_of_charges_per_req: 1,
    //                 since_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 1)
    //                     .and_hms(0, 0, 0)
    //                     .timestamp(),
    //                 until_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 30)
    //                     .and_hms(23, 59, 59)
    //                     .timestamp(),
    //                 tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //             },
    //             expected: Ok(0),
    //         },
    //         TestCase {
    //             name: "partially refunded".to_string(),
    //             input: Input {
    //                 charge_op: ChargeOperationMock {
    //                     num_of_charges_per_req: 1,
    //                     since_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 1)
    //                         .and_hms(0, 0, 0)
    //                         .timestamp(),
    //                     until_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 30)
    //                         .and_hms(23, 59, 59)
    //                         .timestamp(),
    //                     tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //                     num_of_search_trial: 0,
    //                     lists: vec![List {
    //                         object: "list".to_string(),
    //                         has_more: false,
    //                         url: "/v1/charges".to_string(),
    //                         data: vec![create_dummy_charge(
    //                             "ch_7fb5aea258910da9a756985cbe51f",
    //                             "336e7d16726246b69636d58bec7a3a30",
    //                             4000,
    //                             1000,
    //                             "30.0",
    //                             true,
    //                         )],
    //                         count: 1,
    //                     }],
    //                     too_many_requests: false,
    //                 },
    //                 num_of_charges_per_req: 1,
    //                 since_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 1)
    //                     .and_hms(0, 0, 0)
    //                     .timestamp(),
    //                 until_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 30)
    //                     .and_hms(23, 59, 59)
    //                     .timestamp(),
    //                 tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //             },
    //             expected: Ok(2100),
    //         },
    //         TestCase {
    //             name: "non captured charge is not counted as rewards".to_string(),
    //             input: Input {
    //                 charge_op: ChargeOperationMock {
    //                     num_of_charges_per_req: 1,
    //                     since_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 1)
    //                         .and_hms(0, 0, 0)
    //                         .timestamp(),
    //                     until_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 30)
    //                         .and_hms(23, 59, 59)
    //                         .timestamp(),
    //                     tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //                     num_of_search_trial: 0,
    //                     lists: vec![List {
    //                         object: "list".to_string(),
    //                         has_more: false,
    //                         url: "/v1/charges".to_string(),
    //                         data: vec![create_dummy_charge(
    //                             "ch_7fb5aea258910da9a756985cbe51f",
    //                             "336e7d16726246b69636d58bec7a3a30",
    //                             4000,
    //                             0,
    //                             "30.0",
    //                             false,
    //                         )],
    //                         count: 1,
    //                     }],
    //                     too_many_requests: false,
    //                 },
    //                 num_of_charges_per_req: 1,
    //                 since_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 1)
    //                     .and_hms(0, 0, 0)
    //                     .timestamp(),
    //                 until_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 30)
    //                     .and_hms(23, 59, 59)
    //                     .timestamp(),
    //                 tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //             },
    //             expected: Ok(0),
    //         },
    //         TestCase {
    //             name: "too many requests".to_string(),
    //             input: Input {
    //                 charge_op: ChargeOperationMock {
    //                     num_of_charges_per_req: 1,
    //                     since_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 1)
    //                         .and_hms(0, 0, 0)
    //                         .timestamp(),
    //                     until_timestamp: JAPANESE_TIME_ZONE
    //                         .ymd(2022, 9, 30)
    //                         .and_hms(23, 59, 59)
    //                         .timestamp(),
    //                     tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //                     num_of_search_trial: 0,
    //                     lists: vec![List {
    //                         object: "list".to_string(),
    //                         has_more: false,
    //                         url: "/v1/charges".to_string(),
    //                         data: vec![create_dummy_charge(
    //                             "ch_7fb5aea258910da9a756985cbe51f",
    //                             "336e7d16726246b69636d58bec7a3a30",
    //                             4000,
    //                             0,
    //                             "30.0",
    //                             true,
    //                         )],
    //                         count: 1,
    //                     }],
    //                     too_many_requests: true,
    //                 },
    //                 num_of_charges_per_req: 1,
    //                 since_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 1)
    //                     .and_hms(0, 0, 0)
    //                     .timestamp(),
    //                 until_timestamp: JAPANESE_TIME_ZONE
    //                     .ymd(2022, 9, 30)
    //                     .and_hms(23, 59, 59)
    //                     .timestamp(),
    //                 tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
    //             },
    //             expected: Err((
    //                 StatusCode::TOO_MANY_REQUESTS,
    //                 Json(ApiError {
    //                     code: Code::ReachPaymentPlatformRateLimit as u32,
    //                 }),
    //             )),
    //         },
    //     ]
    // });

    // fn create_dummy_charge(
    //     charge_id: &str,
    //     tenant_id: &str,
    //     amount: i32,
    //     amount_refunded: i32,
    //     platform_fee_rate: &str,
    //     captured: bool,
    // ) -> Charge {
    //     let refunded = amount_refunded > 0;
    //     let refund_reason = if refunded {
    //         Some("テスト".to_string())
    //     } else {
    //         None
    //     };
    //     let fee_rate = 3;
    //     let sale = amount - amount_refunded;
    //     let fee = Decimal::from_i32(sale).unwrap()
    //         * (Decimal::from_i32(fee_rate).unwrap() / Decimal::from_i32(100).unwrap());
    //     let platform_fee = Decimal::from_i32(sale).unwrap()
    //         * (Decimal::from_str(platform_fee_rate).unwrap() / Decimal::from_i32(100).unwrap());
    //     // tenantオブジェクトのpayjp_fee_includedがtrueであることが前提のtotal_platform_fee
    //     let total_platform_fee =
    //         (platform_fee - fee).round_dp_with_strategy(0, RoundingStrategy::ToZero);
    //     Charge {
    //         id: charge_id.to_string(),
    //         object: "charge".to_string(),
    //         livemode: false,
    //         created: 1639931415,
    //         amount,
    //         currency: "jpy".to_string(),
    //         paid: true,
    //         expired_at: None,
    //         captured,
    //         captured_at: Some(1639931415),
    //         card: Some(Card {
    //             object: "card".to_string(),
    //             id: "car_33ab04bcdc00f0cc6d6df16bbe79".to_string(),
    //             created: 1639931415,
    //             name: None,
    //             last4: "4242".to_string(),
    //             exp_month: 12,
    //             exp_year: 2022,
    //             brand: "Visa".to_string(),
    //             cvc_check: "passed".to_string(),
    //             fingerprint: "e1d8225886e3a7211127df751c86787f".to_string(),
    //             address_state: None,
    //             address_city: None,
    //             address_line1: None,
    //             address_line2: None,
    //             country: None,
    //             address_zip: None,
    //             address_zip_check: "unchecked".to_string(),
    //             metadata: None,
    //         }),
    //         customer: None,
    //         description: None,
    //         failure_code: None,
    //         failure_message: None,
    //         fee_rate: Some(fee_rate.to_string()),
    //         refunded,
    //         amount_refunded,
    //         refund_reason,
    //         subscription: None,
    //         metadata: None,
    //         platform_fee: None,
    //         tenant: Some(tenant_id.to_string()),
    //         platform_fee_rate: Some(platform_fee_rate.to_string()),
    //         total_platform_fee: Some(
    //             total_platform_fee
    //                 .to_string()
    //                 .parse::<i32>()
    //                 .expect("failed to parse number str"),
    //         ),
    //         three_d_secure_status: Some("verified".to_string()),
    //     }
    // }

    // #[tokio::test]
    // async fn test_get_rewards_of_the_duration() {
    //     for test_case in TEST_CASE_SET.iter() {
    //         let charge_op = test_case.input.charge_op.clone();
    //         let num_of_charges_per_req = test_case.input.num_of_charges_per_req;
    //         let since_timestamp = test_case.input.since_timestamp;
    //         let until_timestamp = test_case.input.until_timestamp;
    //         let tenant_id = test_case.input.tenant_id.clone();

    //         let result = get_rewards_of_the_duration(
    //             charge_op,
    //             num_of_charges_per_req,
    //             since_timestamp,
    //             until_timestamp,
    //             tenant_id.as_str(),
    //         )
    //         .await;

    //         let message = format!("test case \"{}\" failed", test_case.name.clone());
    //         if test_case.expected.is_ok() {
    //             let result = result.expect("failed to get Ok");
    //             let expected_result = *test_case.expected.as_ref().expect("failed to get Ok");
    //             assert_eq!(expected_result, result, "{}", message);
    //         } else {
    //             let result = result.expect_err("failed to get Err");
    //             let expected_result = test_case.expected.as_ref().expect_err("failed to get Err");
    //             assert_eq!(expected_result.0, result.0, "{}", message);
    //             assert_eq!(expected_result.1 .0, result.1 .0, "{}", message);
    //         }
    //     }
    // }

    #[derive(Debug)]
    struct CalculateRewardsTestCase {
        name: String,
        input: Vec<Charge>,
        expected: i32,
    }

    static CALCULATE_REWARDS_TEST_CASE_SET: Lazy<Vec<CalculateRewardsTestCase>> = Lazy::new(|| {
        let mdt = Some((*JAPANESE_TIME_ZONE).ymd(2022, 4, 5).and_hms(21, 00, 00));
        vec![
            CalculateRewardsTestCase {
                name: "1 charge".to_string(),
                input: vec![create_dummy_charge_for_calc(
                    5000, 0, "30.0", true, 1675176747, mdt,
                )],
                expected: 3500,
            },
            CalculateRewardsTestCase {
                name: "2 charges".to_string(),
                input: vec![
                    create_dummy_charge_for_calc(5000, 0, "30.0", true, 1675176747, mdt),
                    create_dummy_charge_for_calc(4000, 0, "30.0", true, 1675176747, mdt),
                ],
                expected: 3500 + 2800,
            },
            CalculateRewardsTestCase {
                name: "3 charges".to_string(),
                input: vec![
                    create_dummy_charge_for_calc(5000, 0, "30.0", true, 1675176747, mdt),
                    create_dummy_charge_for_calc(4000, 0, "30.0", true, 1675176747, mdt),
                    create_dummy_charge_for_calc(3000, 0, "30.0", true, 1675176747, mdt),
                ],
                expected: 3500 + 2800 + 2100,
            },
            CalculateRewardsTestCase {
                name: "non captured charge is not counted as rewards case 1".to_string(),
                input: vec![create_dummy_charge_for_calc(
                    4000, 0, "30.0", false, 1675176747, mdt,
                )],
                expected: 0,
            },
            CalculateRewardsTestCase {
                name: "non captured charge is not counted as rewards case 2".to_string(),
                input: vec![
                    create_dummy_charge_for_calc(5000, 0, "30.0", true, 1675176747, mdt),
                    create_dummy_charge_for_calc(4000, 0, "30.0", false, 1675176747, mdt),
                ],
                expected: 3500,
            },
            CalculateRewardsTestCase {
                name: "fully refunded charge case 1".to_string(),
                input: vec![create_dummy_charge_for_calc(
                    5000, 5000, "30.0", true, 1675176747, mdt,
                )],
                expected: 0,
            },
            CalculateRewardsTestCase {
                name: "fully refunded charge case 2".to_string(),
                input: vec![
                    create_dummy_charge_for_calc(4000, 0, "30.0", true, 1675176747, mdt),
                    create_dummy_charge_for_calc(5000, 5000, "30.0", true, 1675176747, mdt),
                ],
                expected: 2800,
            },
            CalculateRewardsTestCase {
                name: "partially refunded charge case 1".to_string(), // 部分返金を実装する予定はないが、念の為テストしておく
                input: vec![create_dummy_charge_for_calc(
                    5000, 1000, "30.0", true, 1675176747, mdt,
                )],
                expected: 2800,
            },
            CalculateRewardsTestCase {
                name: "partially refunded charge case 2".to_string(), // 部分返金を実装する予定はないが、念の為テストしておく
                input: vec![
                    create_dummy_charge_for_calc(3000, 0, "30.0", true, 1675176747, mdt),
                    create_dummy_charge_for_calc(5000, 1000, "30.0", true, 1675176747, mdt),
                ],
                expected: 2100 + 2800,
            },
            CalculateRewardsTestCase {
                name: "decimal number round to zero case 1".to_string(),
                input: vec![create_dummy_charge_for_calc(
                    5003, 0, "30.0", true, 1675176747, mdt,
                )],
                expected: 3503,
            },
            CalculateRewardsTestCase {
                name: "decimal number round to zero case 2".to_string(),
                input: vec![create_dummy_charge_for_calc(
                    4008, 0, "30.0", true, 1675176747, mdt,
                )],
                expected: 2806,
            },
            CalculateRewardsTestCase {
                name: "decimal number round to zero case 3".to_string(),
                input: vec![
                    create_dummy_charge_for_calc(5003, 0, "30.0", true, 1675176747, mdt),
                    create_dummy_charge_for_calc(4008, 0, "30.0", true, 1675176747, mdt),
                ],
                expected: 3503 + 2806,
            },
        ]
    });

    fn create_dummy_charge_for_calc(
        amount: i32,
        amount_refunded: i32,
        platform_fee_rate: &str,
        captured: bool,
        expired_at: i64,
        meeting_date_time: Option<DateTime<FixedOffset>>,
    ) -> Charge {
        let refunded = amount_refunded > 0;
        let refund_reason = if refunded {
            Some("テスト".to_string())
        } else {
            None
        };
        let fee_rate = 3;
        let sale = amount - amount_refunded;
        let fee = Decimal::from_i32(sale).unwrap()
            * (Decimal::from_i32(fee_rate).unwrap() / Decimal::from_i32(100).unwrap());
        let platform_fee = Decimal::from_i32(sale).unwrap()
            * (Decimal::from_str(platform_fee_rate).unwrap() / Decimal::from_i32(100).unwrap());
        // tenantオブジェクトのpayjp_fee_includedがtrueであることが前提のtotal_platform_fee
        let total_platform_fee =
            (platform_fee - fee).round_dp_with_strategy(0, RoundingStrategy::ToZero);
        let metadata = if let Some(mdt) = meeting_date_time {
            let mut md = Metadata::with_capacity(1); // 実際にはその他にもメタデータもあるが、テストで利用しないため省略
                                                     // md.insert(
                                                     //     KEY_TO_MEETING_DATE_TIME_IN_JST_ON_CHARGE_OBJ.to_string(),
                                                     //     mdt.to_rfc3339(),
                                                     // );
            Some(md)
        } else {
            None
        };
        Charge {
            id: "ch_845572127a994770fe175d906094f".to_string(),
            object: "charge".to_string(),
            livemode: false,
            created: 1639931415,
            amount,
            currency: "jpy".to_string(),
            paid: true,
            expired_at: Some(expired_at),
            captured,
            captured_at: Some(1639931415),
            card: Some(Card {
                object: "card".to_string(),
                id: "car_33ab04bcdc00f0cc6d6df16bbe79".to_string(),
                created: 1639931415,
                name: None,
                last4: "4242".to_string(),
                exp_month: 12,
                exp_year: 2022,
                brand: "Visa".to_string(),
                cvc_check: "passed".to_string(),
                fingerprint: "e1d8225886e3a7211127df751c86787f".to_string(),
                address_state: None,
                address_city: None,
                address_line1: None,
                address_line2: None,
                country: None,
                address_zip: None,
                address_zip_check: "unchecked".to_string(),
                metadata: None,
            }),
            customer: None,
            description: None,
            failure_code: None,
            failure_message: None,
            fee_rate: Some(fee_rate.to_string()),
            refunded,
            amount_refunded,
            refund_reason,
            subscription: None,
            metadata,
            platform_fee: None,
            tenant: Some("bbcccc6d8bfb4dff9d133c993ecbe084".to_string()),
            platform_fee_rate: Some(platform_fee_rate.to_string()),
            total_platform_fee: Some(
                total_platform_fee
                    .to_string()
                    .parse::<i32>()
                    .expect("failed to parse number str"),
            ),
            three_d_secure_status: Some("verified".to_string()),
        }
    }

    #[test]
    fn test_calculate_rewards() {
        for test_case in CALCULATE_REWARDS_TEST_CASE_SET.iter() {
            // let result = calculate_rewards(&test_case.input).expect("failed to get Ok");
            // let message = format!("test case \"{}\" failed", test_case.name.clone());
            // assert_eq!(result, test_case.expected, "{}", message);
        }
    }

    #[derive(Debug)]
    struct CalculateExpectedRewardsTestCase {
        name: String,
        input: (Vec<Charge>, DateTime<FixedOffset>),
        expected: i32,
    }

    static CALCULATE_EXPECTED_REWARDS_TEST_CASE_SET: Lazy<Vec<CalculateExpectedRewardsTestCase>> =
        Lazy::new(|| vec![]);

    #[test]
    fn test_calculate_expected_rewards() {
        for test_case in CALCULATE_EXPECTED_REWARDS_TEST_CASE_SET.iter() {
            // let result = calculate_expected_rewards(&test_case.input.0, &test_case.input.1)
            //     .expect("failed to get Ok");
            // let message = format!("test case \"{}\" failed", test_case.name.clone());
            // assert_eq!(result, test_case.expected, "{}", message);
        }
    }

    #[test]
    fn test_case_normal_year_create_start_and_end_date_time_of_current_year() {
        let current_date_time = JAPANESE_TIME_ZONE.ymd(2022, 7, 15).and_hms(15, 43, 39);
        let (start, end) = create_start_and_end_date_time_of_current_year(&current_date_time);
        assert_eq!(JAPANESE_TIME_ZONE.ymd(2022, 1, 1).and_hms(0, 0, 0), start);
        assert_eq!(
            JAPANESE_TIME_ZONE.ymd(2022, 12, 31).and_hms(23, 59, 59),
            end
        );
    }

    #[test]
    fn test_case_leap_year_create_start_and_end_date_time_of_current_year() {
        let current_date_time = JAPANESE_TIME_ZONE.ymd(2020, 12, 15).and_hms(15, 43, 39);
        let (start, end) = create_start_and_end_date_time_of_current_year(&current_date_time);
        assert_eq!(JAPANESE_TIME_ZONE.ymd(2020, 1, 1).and_hms(0, 0, 0), start);
        assert_eq!(
            JAPANESE_TIME_ZONE.ymd(2020, 12, 31).and_hms(23, 59, 59),
            end
        );
    }
}
