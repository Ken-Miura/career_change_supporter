// Copyright 2022 Ken Miura

use std::str::FromStr;

use axum::{http::StatusCode, Json};
use common::{
    payment_platform::charge::{Charge, ChargeOperation, Query as SearchChargesQuery},
    ApiError, ErrResp,
};
use rust_decimal::{prelude::FromPrimitive, Decimal, RoundingStrategy};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

pub(crate) const MAX_NUM_OF_CHARGES_PER_REQUEST: u32 = 100;

pub(crate) async fn get_rewards_of_the_duration(
    mut charge_op: impl ChargeOperation,
    num_of_charges_per_req: u32,
    since_timestamp: i64,
    until_timestamp: i64,
    tenant_id: &str,
) -> Result<i32, ErrResp> {
    let search_charges_query = SearchChargesQuery::build()
        .limit(num_of_charges_per_req)
        .since(since_timestamp)
        .until(until_timestamp)
        .tenant(tenant_id)
        .finish()
        .map_err(|e| {
            error!("failed to build search charges query: {}", e);
            unexpected_err_resp()
        })?;
    let mut has_more_charges = true;
    let mut rewards_of_the_duration = 0;
    while has_more_charges {
        let charges = charge_op
            .search_charges(&search_charges_query)
            .await
            .map_err(|err| match err {
                common::payment_platform::Error::RequestProcessingError(err) => {
                    error!("failed to process request on getting charges: {}", err);
                    unexpected_err_resp()
                }
                common::payment_platform::Error::ApiError(err) => {
                    error!("failed to request charge operation: {}", err);
                    let status_code = err.error.status as u16;
                    if status_code == StatusCode::TOO_MANY_REQUESTS.as_u16() {
                        return (
                            StatusCode::TOO_MANY_REQUESTS,
                            Json(ApiError {
                                code: Code::ReachPaymentPlatformRateLimit as u32,
                            }),
                        );
                    }
                    unexpected_err_resp()
                }
            })?;
        let rewards = charges
            .data
            .into_iter()
            .filter(|charge| charge.captured)
            .try_fold(0, accumulate_rewards)?;
        rewards_of_the_duration += rewards;
        has_more_charges = charges.has_more;
    }
    Ok(rewards_of_the_duration)
}

