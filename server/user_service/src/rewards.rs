// Copyright 2021 Ken Miura

use std::str::FromStr;

use axum::{http::StatusCode, Json};
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, TimeZone, Utc};
use common::{
    model::user::Tenant,
    payment_platform::{
        charge::{Charge, ChargeOperation, ChargeOperationImpl, Query as SearchChargesQuery},
        tenant::{TenantOperation, TenantOperationImpl},
        tenant_transfer::{
            Query as SearchTenantTransfersQuery, TenantTransfer, TenantTransferOperation,
            TenantTransferOperationImpl,
        },
    },
    schema::ccs_schema::tenant::dsl::tenant as tenant_table,
    ApiError, DatabaseConnection, ErrResp, RespResult,
};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    result::Error::NotFound,
    PgConnection, QueryDsl, RunQueryDsl,
};
use rust_decimal::{prelude::FromPrimitive, Decimal, RoundingStrategy};
use serde::Serialize;

use crate::{
    err_code::{self},
    util::{session::User, unexpected_err_resp, BankAccount, Ymd, ACCESS_INFO, JAPANESE_TIME_ZONE},
};

const MAX_NUM_OF_CHARGES_PER_REQUEST: u32 = 32;
const MAX_NUM_OF_TENANT_TRANSFERS_PER_REQUEST: u32 = 2;

