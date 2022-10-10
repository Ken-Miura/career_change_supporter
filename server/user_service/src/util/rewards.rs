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

const MAX_NUM_OF_CHARGES_PER_REQUEST: u32 = 100;

pub(crate) async fn get_rewards_of_the_duration(
    mut charge_op: impl ChargeOperation,
    since_timestamp: i64,
    until_timestamp: i64,
    tenant_id: &str,
) -> Result<i32, ErrResp> {
    let search_charges_query = SearchChargesQuery::build()
        .limit(MAX_NUM_OF_CHARGES_PER_REQUEST)
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