// [tenantオブジェクト](https://pay.jp/docs/api/#tenant%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88)のpayjp_fee_includedがtrueであるとことを前提として実装
// payjp_fee_includedの値を設定できるのはテナント作成時のみ。そのためテナント作成時のコードで必ずtrueを設定することで、ここでtrueを前提として処理を行う
fn accumulate_rewards(sum: i32, charge: Charge) -> Result<i32, ErrResp> {
    let sales = charge.amount - charge.amount_refunded;
    if let Some(platform_fee_rate) = charge.platform_fee_rate.clone() {
        let fee = calculate_fee(sales, &platform_fee_rate)?;
        let reward_of_the_charge = sales - fee;
        if reward_of_the_charge < 0 {
            error!("negative reward_of_the_charge: {:?}", charge);
            return Err(unexpected_err_resp());
        }
        Ok(sum + reward_of_the_charge)
    } else {
        error!("no platform_fee_rate found in the charge: {:?}", charge);
        Err(unexpected_err_resp())
    }
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use chrono::TimeZone;
    use common::{
        payment_platform::{
            charge::{Charge, ChargeOperation, CreateCharge, Query as SearchChargesQuery},
            customer::Card,
            ErrorDetail, ErrorInfo, List,
        },
        ErrResp,
    };
    use common::{ApiError, JAPANESE_TIME_ZONE};
    use once_cell::sync::Lazy;
    use rust_decimal::{prelude::FromPrimitive, Decimal, RoundingStrategy};

    use crate::err::Code;

    use super::get_rewards_of_the_duration;

    #[derive(Debug, Clone)]
    struct ChargeOperationMock {
        num_of_charges_per_req: u32,
        since_timestamp: i64,
        until_timestamp: i64,
        tenant_id: String,
        num_of_search_trial: usize,
        lists: Vec<List<Charge>>,
        too_many_requests: bool,
    }

    #[async_trait]
    impl ChargeOperation for ChargeOperationMock {
        async fn search_charges(
            &mut self,
            query: &SearchChargesQuery,
        ) -> Result<List<Charge>, common::payment_platform::Error> {
            assert_eq!(
                self.num_of_charges_per_req,
                query.limit().expect("failed to get limit")
            );
            assert_eq!(
                self.since_timestamp,
                query.since().expect("failed to get since")
            );
            assert_eq!(
                self.until_timestamp,
                query.until().expect("failed to get until")
            );
            assert_eq!(
                self.tenant_id,
                query.tenant().expect("failed to get tenant")
            );
            if self.too_many_requests {
                let err_detail = ErrorDetail {
                    message: "message".to_string(),
                    status: StatusCode::TOO_MANY_REQUESTS.as_u16() as u32,
                    r#type: "type".to_string(),
                    code: None,
                    param: None,
                    charge: None,
                };
                let err_info = ErrorInfo { error: err_detail };
                return Err(common::payment_platform::Error::ApiError(Box::new(
                    err_info,
                )));
            }
            let result = self.lists[self.num_of_search_trial].clone();
            self.num_of_search_trial += 1;
            Ok(result)
        }

        async fn create_charge(
            &self,
            _create_charge: &CreateCharge,
        ) -> Result<Charge, common::payment_platform::Error> {
            // このAPIでは必要ない機能なので、呼んだらテストを失敗させる
            panic!("this method must not be called")
        }

        async fn ge_charge_by_charge_id(
            &self,
            _charge_id: &str,
        ) -> Result<Charge, common::payment_platform::Error> {
            // このAPIでは必要ない機能なので、呼んだらテストを失敗させる
            panic!("this method must not be called")
        }

        async fn finish_three_d_secure_flow(
            &self,
            _charge_id: &str,
        ) -> Result<Charge, common::payment_platform::Error> {
            // このAPIでは必要ない機能なので、呼んだらテストを失敗させる
            panic!("this method must not be called")
        }
    }

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: Result<i32, ErrResp>,
    }

    #[derive(Debug)]
    struct Input {
        charge_op: ChargeOperationMock,
        num_of_charges_per_req: u32,
        since_timestamp: i64,
        until_timestamp: i64,
        tenant_id: String,
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        vec![
            TestCase {
                name: "empty results".to_string(),
                input: Input {
                    charge_op: ChargeOperationMock {
                        num_of_charges_per_req: 1,
                        since_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 1)
                            .and_hms(0, 0, 0)
                            .timestamp(),
                        until_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 30)
                            .and_hms(23, 59, 59)
                            .timestamp(),
                        tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                        num_of_search_trial: 0,
                        lists: vec![List {
                            object: "list".to_string(),
                            has_more: false,
                            url: "/v1/charges".to_string(),
                            data: vec![],
                            count: 0,
                        }],
                        too_many_requests: false,
                    },
                    num_of_charges_per_req: 1,
                    since_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 1)
                        .and_hms(0, 0, 0)
                        .timestamp(),
                    until_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 30)
                        .and_hms(23, 59, 59)
                        .timestamp(),
                    tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                },
                expected: Ok(0),
            },
            TestCase {
                name: "one result one request".to_string(),
                input: Input {
                    charge_op: ChargeOperationMock {
                        num_of_charges_per_req: 1,
                        since_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 1)
                            .and_hms(0, 0, 0)
                            .timestamp(),
                        until_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 30)
                            .and_hms(23, 59, 59)
                            .timestamp(),
                        tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                        num_of_search_trial: 0,
                        lists: vec![List {
                            object: "list".to_string(),
                            has_more: false,
                            url: "/v1/charges".to_string(),
                            data: vec![create_dummy_charge(
                                "ch_7fb5aea258910da9a756985cbe51f",
                                "336e7d16726246b69636d58bec7a3a30",
                                4000,
                                0,
                                "30.0",
                                true,
                            )],
                            count: 1,
                        }],
                        too_many_requests: false,
                    },
                    num_of_charges_per_req: 1,
                    since_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 1)
                        .and_hms(0, 0, 0)
                        .timestamp(),
                    until_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 30)
                        .and_hms(23, 59, 59)
                        .timestamp(),
                    tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                },
                expected: Ok(2800),
            },
            TestCase {
                name: "two results one request".to_string(),
                input: Input {
                    charge_op: ChargeOperationMock {
                        num_of_charges_per_req: 2,
                        since_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 1)
                            .and_hms(0, 0, 0)
                            .timestamp(),
                        until_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 30)
                            .and_hms(23, 59, 59)
                            .timestamp(),
                        tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                        num_of_search_trial: 0,
                        lists: vec![List {
                            object: "list".to_string(),
                            has_more: false,
                            url: "/v1/charges".to_string(),
                            data: vec![
                                create_dummy_charge(
                                    "ch_7fb5aea258910da9a756985cbe51f",
                                    "336e7d16726246b69636d58bec7a3a30",
                                    4000,
                                    0,
                                    "30.0",
                                    true,
                                ),
                                create_dummy_charge(
                                    "ch_7fb5aea258910da9a756985cbe511",
                                    "336e7d16726246b69636d58bec7a3a30",
                                    3000,
                                    0,
                                    "30.0",
                                    true,
                                ),
                            ],
                            count: 2,
                        }],
                        too_many_requests: false,
                    },
                    num_of_charges_per_req: 2,
                    since_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 1)
                        .and_hms(0, 0, 0)
                        .timestamp(),
                    until_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 30)
                        .and_hms(23, 59, 59)
                        .timestamp(),
                    tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                },
                expected: Ok(4900),
            },
            TestCase {
                name: "three results two requests".to_string(),
                input: Input {
                    charge_op: ChargeOperationMock {
                        num_of_charges_per_req: 2,
                        since_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 1)
                            .and_hms(0, 0, 0)
                            .timestamp(),
                        until_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 30)
                            .and_hms(23, 59, 59)
                            .timestamp(),
                        tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                        num_of_search_trial: 0,
                        lists: vec![
                            List {
                                object: "list".to_string(),
                                has_more: true,
                                url: "/v1/charges".to_string(),
                                data: vec![
                                    create_dummy_charge(
                                        "ch_7fb5aea258910da9a756985cbe51f",
                                        "336e7d16726246b69636d58bec7a3a30",
                                        4000,
                                        0,
                                        "30.0",
                                        true,
                                    ),
                                    create_dummy_charge(
                                        "ch_7fb5aea258910da9a756985cbe511",
                                        "336e7d16726246b69636d58bec7a3a30",
                                        3000,
                                        0,
                                        "30.0",
                                        true,
                                    ),
                                ],
                                count: 2,
                            },
                            List {
                                object: "list".to_string(),
                                has_more: false,
                                url: "/v1/charges".to_string(),
                                data: vec![create_dummy_charge(
                                    "ch_7fb5aea258910da9a756985cbe512",
                                    "336e7d16726246b69636d58bec7a3a30",
                                    5000,
                                    0,
                                    "30.0",
                                    true,
                                )],
                                count: 1,
                            },
                        ],
                        too_many_requests: false,
                    },
                    num_of_charges_per_req: 2,
                    since_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 1)
                        .and_hms(0, 0, 0)
                        .timestamp(),
                    until_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 30)
                        .and_hms(23, 59, 59)
                        .timestamp(),
                    tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                },
                expected: Ok(8400),
            },
            TestCase {
                name: "refunded".to_string(),
                input: Input {
                    charge_op: ChargeOperationMock {
                        num_of_charges_per_req: 1,
                        since_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 1)
                            .and_hms(0, 0, 0)
                            .timestamp(),
                        until_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 30)
                            .and_hms(23, 59, 59)
                            .timestamp(),
                        tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                        num_of_search_trial: 0,
                        lists: vec![List {
                            object: "list".to_string(),
                            has_more: false,
                            url: "/v1/charges".to_string(),
                            data: vec![create_dummy_charge(
                                "ch_7fb5aea258910da9a756985cbe51f",
                                "336e7d16726246b69636d58bec7a3a30",
                                4000,
                                4000,
                                "30.0",
                                true,
                            )],
                            count: 1,
                        }],
                        too_many_requests: false,
                    },
                    num_of_charges_per_req: 1,
                    since_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 1)
                        .and_hms(0, 0, 0)
                        .timestamp(),
                    until_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 30)
                        .and_hms(23, 59, 59)
                        .timestamp(),
                    tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                },
                expected: Ok(0),
            },
            TestCase {
                name: "partially refunded".to_string(),
                input: Input {
                    charge_op: ChargeOperationMock {
                        num_of_charges_per_req: 1,
                        since_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 1)
                            .and_hms(0, 0, 0)
                            .timestamp(),
                        until_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 30)
                            .and_hms(23, 59, 59)
                            .timestamp(),
                        tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                        num_of_search_trial: 0,
                        lists: vec![List {
                            object: "list".to_string(),
                            has_more: false,
                            url: "/v1/charges".to_string(),
                            data: vec![create_dummy_charge(
                                "ch_7fb5aea258910da9a756985cbe51f",
                                "336e7d16726246b69636d58bec7a3a30",
                                4000,
                                1000,
                                "30.0",
                                true,
                            )],
                            count: 1,
                        }],
                        too_many_requests: false,
                    },
                    num_of_charges_per_req: 1,
                    since_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 1)
                        .and_hms(0, 0, 0)
                        .timestamp(),
                    until_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 30)
                        .and_hms(23, 59, 59)
                        .timestamp(),
                    tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                },
                expected: Ok(2100),
            },
            TestCase {
                name: "non captured charge is not counted as rewards".to_string(),
                input: Input {
                    charge_op: ChargeOperationMock {
                        num_of_charges_per_req: 1,
                        since_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 1)
                            .and_hms(0, 0, 0)
                            .timestamp(),
                        until_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 30)
                            .and_hms(23, 59, 59)
                            .timestamp(),
                        tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                        num_of_search_trial: 0,
                        lists: vec![List {
                            object: "list".to_string(),
                            has_more: false,
                            url: "/v1/charges".to_string(),
                            data: vec![create_dummy_charge(
                                "ch_7fb5aea258910da9a756985cbe51f",
                                "336e7d16726246b69636d58bec7a3a30",
                                4000,
                                0,
                                "30.0",
                                false,
                            )],
                            count: 1,
                        }],
                        too_many_requests: false,
                    },
                    num_of_charges_per_req: 1,
                    since_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 1)
                        .and_hms(0, 0, 0)
                        .timestamp(),
                    until_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 30)
                        .and_hms(23, 59, 59)
                        .timestamp(),
                    tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                },
                expected: Ok(0),
            },
            TestCase {
                name: "too many requests".to_string(),
                input: Input {
                    charge_op: ChargeOperationMock {
                        num_of_charges_per_req: 1,
                        since_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 1)
                            .and_hms(0, 0, 0)
                            .timestamp(),
                        until_timestamp: JAPANESE_TIME_ZONE
                            .ymd(2022, 9, 30)
                            .and_hms(23, 59, 59)
                            .timestamp(),
                        tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                        num_of_search_trial: 0,
                        lists: vec![List {
                            object: "list".to_string(),
                            has_more: false,
                            url: "/v1/charges".to_string(),
                            data: vec![create_dummy_charge(
                                "ch_7fb5aea258910da9a756985cbe51f",
                                "336e7d16726246b69636d58bec7a3a30",
                                4000,
                                0,
                                "30.0",
                                true,
                            )],
                            count: 1,
                        }],
                        too_many_requests: true,
                    },
                    num_of_charges_per_req: 1,
                    since_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 1)
                        .and_hms(0, 0, 0)
                        .timestamp(),
                    until_timestamp: JAPANESE_TIME_ZONE
                        .ymd(2022, 9, 30)
                        .and_hms(23, 59, 59)
                        .timestamp(),
                    tenant_id: "336e7d16726246b69636d58bec7a3a30".to_string(),
                },
                expected: Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ApiError {
                        code: Code::ReachPaymentPlatformRateLimit as u32,
                    }),
                )),
            },
        ]
    });

    fn create_dummy_charge(
        charge_id: &str,
        tenant_id: &str,
        amount: i32,
        amount_refunded: i32,
        platform_fee_rate: &str,
        captured: bool,
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
        Charge {
            id: charge_id.to_string(),
            object: "charge".to_string(),
            livemode: false,
            created: 1639931415,
            amount,
            currency: "jpy".to_string(),
            paid: true,
            expired_at: None,
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
            metadata: None,
            platform_fee: None,
            tenant: Some(tenant_id.to_string()),
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

    #[tokio::test]
    async fn test_get_rewards_of_the_duration() {
        for test_case in TEST_CASE_SET.iter() {
            let charge_op = test_case.input.charge_op.clone();
            let num_of_charges_per_req = test_case.input.num_of_charges_per_req;
            let since_timestamp = test_case.input.since_timestamp;
            let until_timestamp = test_case.input.until_timestamp;
            let tenant_id = test_case.input.tenant_id.clone();

            let result = get_rewards_of_the_duration(
                charge_op,
                num_of_charges_per_req,
                since_timestamp,
                until_timestamp,
                tenant_id.as_str(),
            )
            .await;

            let message = format!("test case \"{}\" failed", test_case.name.clone());
            if test_case.expected.is_ok() {
                let result = result.expect("failed to get Ok");
                let expected_result = *test_case.expected.as_ref().expect("failed to get Ok");
                assert_eq!(expected_result, result, "{}", message);
            } else {
                let result = result.expect_err("failed to get Err");
                let expected_result = test_case.expected.as_ref().expect_err("failed to get Err");
                assert_eq!(expected_result.0, result.0, "{}", message);
                assert_eq!(expected_result.1 .0, result.1 .0, "{}", message);
            }
        }
    }
}