pub(crate) async fn get_reward(
    User { account_id }: User,
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<RewardResult> {
    let reward_op = RewardOperationImpl::new(conn);
    let tenant_op = TenantOperationImpl::new(&ACCESS_INFO);
    let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
    let tenant_transfer_op = TenantTransferOperationImpl::new(&ACCESS_INFO);
    let current_datetime = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
    get_reward_internal(
        account_id,
        reward_op,
        tenant_op,
        charge_op,
        current_datetime,
        tenant_transfer_op,
    )
    .await
}

async fn get_reward_internal(
    account_id: i32,
    reward_op: impl RewardOperation,
    tenant_op: impl TenantOperation,
    charge_op: impl ChargeOperation,
    current_time: DateTime<FixedOffset>,
    tenant_transfer_op: impl TenantTransferOperation,
) -> RespResult<RewardResult> {
    let tenant_option = async move { reward_op.find_tenant_by_user_account_id(account_id) }.await?;
    let payment_platform_results = if let Some(tenant) = tenant_option {
        let tenant_obj = get_tenant_obj_by_tenant_id(tenant_op, &tenant.tenant_id).await?;
        let bank_account = BankAccount {
            bank_code: tenant_obj.bank_code,
            branch_code: tenant_obj.bank_branch_code,
            account_type: tenant_obj.bank_account_type,
            account_number: tenant_obj.bank_account_number,
            account_holder_name: tenant_obj.bank_account_holder_name,
        };
        if !tenant_obj.payjp_fee_included {
            tracing::error!(
                "payjp_fee_included is false in tenant (id: {})",
                &tenant.tenant_id
            );
            return Err(unexpected_err_resp());
        }
        let rewards_of_the_month =
            get_rewards_of_current_month(charge_op, &tenant.tenant_id, current_time).await?;
        let transfers =
            get_latest_two_tenant_transfers(tenant_transfer_op, &tenant.tenant_id).await?;
        (Some(bank_account), Some(rewards_of_the_month), transfers)
    } else {
        (None, None, vec![])
    };
    Ok((
        StatusCode::OK,
        Json(
            RewardResult::build()
                .bank_account(payment_platform_results.0)
                .rewards_of_the_month(payment_platform_results.1)
                .latest_two_transfers(payment_platform_results.2)
                .finish(),
        ),
    ))
}

#[derive(Serialize, Debug)]
pub(crate) struct RewardResult {
    pub bank_account: Option<BankAccount>,
    pub rewards_of_the_month: Option<i32>, // 一ヶ月の報酬の合計。報酬 = 相談料 - プラットフォーム利用料。振込手数料は引かない。
    pub latest_two_transfers: Vec<Transfer>,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct Transfer {
    pub status: String,
    pub amount: i32,
    pub scheduled_date_in_jst: Ymd,
    // status == "paid"のときのみ存在
    pub transfer_amount: Option<i32>,
    pub transfer_date_in_jst: Option<Ymd>,
    // status == "carried_over"のときのみ存在
    pub carried_balance: Option<i32>,
}

impl RewardResult {
    fn build() -> RewardResultBuilder {
        RewardResultBuilder {
            bank_account: None,
            rewards_of_the_month: None,
            latest_two_transfers: vec![],
        }
    }
}

struct RewardResultBuilder {
    bank_account: Option<BankAccount>,
    rewards_of_the_month: Option<i32>,
    latest_two_transfers: Vec<Transfer>,
}

impl RewardResultBuilder {
    fn bank_account(mut self, bank_account: Option<BankAccount>) -> RewardResultBuilder {
        self.bank_account = bank_account;
        self
    }

    fn rewards_of_the_month(mut self, rewards_of_the_month: Option<i32>) -> RewardResultBuilder {
        self.rewards_of_the_month = rewards_of_the_month;
        self
    }

    fn latest_two_transfers(mut self, latest_two_transfers: Vec<Transfer>) -> RewardResultBuilder {
        self.latest_two_transfers = latest_two_transfers;
        self
    }

    fn finish(self) -> RewardResult {
        RewardResult {
            bank_account: self.bank_account,
            rewards_of_the_month: self.rewards_of_the_month,
            latest_two_transfers: self.latest_two_transfers,
        }
    }
}

async fn get_tenant_obj_by_tenant_id(
    tenant_op: impl TenantOperation,
    tenant_id: &str,
) -> Result<common::payment_platform::tenant::Tenant, ErrResp> {
    let tenant = tenant_op
        .get_tenant_by_tenant_id(tenant_id)
        .await
        .map_err(|e| match e {
            common::payment_platform::Error::RequestProcessingError(err) => {
                tracing::error!("failed to process request on getting tenant: {}", err);
                unexpected_err_resp()
            }
            common::payment_platform::Error::ApiError(err) => {
                tracing::error!("failed to request tenant operation: {}", err);
                let status_code = err.error.status as u16;
                if status_code == StatusCode::TOO_MANY_REQUESTS.as_u16() {
                    return (
                        StatusCode::TOO_MANY_REQUESTS,
                        Json(ApiError {
                            code: err_code::REACH_PAYMENT_PLATFORM_RATE_LIMIT,
                        }),
                    );
                }
                unexpected_err_resp()
            }
        })?;
    Ok(tenant)
}

async fn get_rewards_of_current_month(
    mut charge_op: impl ChargeOperation,
    tenant_id: &str,
    current_time: DateTime<FixedOffset>,
) -> Result<i32, ErrResp> {
    let current_year = current_time.year();
    let current_month = current_time.month();
    let (since_timestamp, until_timestamp) =
        create_start_and_end_timestamps_of_current_month(current_year, current_month);
    let search_charges_query = SearchChargesQuery::build()
        .limit(MAX_NUM_OF_CHARGES_PER_REQUEST)
        .since(since_timestamp)
        .until(until_timestamp)
        .tenant(tenant_id)
        .finish()
        .map_err(|e| {
            tracing::error!("failed to build search charges query: {}", e);
            unexpected_err_resp()
        })?;
    let mut has_more_charges = true;
    let mut rewards_of_the_month = 0;
    while has_more_charges {
        let charges = charge_op
            .search_charges(&search_charges_query)
            .await
            .map_err(|err| match err {
                common::payment_platform::Error::RequestProcessingError(err) => {
                    tracing::error!("failed to process request on getting charges: {}", err);
                    unexpected_err_resp()
                }
                common::payment_platform::Error::ApiError(err) => {
                    tracing::error!("failed to request charge operation: {}", err);
                    let status_code = err.error.status as u16;
                    if status_code == StatusCode::TOO_MANY_REQUESTS.as_u16() {
                        return (
                            StatusCode::TOO_MANY_REQUESTS,
                            Json(ApiError {
                                code: err_code::REACH_PAYMENT_PLATFORM_RATE_LIMIT,
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
        rewards_of_the_month += rewards;
        has_more_charges = charges.has_more;
    }
    Ok(rewards_of_the_month)
}

fn create_start_and_end_timestamps_of_current_month(
    current_year: i32,
    current_month: u32,
) -> (i64, i64) {
    let start_timestamp = chrono::Utc
        .ymd(current_year, current_month, 1)
        .and_hms(0, 0, 0)
        .timestamp();

    let (year_for_until, month_for_until) = if current_month >= 12 {
        (current_year + 1, 1)
    } else {
        (current_year, current_month + 1)
    };
    let end_timestamp = (chrono::Utc
        .ymd(year_for_until, month_for_until, 1)
        .and_hms(23, 59, 59)
        - Duration::days(1))
    .timestamp();

    (start_timestamp, end_timestamp)
}

// [tenantオブジェクト](https://pay.jp/docs/api/#tenant%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88)のpayjp_fee_includedがtrueであるとことを前提として実装
// payjp_fee_includedの値を設定できるのはテナント作成時のみ。そのためテナント作成時のコードで必ずtrueを設定することで、ここでtrueを前提として処理を行う
fn accumulate_rewards(sum: i32, charge: Charge) -> Result<i32, ErrResp> {
    let sales = charge.amount - charge.amount_refunded;
    if let Some(platform_fee_rate) = charge.platform_fee_rate.clone() {
        let fee = calculate_fee(sales, &platform_fee_rate)?;
        let reward_of_the_charge = sales - fee;
        if reward_of_the_charge < 0 {
            tracing::error!("negative reward_of_the_charge: {:?}", charge);
            return Err(unexpected_err_resp());
        }
        Ok(sum + reward_of_the_charge)
    } else {
        tracing::error!("No platform_fee_rate found in the charge: {:?}", charge);
        Err(unexpected_err_resp())
    }
}

// percentageはパーセンテージを示す少数の文字列。feeは、sales * (percentage/100) の結果の少数部分を切り捨てた値。
fn calculate_fee(sales: i32, percentage: &str) -> Result<i32, ErrResp> {
    let percentage_decimal = Decimal::from_str(percentage).map_err(|e| {
        tracing::error!("failed to parse percentage ({}): {}", percentage, e);
        unexpected_err_resp()
    })?;
    let one_handred_decimal = Decimal::from_str("100").map_err(|e| {
        tracing::error!("failed to parse str literal: {}", e);
        unexpected_err_resp()
    })?;
    let sales_decimal = match Decimal::from_i32(sales) {
        Some(s) => s,
        None => {
            tracing::error!("failed to parse sales value ({})", sales);
            return Err(unexpected_err_resp());
        }
    };
    let fee_decimal = (sales_decimal * (percentage_decimal / one_handred_decimal))
        .round_dp_with_strategy(0, RoundingStrategy::ToZero);
    let fee = fee_decimal.to_string().parse::<i32>().map_err(|e| {
        tracing::error!("failed to parse fee_decimal ({}): {}", fee_decimal, e);
        unexpected_err_resp()
    })?;
    Ok(fee)
}

async fn get_latest_two_tenant_transfers(
    tenant_transfer_op: impl TenantTransferOperation,
    tenant_id: &str,
) -> Result<Vec<Transfer>, ErrResp> {
    let search_tenant_transfers_query = SearchTenantTransfersQuery::build()
        .limit(MAX_NUM_OF_TENANT_TRANSFERS_PER_REQUEST)
        .tenant(tenant_id)
        .finish()
        .map_err(|e| {
            tracing::error!("failed to build search tenant transfers query: {}", e);
            unexpected_err_resp()
        })?;

    let tenant_transfers = tenant_transfer_op
        .search_tenant_transfers(&search_tenant_transfers_query)
        .await
        .map_err(|err| match err {
            common::payment_platform::Error::RequestProcessingError(err) => {
                tracing::error!("failed to process request on getting charges: {}", err);
                unexpected_err_resp()
            }
            common::payment_platform::Error::ApiError(err) => {
                tracing::error!("failed to request charge operation: {}", err);
                let status_code = err.error.status as u16;
                if status_code == StatusCode::TOO_MANY_REQUESTS.as_u16() {
                    return (
                        StatusCode::TOO_MANY_REQUESTS,
                        Json(ApiError {
                            code: err_code::REACH_PAYMENT_PLATFORM_RATE_LIMIT,
                        }),
                    );
                }
                unexpected_err_resp()
            }
        })?;

    let mut transfers = vec![];
    for tenant_transfer in tenant_transfers.data {
        let transfer = convert_tenant_transfer_to_transfer(tenant_transfer)?;
        transfers.push(transfer);
    }
    Ok(transfers)
}

fn convert_tenant_transfer_to_transfer(
    tenant_transfer: TenantTransfer,
) -> Result<Transfer, ErrResp> {
    let scheduled_date = NaiveDate::parse_from_str(&tenant_transfer.scheduled_date, "%Y-%m-%d")
        .map_err(|e| {
            tracing::error!(
                "failed to parse scheduled_date {}: {}",
                tenant_transfer.scheduled_date,
                e
            );
            unexpected_err_resp()
        })?;
    let scheduled_date_in_jst = Ymd {
        year: scheduled_date.year(),
        month: scheduled_date.month(),
        day: scheduled_date.day(),
    };
    let transfer_date_in_jst = if let Some(d) = tenant_transfer.transfer_date {
        let parsed_date = NaiveDate::parse_from_str(&d, "%Y-%m-%d").map_err(|e| {
            tracing::error!("failed to parse transfer_date {}: {}", d, e);
            unexpected_err_resp()
        })?;
        let date = Ymd {
            year: parsed_date.year(),
            month: parsed_date.month(),
            day: parsed_date.day(),
        };
        Some(date)
    } else {
        None
    };
    Ok(Transfer {
        status: tenant_transfer.status,
        amount: tenant_transfer.amount,
        scheduled_date_in_jst,
        transfer_amount: tenant_transfer.transfer_amount,
        transfer_date_in_jst,
        carried_balance: tenant_transfer.carried_balance,
    })
}

trait RewardOperation {
    fn find_tenant_by_user_account_id(&self, id: i32) -> Result<Option<Tenant>, ErrResp>;
}

struct RewardOperationImpl {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl RewardOperationImpl {
    fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { conn }
    }
}

impl RewardOperation for RewardOperationImpl {
    fn find_tenant_by_user_account_id(&self, id: i32) -> Result<Option<Tenant>, ErrResp> {
        let result = tenant_table.find(id).first::<Tenant>(&self.conn);
        match result {
            Ok(tenant) => Ok(Some(tenant)),
            Err(e) => {
                if e == NotFound {
                    Ok(None)
                } else {
                    Err(unexpected_err_resp())
                }
            }
        }
    }
}

// TODO: 事前準備に用意するデータに関して、データの追加、編集でvalidatorを実装した後、それを使ってチェックを行うよう修正する
// TODO: payjp_fee_includedがtrueのテナントで作成された入金データを実際のテスト環境から取得し、それを使ってコードを改善する
#[cfg(test)]
mod tests {

    use async_session::async_trait;
    use axum::http::StatusCode;
    use chrono::{TimeZone, Utc};
    use common::{
        model::user::Tenant,
        payment_platform::{
            charge::{Charge, ChargeOperation, Query as SearchChargesQuery},
            customer::Card,
            tenant::{ReviewedBrands, TenantOperation},
            tenant_transfer::{
                Query as SearchTenantTransfersQuery, Summary, TenantTransfer,
                TenantTransferOperation,
            },
            ErrorDetail, ErrorInfo, List,
        },
    };

    use crate::{
        err_code,
        rewards::Transfer,
        util::{BankAccount, Ymd, JAPANESE_TIME_ZONE},
    };

    use super::{get_reward_internal, RewardOperation};

    struct RewardOperationMock {
        tenant_option: Option<Tenant>,
    }

    impl RewardOperation for RewardOperationMock {
        fn find_tenant_by_user_account_id(
            &self,
            _id: i32,
        ) -> Result<Option<Tenant>, common::ErrResp> {
            Ok(self.tenant_option.clone())
        }
    }

    struct TenantOperationMock {
        tenant: common::payment_platform::tenant::Tenant,
        too_many_requests: bool,
    }

    #[async_trait]
    impl TenantOperation for TenantOperationMock {
        async fn get_tenant_by_tenant_id(
            &self,
            _tenant_id: &str,
        ) -> Result<common::payment_platform::tenant::Tenant, common::payment_platform::Error>
        {
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
            return Ok(self.tenant.clone());
        }
    }

    struct ChargeOperationMock {
        num_of_search_trial: usize,
        lists: Vec<List<Charge>>,
        too_many_requests: bool,
    }

    #[async_trait]
    impl ChargeOperation for ChargeOperationMock {
        async fn search_charges(
            &mut self,
            _query: &SearchChargesQuery,
        ) -> Result<List<Charge>, common::payment_platform::Error> {
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
    }

    struct TenantTransferOperationMock {
        tenant_transfers: List<TenantTransfer>,
        too_many_requests: bool,
    }

    #[async_trait]
    impl TenantTransferOperation for TenantTransferOperationMock {
        async fn search_tenant_transfers(
            &self,
            _query: &SearchTenantTransfersQuery,
        ) -> Result<List<TenantTransfer>, common::payment_platform::Error> {
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
            Ok(self.tenant_transfers.clone())
        }
    }

    #[tokio::test]
    async fn return_empty_rewards() {
        let account_id = 9853;
        let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
        let reward_op = RewardOperationMock {
            tenant_option: None,
        };
        let tenant = create_dummy_tenant(tenant_id);
        let tenant_op = TenantOperationMock {
            tenant,
            too_many_requests: false,
        };
        let charge_op = ChargeOperationMock {
            num_of_search_trial: 0,
            lists: vec![List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/charges".to_string(),
                data: vec![],
                count: 0,
            }],
            too_many_requests: false,
        };
        let tenant_transfer_op = TenantTransferOperationMock {
            tenant_transfers: List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/tenant_transfers".to_string(),
                data: vec![],
                count: 0,
            },
            too_many_requests: false,
        };
        let current_datetime = Utc
            .ymd(2021, 12, 31)
            .and_hms(14, 59, 59)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned());

        let result = get_reward_internal(
            account_id,
            reward_op,
            tenant_op,
            charge_op,
            current_datetime,
            tenant_transfer_op,
        )
        .await
        .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(None, result.1 .0.bank_account);
        assert_eq!(None, result.1 .0.rewards_of_the_month);
        let empty = Vec::<Transfer>::with_capacity(0);
        assert_eq!(empty, result.1 .0.latest_two_transfers);
    }

    fn create_dummy_tenant(tenant_id: &str) -> common::payment_platform::tenant::Tenant {
        let reviewed_brands = vec![
            ReviewedBrands {
                brand: "Visa".to_string(),
                status: "passed".to_string(),
                available_date: Some(1626016999),
            },
            ReviewedBrands {
                brand: "MasterCard".to_string(),
                status: "passed".to_string(),
                available_date: Some(1626016999),
            },
            ReviewedBrands {
                brand: "JCB".to_string(),
                status: "passed".to_string(),
                available_date: Some(1626016999),
            },
            ReviewedBrands {
                brand: "AmericanExpress".to_string(),
                status: "passed".to_string(),
                available_date: Some(1626016999),
            },
            ReviewedBrands {
                brand: "DinersClub".to_string(),
                status: "passed".to_string(),
                available_date: Some(1626016999),
            },
        ];
        common::payment_platform::tenant::Tenant {
            id: tenant_id.to_string(),
            name: "タナカ　タロウ".to_string(),
            object: "tenant".to_string(),
            livemode: false,
            created: 1626016999,
            platform_fee_rate: "10.15".to_string(),
            payjp_fee_included: true,
            minimum_transfer_amount: 1000,
            bank_code: "0001".to_string(),
            bank_branch_code: "123".to_string(),
            bank_account_type: "普通".to_string(),
            bank_account_number: "1111222".to_string(),
            bank_account_holder_name: "タナカ　タロウ".to_string(),
            bank_account_status: "pending".to_string(),
            currencies_supported: vec!["jpy".to_string()],
            default_currency: "jpy".to_string(),
            reviewed_brands,
            metadata: None,
        }
    }

    #[tokio::test]
    async fn fail_tenant_too_many_requests() {
        let account_id = 9853;
        let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
        let reward_op = RewardOperationMock {
            tenant_option: Some(Tenant {
                user_account_id: account_id,
                tenant_id: "c8f0aa44901940849cbdb8b3e7d9f305".to_string(),
            }),
        };
        let tenant = create_dummy_tenant(tenant_id);
        let tenant_op = TenantOperationMock {
            tenant,
            too_many_requests: true,
        };
        let charge_op = ChargeOperationMock {
            num_of_search_trial: 0,
            lists: vec![List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/charges".to_string(),
                data: vec![],
                count: 0,
            }],
            too_many_requests: false,
        };
        let tenant_transfer_op = TenantTransferOperationMock {
            tenant_transfers: List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/tenant_transfers".to_string(),
                data: vec![],
                count: 0,
            },
            too_many_requests: false,
        };
        let current_datetime = Utc
            .ymd(2021, 12, 31)
            .and_hms(14, 59, 59)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned());

        let result = get_reward_internal(
            account_id,
            reward_op,
            tenant_op,
            charge_op,
            current_datetime,
            tenant_transfer_op,
        )
        .await
        .expect_err("failed to get Err");

        assert_eq!(StatusCode::TOO_MANY_REQUESTS, result.0);
        assert_eq!(
            err_code::REACH_PAYMENT_PLATFORM_RATE_LIMIT,
            result.1 .0.code
        );
    }

    #[tokio::test]
    async fn fail_charges_too_many_requests() {
        let account_id = 9853;
        let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
        let reward_op = RewardOperationMock {
            tenant_option: Some(Tenant {
                user_account_id: account_id,
                tenant_id: "c8f0aa44901940849cbdb8b3e7d9f305".to_string(),
            }),
        };
        let tenant = create_dummy_tenant(tenant_id);
        let tenant_op = TenantOperationMock {
            tenant,
            too_many_requests: false,
        };
        let charge_op = ChargeOperationMock {
            num_of_search_trial: 0,
            lists: vec![List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/charges".to_string(),
                data: vec![],
                count: 0,
            }],
            too_many_requests: true,
        };
        let tenant_transfer_op = TenantTransferOperationMock {
            tenant_transfers: List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/tenant_transfers".to_string(),
                data: vec![],
                count: 0,
            },
            too_many_requests: false,
        };
        let current_datetime = Utc
            .ymd(2021, 12, 31)
            .and_hms(14, 59, 59)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned());

        let result = get_reward_internal(
            account_id,
            reward_op,
            tenant_op,
            charge_op,
            current_datetime,
            tenant_transfer_op,
        )
        .await
        .expect_err("failed to get Err");

        assert_eq!(StatusCode::TOO_MANY_REQUESTS, result.0);
        assert_eq!(
            err_code::REACH_PAYMENT_PLATFORM_RATE_LIMIT,
            result.1 .0.code
        );
    }

    #[tokio::test]
    async fn fail_tenant_transfers_too_many_requests() {
        let account_id = 9853;
        let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
        let reward_op = RewardOperationMock {
            tenant_option: Some(Tenant {
                user_account_id: account_id,
                tenant_id: "c8f0aa44901940849cbdb8b3e7d9f305".to_string(),
            }),
        };
        let tenant = create_dummy_tenant(tenant_id);
        let tenant_op = TenantOperationMock {
            tenant,
            too_many_requests: false,
        };
        let charge_op = ChargeOperationMock {
            num_of_search_trial: 0,
            lists: vec![List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/charges".to_string(),
                data: vec![],
                count: 0,
            }],
            too_many_requests: false,
        };
        let tenant_transfer_op = TenantTransferOperationMock {
            tenant_transfers: List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/tenant_transfers".to_string(),
                data: vec![],
                count: 0,
            },
            too_many_requests: true,
        };
        let current_datetime = Utc
            .ymd(2021, 12, 31)
            .and_hms(14, 59, 59)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned());

        let result = get_reward_internal(
            account_id,
            reward_op,
            tenant_op,
            charge_op,
            current_datetime,
            tenant_transfer_op,
        )
        .await
        .expect_err("failed to get Err");

        assert_eq!(StatusCode::TOO_MANY_REQUESTS, result.0);
        assert_eq!(
            err_code::REACH_PAYMENT_PLATFORM_RATE_LIMIT,
            result.1 .0.code
        );
    }

    #[tokio::test]
    async fn return_reward_with_tenant_1charge_1tenant_transfer() {
        let account_id = 9853;
        let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
        let reward_op = RewardOperationMock {
            tenant_option: Some(Tenant {
                user_account_id: account_id,
                tenant_id: tenant_id.to_string(),
            }),
        };
        let tenant = create_dummy_tenant(tenant_id);
        let tenant_op = TenantOperationMock {
            tenant: tenant.clone(),
            too_many_requests: false,
        };
        let charge_id = "ch_7fb5aea258910da9a756985cbe51f";
        let charge = create_dummy_charge(charge_id, tenant_id);
        let charge_op = ChargeOperationMock {
            num_of_search_trial: 0,
            lists: vec![List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/charges".to_string(),
                data: vec![charge.clone()],
                count: 1,
            }],
            too_many_requests: false,
        };
        let transfer_id = "ten_tr_920fdff2a571ace3441bd78b3";
        let tenant_transfer = create_dummy_tenant_transfer1(transfer_id, tenant_id);
        let tenant_transfer_op = TenantTransferOperationMock {
            tenant_transfers: List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/tenant_transfers".to_string(),
                data: vec![tenant_transfer.clone()],
                count: 1,
            },
            too_many_requests: false,
        };
        let current_datetime = Utc
            .ymd(2021, 12, 31)
            .and_hms(14, 59, 59)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned());

        let result = get_reward_internal(
            account_id,
            reward_op,
            tenant_op,
            charge_op,
            current_datetime,
            tenant_transfer_op,
        )
        .await
        .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        let bank_account = BankAccount {
            bank_code: tenant.bank_code.to_string(),
            branch_code: tenant.bank_branch_code.to_string(),
            account_type: tenant.bank_account_type.to_string(),
            account_number: tenant.bank_account_number.to_string(),
            account_holder_name: tenant.bank_account_holder_name.to_string(),
        };
        assert_eq!(Some(bank_account), result.1 .0.bank_account);
        // create_dummy_chargeから計算される値。詳細は下記の通り。
        // 売上 = amount - refunded_amount
        // 手数料 = 売上 * platform_fee_rate/100 (小数点切り捨て)
        // 結果 = 売上 - 手数料
        // なので
        // 売上 = 4000 - 1000 = 3000
        // 手数料 = 3000 * 10.15/100 = 304
        // 結果 = 3000 - 304 = 2696
        // 本テストでは、chargeは一つなので2696で確定
        assert_eq!(Some(2696), result.1 .0.rewards_of_the_month);
        // create_dummy_transfer1から導出される結果
        let transfer = Transfer {
            status: "pending".to_string(),
            amount: 2696,
            scheduled_date_in_jst: Ymd {
                year: 2022,
                month: 1,
                day: 31,
            },
            transfer_amount: None,
            transfer_date_in_jst: None,
            carried_balance: Some(0),
        };
        assert_eq!(vec![transfer], result.1 .0.latest_two_transfers);
    }

    fn create_dummy_charge(charge_id: &str, tenant_id: &str) -> Charge {
        Charge {
            id: charge_id.to_string(),
            object: "charge".to_string(),
            livemode: false,
            created: 1639931415,
            amount: 4000,
            currency: "jpy".to_string(),
            paid: true,
            expired_at: None,
            captured: true,
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
            fee_rate: Some("3.00".to_string()),
            refunded: true,
            amount_refunded: 1000,
            refund_reason: Some("テスト".to_string()),
            subscription: None,
            metadata: None,
            platform_fee: None,
            tenant: Some(tenant_id.to_string()),
            platform_fee_rate: Some("10.15".to_string()),
            total_platform_fee: Some(214),
        }
    }

    fn create_dummy_tenant_transfer1(transfer_id: &str, tenant_id: &str) -> TenantTransfer {
        let charge_id = "ch_7fb5aea258910da9a756985cbe51f";
        TenantTransfer {
            object: "tenant_transfer".to_string(),
            id: transfer_id.to_string(),
            livemode: false,
            created: 1641055119,
            amount: 2696,
            currency: "jpy".to_string(),
            status: "pending".to_string(),
            charges: List {
                object: "list".to_string(),
                has_more: false,
                url: format!("/v1/tenant_transfers/{}/charges", transfer_id),
                data: vec![create_dummy_charge(charge_id, tenant_id)],
                count: 1,
            },
            scheduled_date: "2022-01-31".to_string(),
            summary: Summary {
                charge_count: 1,
                charge_fee: 90,
                charge_gross: 4000,
                net: 3696,
                refund_amount: 1000,
                refund_count: 1,
                dispute_amount: 0,
                dispute_count: 0,
                total_platform_fee: 214,
            },
            term_start: 1638284400,
            term_end: 1640962800,
            transfer_amount: None,
            transfer_date: None,
            carried_balance: Some(0),
            tenant_id: tenant_id.to_string(),
        }
    }

    #[tokio::test]
    async fn check_non_captured_charge_is_filterd() {
        let account_id = 9853;
        let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
        let reward_op = RewardOperationMock {
            tenant_option: Some(Tenant {
                user_account_id: account_id,
                tenant_id: tenant_id.to_string(),
            }),
        };
        let tenant = create_dummy_tenant(tenant_id);
        let tenant_op = TenantOperationMock {
            tenant: tenant.clone(),
            too_many_requests: false,
        };
        let charge1_id = "ch_7fb5aea258910da9a756985cbe51f";
        let charge1 = create_dummy_charge(charge1_id, tenant_id);
        let charge2_id = "ch_7fb5aea258910da9a756985cbe51g";
        let charge2 = create_non_captured_dummy_charge(charge2_id, tenant_id);
        let charge_op = ChargeOperationMock {
            num_of_search_trial: 0,
            lists: vec![List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/charges".to_string(),
                data: vec![charge1.clone(), charge2.clone()],
                count: 1,
            }],
            too_many_requests: false,
        };
        let transfer_id = "ten_tr_920fdff2a571ace3441bd78b3";
        let tenant_transfer = create_dummy_tenant_transfer1(transfer_id, tenant_id);
        let tenant_transfer_op = TenantTransferOperationMock {
            tenant_transfers: List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/tenant_transfers".to_string(),
                data: vec![tenant_transfer.clone()],
                count: 1,
            },
            too_many_requests: false,
        };
        let current_datetime = Utc
            .ymd(2021, 12, 31)
            .and_hms(14, 59, 59)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned());

        let result = get_reward_internal(
            account_id,
            reward_op,
            tenant_op,
            charge_op,
            current_datetime,
            tenant_transfer_op,
        )
        .await
        .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        let bank_account = BankAccount {
            bank_code: tenant.bank_code.to_string(),
            branch_code: tenant.bank_branch_code.to_string(),
            account_type: tenant.bank_account_type.to_string(),
            account_number: tenant.bank_account_number.to_string(),
            account_holder_name: tenant.bank_account_holder_name.to_string(),
        };
        assert_eq!(Some(bank_account), result.1 .0.bank_account);
        assert_eq!(Some(2696), result.1 .0.rewards_of_the_month);
        // create_dummy_transfer1から導出される結果
        let transfer = Transfer {
            status: "pending".to_string(),
            amount: 2696,
            scheduled_date_in_jst: Ymd {
                year: 2022,
                month: 1,
                day: 31,
            },
            transfer_amount: None,
            transfer_date_in_jst: None,
            carried_balance: Some(0),
        };
        assert_eq!(vec![transfer], result.1 .0.latest_two_transfers);
    }

    fn create_non_captured_dummy_charge(charge_id: &str, tenant_id: &str) -> Charge {
        Charge {
            id: charge_id.to_string(),
            object: "charge".to_string(),
            livemode: false,
            created: 1639931415,
            amount: 4000,
            currency: "jpy".to_string(),
            paid: true,
            expired_at: None,
            captured: false,
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
            fee_rate: Some("3.00".to_string()),
            refunded: true,
            amount_refunded: 1000,
            refund_reason: Some("テスト".to_string()),
            subscription: None,
            metadata: None,
            platform_fee: None,
            tenant: Some(tenant_id.to_string()),
            platform_fee_rate: Some("10.15".to_string()),
            total_platform_fee: Some(214),
        }
    }

    #[tokio::test]
    async fn return_reward_with_tenant_32charge_1tenant_transfer() {
        let account_id = 9853;
        let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
        let reward_op = RewardOperationMock {
            tenant_option: Some(Tenant {
                user_account_id: account_id,
                tenant_id: tenant_id.to_string(),
            }),
        };
        let tenant = create_dummy_tenant(tenant_id);
        let tenant_op = TenantOperationMock {
            tenant: tenant.clone(),
            too_many_requests: false,
        };
        let charges = create_dummy_32charges(tenant_id);
        let charge_op = ChargeOperationMock {
            num_of_search_trial: 0,
            lists: vec![List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/charges".to_string(),
                data: charges,
                count: 32,
            }],
            too_many_requests: false,
        };
        let transfer_id = "ten_tr_920fdff2a571ace3441bd78b3";
        let tenant_transfer = create_dummy_tenant_transfer1(transfer_id, tenant_id);
        let tenant_transfer_op = TenantTransferOperationMock {
            tenant_transfers: List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/tenant_transfers".to_string(),
                data: vec![tenant_transfer.clone()],
                count: 1,
            },
            too_many_requests: false,
        };
        let current_datetime = Utc
            .ymd(2021, 12, 31)
            .and_hms(14, 59, 59)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned());

        let result = get_reward_internal(
            account_id,
            reward_op,
            tenant_op,
            charge_op,
            current_datetime,
            tenant_transfer_op,
        )
        .await
        .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        let bank_account = BankAccount {
            bank_code: tenant.bank_code.to_string(),
            branch_code: tenant.bank_branch_code.to_string(),
            account_type: tenant.bank_account_type.to_string(),
            account_number: tenant.bank_account_number.to_string(),
            account_holder_name: tenant.bank_account_holder_name.to_string(),
        };
        assert_eq!(Some(bank_account), result.1 .0.bank_account);
        assert_eq!(Some(2696 * 32), result.1 .0.rewards_of_the_month);
        // create_dummy_transfer1から導出される結果
        let transfer = Transfer {
            status: "pending".to_string(),
            amount: 2696,
            scheduled_date_in_jst: Ymd {
                year: 2022,
                month: 1,
                day: 31,
            },
            transfer_amount: None,
            transfer_date_in_jst: None,
            carried_balance: Some(0),
        };
        assert_eq!(vec![transfer], result.1 .0.latest_two_transfers);
    }

    fn create_dummy_32charges(tenant_id: &str) -> Vec<Charge> {
        let mut charges = Vec::with_capacity(32);
        for i in 0..32 {
            let charge = Charge {
                id: format!("ch_7fb5aea258910da9a756985cbe5{:02}", i),
                object: "charge".to_string(),
                livemode: false,
                created: 1639931415,
                amount: 4000,
                currency: "jpy".to_string(),
                paid: true,
                expired_at: None,
                captured: true,
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
                fee_rate: Some("3.00".to_string()),
                refunded: true,
                amount_refunded: 1000,
                refund_reason: Some("テスト".to_string()),
                subscription: None,
                metadata: None,
                platform_fee: None,
                tenant: Some(tenant_id.to_string()),
                platform_fee_rate: Some("10.15".to_string()),
                total_platform_fee: Some(214),
            };
            charges.push(charge);
        }
        charges
    }

    #[tokio::test]
    async fn return_reward_with_tenant_33charge_1tenant_transfer() {
        let account_id = 9853;
        let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
        let reward_op = RewardOperationMock {
            tenant_option: Some(Tenant {
                user_account_id: account_id,
                tenant_id: tenant_id.to_string(),
            }),
        };
        let tenant = create_dummy_tenant(tenant_id);
        let tenant_op = TenantOperationMock {
            tenant: tenant.clone(),
            too_many_requests: false,
        };
        let charges = create_dummy_32charges(tenant_id);
        let charge_id = "ch_7fb5aea258910da9a756985cbe51f";
        let charge = create_dummy_charge(charge_id, tenant_id);
        let charge_op = ChargeOperationMock {
            num_of_search_trial: 0,
            lists: vec![
                List {
                    object: "list".to_string(),
                    has_more: true,
                    url: "/v1/charges".to_string(),
                    data: charges,
                    count: 32,
                },
                List {
                    object: "list".to_string(),
                    has_more: false,
                    url: "/v1/charges".to_string(),
                    data: vec![charge],
                    count: 1,
                },
            ],
            too_many_requests: false,
        };
        let transfer_id = "ten_tr_920fdff2a571ace3441bd78b3";
        let tenant_transfer = create_dummy_tenant_transfer1(transfer_id, tenant_id);
        let tenant_transfer_op = TenantTransferOperationMock {
            tenant_transfers: List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/tenant_transfers".to_string(),
                data: vec![tenant_transfer.clone()],
                count: 1,
            },
            too_many_requests: false,
        };
        let current_datetime = Utc
            .ymd(2021, 12, 31)
            .and_hms(14, 59, 59)
            .with_timezone(&JAPANESE_TIME_ZONE.to_owned());

        let result = get_reward_internal(
            account_id,
            reward_op,
            tenant_op,
            charge_op,
            current_datetime,
            tenant_transfer_op,
        )
        .await
        .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        let bank_account = BankAccount {
            bank_code: tenant.bank_code.to_string(),
            branch_code: tenant.bank_branch_code.to_string(),
            account_type: tenant.bank_account_type.to_string(),
            account_number: tenant.bank_account_number.to_string(),
            account_holder_name: tenant.bank_account_holder_name.to_string(),
        };
        assert_eq!(Some(bank_account), result.1 .0.bank_account);
        assert_eq!(Some(2696 * 33), result.1 .0.rewards_of_the_month);
        // create_dummy_transfer1から導出される結果
        let transfer = Transfer {
            status: "pending".to_string(),
            amount: 2696,
            scheduled_date_in_jst: Ymd {
                year: 2022,
                month: 1,
                day: 31,
            },
            transfer_amount: None,
            transfer_date_in_jst: None,
            carried_balance: Some(0),
        };
        assert_eq!(vec![transfer], result.1 .0.latest_two_transfers);
    }

    // transferが2つのパターン
    // transferが3つのパターンはいらない。なぜなら2つ返すのはpayjpの責務だから。
}
