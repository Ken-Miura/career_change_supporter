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

// TODO: Add test
#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::{
        payment_platform::{
            charge::{Charge, ChargeOperation, CreateCharge, Query as SearchChargesQuery},
            ErrorDetail, ErrorInfo, List,
        },
        ErrResp,
    };
    use once_cell::sync::Lazy;

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
                return Err(common::payment_platform::Error::ApiError(err_info));
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

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| vec![]);

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
