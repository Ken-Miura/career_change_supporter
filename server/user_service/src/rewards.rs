// Copyright 2021 Ken Miura

use axum::async_trait;
use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, TimeZone, Utc};
use common::payment_platform::charge::Charge;
use common::util::Ymd;
use common::JAPANESE_TIME_ZONE;
use common::{
    payment_platform::{
        charge::ChargeOperationImpl,
        tenant::{TenantOperation, TenantOperationImpl},
        tenant_transfer::{
            Query as SearchTenantTransfersQuery, TenantTransfer, TenantTransferOperation,
            TenantTransferOperationImpl,
        },
    },
    ApiError, ErrResp, RespResult,
};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::Serialize;
use tracing::{error, info};

use crate::util::create_start_and_end_timestamps_of_current_year;
use crate::util::rewards::{calculate_rewards, get_charges, MAX_NUM_OF_CHARGES_PER_REQUEST};
use crate::{
    err::{self, unexpected_err_resp},
    util::{session::User, BankAccount, ACCESS_INFO},
};

const MAX_NUM_OF_TENANT_TRANSFERS_PER_REQUEST: u32 = 2;

pub(crate) async fn get_reward(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<RewardResult> {
    let reward_op = RewardOperationImpl::new(pool);
    let tenant_op = TenantOperationImpl::new(&ACCESS_INFO);
    let current_datetime = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let tenant_transfer_op = TenantTransferOperationImpl::new(&ACCESS_INFO);
    handle_reward_req(
        account_id,
        reward_op,
        tenant_op,
        current_datetime,
        tenant_transfer_op,
    )
    .await
}

async fn handle_reward_req(
    account_id: i64,
    reward_op: impl RewardOperation,
    tenant_op: impl TenantOperation,
    current_time: DateTime<FixedOffset>,
    tenant_transfer_op: impl TenantTransferOperation,
) -> RespResult<RewardResult> {
    let tenant_id_option = reward_op.find_tenant_id_by_account_id(account_id).await?;
    let payment_platform_results = if let Some(tenant_id) = tenant_id_option {
        info!("tenant found (tenant id: {})", tenant_id);
        let tenant_obj = get_tenant_obj_by_tenant_id(tenant_op, &tenant_id).await?;
        if !tenant_obj.payjp_fee_included {
            error!("payjp_fee_included is false (tenant id: {})", &tenant_id);
            return Err(unexpected_err_resp());
        }
        let bank_account = BankAccount {
            bank_code: tenant_obj.bank_code,
            branch_code: tenant_obj.bank_branch_code,
            account_type: tenant_obj.bank_account_type,
            account_number: tenant_obj.bank_account_number,
            account_holder_name: tenant_obj.bank_account_holder_name,
        };

        let (current_month_since_timestamp, current_month_until_timestamp) =
            create_start_and_end_timestamps_of_current_month(
                current_time.year(),
                current_time.month(),
            );
        let charges_of_the_month = reward_op
            .get_charges_of_the_month(
                current_month_since_timestamp,
                current_month_until_timestamp,
                tenant_id.as_str(),
            )
            .await?;
        let rewards_of_the_month = calculate_rewards(&charges_of_the_month)?;

        let (current_year_since_timestamp, current_year_until_timestamp) =
            create_start_and_end_timestamps_of_current_year(current_time.year());
        let charges_of_the_year = reward_op
            .get_charges_of_the_year(
                current_year_since_timestamp,
                current_year_until_timestamp,
                tenant_id.as_str(),
            )
            .await?;
        let rewards_of_the_year = calculate_rewards(&charges_of_the_year)?;

        let transfers = get_latest_two_tenant_transfers(tenant_transfer_op, &tenant_id).await?;
        (
            Some(bank_account),
            Some(rewards_of_the_month),
            Some(rewards_of_the_year),
            transfers,
        )
    } else {
        info!("no tenant found (account id: {})", account_id);
        (None, None, None, vec![])
    };

    Ok((
        StatusCode::OK,
        Json(
            RewardResult::build()
                .bank_account(payment_platform_results.0)
                .rewards_of_the_month(payment_platform_results.1)
                .rewards_of_the_year(payment_platform_results.2)
                .latest_two_transfers(payment_platform_results.3)
                .finish(),
        ),
    ))
}

#[derive(Serialize, Debug)]
pub(crate) struct RewardResult {
    pub(crate) bank_account: Option<BankAccount>,
    pub(crate) rewards_of_the_month: Option<i32>, // 一ヶ月の報酬の合計。報酬 = 相談料 - プラットフォーム利用料。振込手数料は引かない。
    pub(crate) rewards_of_the_year: Option<i32>, // 1年間（1月-12月）の報酬の合計。報酬 = 相談料 - プラットフォーム利用料。振込手数料は引かない。
    pub(crate) latest_two_transfers: Vec<Transfer>,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct Transfer {
    pub(crate) status: String,
    pub(crate) amount: i32,
    pub(crate) scheduled_date_in_jst: Ymd,
    // status == "paid"のときのみ存在
    pub(crate) transfer_amount: Option<i32>,
    pub(crate) transfer_date_in_jst: Option<Ymd>,
    // status == "carried_over"のときのみ存在
    pub(crate) carried_balance: Option<i32>,
}

impl RewardResult {
    fn build() -> RewardResultBuilder {
        RewardResultBuilder {
            bank_account: None,
            rewards_of_the_month: None,
            rewards_of_the_year: None,
            latest_two_transfers: vec![],
        }
    }
}

struct RewardResultBuilder {
    bank_account: Option<BankAccount>,
    rewards_of_the_month: Option<i32>,
    rewards_of_the_year: Option<i32>,
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

    fn rewards_of_the_year(mut self, rewards_of_the_year: Option<i32>) -> RewardResultBuilder {
        self.rewards_of_the_year = rewards_of_the_year;
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
            rewards_of_the_year: self.rewards_of_the_year,
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
                error!("failed to process request on getting tenant: {}", err);
                unexpected_err_resp()
            }
            common::payment_platform::Error::ApiError(err) => {
                error!("failed to request tenant operation: {}", err);
                let status_code = err.error.status as u16;
                if status_code == StatusCode::TOO_MANY_REQUESTS.as_u16() {
                    return (
                        StatusCode::TOO_MANY_REQUESTS,
                        Json(ApiError {
                            code: err::Code::ReachPaymentPlatformRateLimit as u32,
                        }),
                    );
                }
                unexpected_err_resp()
            }
        })?;
    Ok(tenant)
}

fn create_start_and_end_timestamps_of_current_month(
    current_year: i32,
    current_month: u32,
) -> (i64, i64) {
    let start_timestamp = JAPANESE_TIME_ZONE
        .ymd(current_year, current_month, 1)
        .and_hms(0, 0, 0)
        .timestamp();

    let (year_for_until, month_for_until) = if current_month >= 12 {
        (current_year + 1, 1)
    } else {
        (current_year, current_month + 1)
    };
    let end_timestamp = (JAPANESE_TIME_ZONE
        .ymd(year_for_until, month_for_until, 1)
        .and_hms(23, 59, 59)
        - Duration::days(1))
    .timestamp();

    (start_timestamp, end_timestamp)
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
            error!("failed to build search tenant transfers query: {}", e);
            unexpected_err_resp()
        })?;

    let tenant_transfers = tenant_transfer_op
        .search_tenant_transfers(&search_tenant_transfers_query)
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
                            code: err::Code::ReachPaymentPlatformRateLimit as u32,
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
            error!(
                "failed to parse scheduled_date {}: {}",
                tenant_transfer.scheduled_date, e
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
            error!("failed to parse transfer_date {}: {}", d, e);
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

#[async_trait]
trait RewardOperation {
    async fn find_tenant_id_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<String>, ErrResp>;

    async fn get_charges_of_the_month(
        &self,
        since_timestamp: i64,
        until_timestamp: i64,
        tenant_id: &str,
    ) -> Result<Vec<Charge>, ErrResp>;

    async fn get_charges_of_the_year(
        &self,
        since_timestamp: i64,
        until_timestamp: i64,
        tenant_id: &str,
    ) -> Result<Vec<Charge>, ErrResp>;
}

struct RewardOperationImpl {
    pool: DatabaseConnection,
}

impl RewardOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RewardOperation for RewardOperationImpl {
    async fn find_tenant_id_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<String>, ErrResp> {
        let model = entity::prelude::Tenant::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!("failed to find tenant (account_id: {}): {}", account_id, e);
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.tenant_id))
    }

    async fn get_charges_of_the_month(
        &self,
        since_timestamp: i64,
        until_timestamp: i64,
        tenant_id: &str,
    ) -> Result<Vec<Charge>, ErrResp> {
        let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
        let result = get_charges(
            charge_op,
            MAX_NUM_OF_CHARGES_PER_REQUEST,
            since_timestamp,
            until_timestamp,
            tenant_id,
        )
        .await?;
        Ok(result)
    }

    async fn get_charges_of_the_year(
        &self,
        since_timestamp: i64,
        until_timestamp: i64,
        tenant_id: &str,
    ) -> Result<Vec<Charge>, ErrResp> {
        let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
        let result = get_charges(
            charge_op,
            MAX_NUM_OF_CHARGES_PER_REQUEST,
            since_timestamp,
            until_timestamp,
            tenant_id,
        )
        .await?;
        Ok(result)
    }
}

// TODO: 事前準備に用意するデータに関して、データの追加、編集でvalidatorを実装した後、それを使ってチェックを行うよう修正する
#[cfg(test)]
mod tests {

    // TODO: rewrite tests
    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use common::{
        payment_platform::{
            charge::Charge,
            tenant::{CreateTenant, TenantOperation, UpdateTenant},
            tenant_transfer::{
                Query as SearchTenantTransfersQuery, TenantTransfer, TenantTransferOperation,
            },
            ErrorDetail, ErrorInfo, List,
        },
        ApiError, ErrResp,
    };

    use crate::err::Code;

    use super::RewardOperation;

    struct RewardOperationMock {
        tenant_id_option: Option<String>,
        too_many_requests: bool,
        month_since_timestamp: i64,
        month_until_timestamp: i64,
        // rewards_of_the_month: i32,
        year_since_timestamp: i64,
        year_until_timestamp: i64,
        // rewards_of_the_year: i32,
    }

    #[async_trait]
    impl RewardOperation for RewardOperationMock {
        async fn find_tenant_id_by_account_id(
            &self,
            _account_id: i64,
        ) -> Result<Option<String>, ErrResp> {
            Ok(self.tenant_id_option.clone())
        }

        async fn get_charges_of_the_month(
            &self,
            since_timestamp: i64,
            until_timestamp: i64,
            tenant_id: &str,
        ) -> Result<Vec<Charge>, ErrResp> {
            assert_eq!(self.month_since_timestamp, since_timestamp);
            assert_eq!(self.month_until_timestamp, until_timestamp);
            if let Some(tenant) = self.tenant_id_option.clone() {
                assert_eq!(tenant.as_str(), tenant_id);
            };
            if self.too_many_requests {
                return Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ApiError {
                        code: Code::ReachPaymentPlatformRateLimit as u32,
                    }),
                ));
            }
            // Ok(self.rewards_of_the_month)
            todo!()
        }

        async fn get_charges_of_the_year(
            &self,
            since_timestamp: i64,
            until_timestamp: i64,
            tenant_id: &str,
        ) -> Result<Vec<Charge>, ErrResp> {
            assert_eq!(self.year_since_timestamp, since_timestamp);
            assert_eq!(self.year_until_timestamp, until_timestamp);
            if let Some(tenant) = self.tenant_id_option.clone() {
                assert_eq!(tenant.as_str(), tenant_id);
            };
            if self.too_many_requests {
                return Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ApiError {
                        code: Code::ReachPaymentPlatformRateLimit as u32,
                    }),
                ));
            }
            // Ok(self.rewards_of_the_year)
            todo!()
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
                return Err(common::payment_platform::Error::ApiError(Box::new(
                    err_info,
                )));
            }
            return Ok(self.tenant.clone());
        }

        async fn create_tenant(
            &self,
            _create_tenant: &CreateTenant,
        ) -> Result<common::payment_platform::tenant::Tenant, common::payment_platform::Error>
        {
            panic!("must not reach this line")
        }

        async fn update_tenant(
            &self,
            _tenant_id: &str,
            _update_tenant: &UpdateTenant,
        ) -> Result<common::payment_platform::tenant::Tenant, common::payment_platform::Error>
        {
            panic!("must not reach this line")
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
                return Err(common::payment_platform::Error::ApiError(Box::new(
                    err_info,
                )));
            }
            Ok(self.tenant_transfers.clone())
        }
    }

    // #[tokio::test]
    // async fn handle_reward_req_returns_empty_rewards() {
    //     let account_id = 9853;
    //     let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
    //     let current_datetime = JAPANESE_TIME_ZONE.ymd(2021, 12, 31).and_hms(23, 59, 59);
    //     let reward_op = RewardOperationMock {
    //         tenant_id_option: None,
    //         too_many_requests: false,
    //         month_since_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 1)
    //             .and_hms(0, 0, 0)
    //             .timestamp(),
    //         month_until_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 31)
    //             .and_hms(23, 59, 59)
    //             .timestamp(),
    //         rewards_of_the_month: 0,
    //         year_since_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 1, 1)
    //             .and_hms(0, 0, 0)
    //             .timestamp(),
    //         year_until_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 31)
    //             .and_hms(23, 59, 59)
    //             .timestamp(),
    //         rewards_of_the_year: 0,
    //     };
    //     let tenant = create_dummy_tenant(tenant_id);
    //     let tenant_op = TenantOperationMock {
    //         tenant,
    //         too_many_requests: false,
    //     };
    //     let tenant_transfer_op = TenantTransferOperationMock {
    //         tenant_transfers: List {
    //             object: "list".to_string(),
    //             has_more: false,
    //             url: "/v1/tenant_transfers".to_string(),
    //             data: vec![],
    //             count: 0,
    //         },
    //         too_many_requests: false,
    //     };

    //     let result = handle_reward_req(
    //         account_id,
    //         reward_op,
    //         tenant_op,
    //         current_datetime,
    //         tenant_transfer_op,
    //     )
    //     .await
    //     .expect("failed to get Ok");

    //     assert_eq!(StatusCode::OK, result.0);
    //     assert_eq!(None, result.1 .0.bank_account);
    //     assert_eq!(None, result.1 .0.rewards_of_the_month);
    //     assert_eq!(None, result.1 .0.rewards_of_the_year);
    //     let empty = Vec::<Transfer>::with_capacity(0);
    //     assert_eq!(empty, result.1 .0.latest_two_transfers);
    // }

    // fn create_dummy_tenant(tenant_id: &str) -> common::payment_platform::tenant::Tenant {
    //     let reviewed_brands = vec![
    //         ReviewedBrands {
    //             brand: "Visa".to_string(),
    //             status: "passed".to_string(),
    //             available_date: Some(1626016999),
    //         },
    //         ReviewedBrands {
    //             brand: "MasterCard".to_string(),
    //             status: "passed".to_string(),
    //             available_date: Some(1626016999),
    //         },
    //         ReviewedBrands {
    //             brand: "JCB".to_string(),
    //             status: "passed".to_string(),
    //             available_date: Some(1626016999),
    //         },
    //         ReviewedBrands {
    //             brand: "AmericanExpress".to_string(),
    //             status: "passed".to_string(),
    //             available_date: Some(1626016999),
    //         },
    //         ReviewedBrands {
    //             brand: "DinersClub".to_string(),
    //             status: "passed".to_string(),
    //             available_date: Some(1626016999),
    //         },
    //     ];
    //     common::payment_platform::tenant::Tenant {
    //         id: tenant_id.to_string(),
    //         name: "タナカ　タロウ".to_string(),
    //         object: "tenant".to_string(),
    //         livemode: false,
    //         created: 1626016999,
    //         platform_fee_rate: "10.15".to_string(),
    //         payjp_fee_included: true,
    //         minimum_transfer_amount: 1000,
    //         bank_code: "0001".to_string(),
    //         bank_branch_code: "123".to_string(),
    //         bank_account_type: "普通".to_string(),
    //         bank_account_number: "1111222".to_string(),
    //         bank_account_holder_name: "タナカ　タロウ".to_string(),
    //         bank_account_status: "pending".to_string(),
    //         currencies_supported: vec!["jpy".to_string()],
    //         default_currency: "jpy".to_string(),
    //         reviewed_brands,
    //         metadata: None,
    //     }
    // }

    // #[tokio::test]
    // async fn handle_reward_req_fail_tenant_too_many_requests() {
    //     let account_id = 9853;
    //     let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
    //     let current_datetime = JAPANESE_TIME_ZONE.ymd(2021, 12, 31).and_hms(23, 59, 59);
    //     let reward_op = RewardOperationMock {
    //         tenant_id_option: Some(tenant_id.to_string()),
    //         too_many_requests: false,
    //         month_since_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 1)
    //             .and_hms(0, 0, 0)
    //             .timestamp(),
    //         month_until_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 31)
    //             .and_hms(23, 59, 59)
    //             .timestamp(),
    //         rewards_of_the_month: 0,
    //         year_since_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 1, 1)
    //             .and_hms(0, 0, 0)
    //             .timestamp(),
    //         year_until_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 31)
    //             .and_hms(23, 59, 59)
    //             .timestamp(),
    //         rewards_of_the_year: 0,
    //     };
    //     let tenant = create_dummy_tenant(tenant_id);
    //     let tenant_op = TenantOperationMock {
    //         tenant,
    //         too_many_requests: true,
    //     };
    //     let tenant_transfer_op = TenantTransferOperationMock {
    //         tenant_transfers: List {
    //             object: "list".to_string(),
    //             has_more: false,
    //             url: "/v1/tenant_transfers".to_string(),
    //             data: vec![],
    //             count: 0,
    //         },
    //         too_many_requests: false,
    //     };

    //     let result = handle_reward_req(
    //         account_id,
    //         reward_op,
    //         tenant_op,
    //         current_datetime,
    //         tenant_transfer_op,
    //     )
    //     .await
    //     .expect_err("failed to get Err");

    //     assert_eq!(StatusCode::TOO_MANY_REQUESTS, result.0);
    //     assert_eq!(
    //         err::Code::ReachPaymentPlatformRateLimit as u32,
    //         result.1 .0.code
    //     );
    // }

    // #[tokio::test]
    // async fn handle_reward_req_fail_rewards_too_many_requests() {
    //     let account_id = 9853;
    //     let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
    //     let current_datetime = JAPANESE_TIME_ZONE.ymd(2021, 12, 31).and_hms(23, 59, 59);
    //     let reward_op = RewardOperationMock {
    //         tenant_id_option: Some(tenant_id.to_string()),
    //         too_many_requests: true,
    //         month_since_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 1)
    //             .and_hms(0, 0, 0)
    //             .timestamp(),
    //         month_until_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 31)
    //             .and_hms(23, 59, 59)
    //             .timestamp(),
    //         rewards_of_the_month: 0,
    //         year_since_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 1, 1)
    //             .and_hms(0, 0, 0)
    //             .timestamp(),
    //         year_until_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 31)
    //             .and_hms(23, 59, 59)
    //             .timestamp(),
    //         rewards_of_the_year: 0,
    //     };
    //     let tenant = create_dummy_tenant(tenant_id);
    //     let tenant_op = TenantOperationMock {
    //         tenant,
    //         too_many_requests: false,
    //     };
    //     let tenant_transfer_op = TenantTransferOperationMock {
    //         tenant_transfers: List {
    //             object: "list".to_string(),
    //             has_more: false,
    //             url: "/v1/tenant_transfers".to_string(),
    //             data: vec![],
    //             count: 0,
    //         },
    //         too_many_requests: false,
    //     };

    //     let result = handle_reward_req(
    //         account_id,
    //         reward_op,
    //         tenant_op,
    //         current_datetime,
    //         tenant_transfer_op,
    //     )
    //     .await
    //     .expect_err("failed to get Err");

    //     assert_eq!(StatusCode::TOO_MANY_REQUESTS, result.0);
    //     assert_eq!(
    //         err::Code::ReachPaymentPlatformRateLimit as u32,
    //         result.1 .0.code
    //     );
    // }

    // #[tokio::test]
    // async fn handle_reward_req_fail_tenant_transfers_too_many_requests() {
    //     let account_id = 9853;
    //     let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
    //     let current_datetime = JAPANESE_TIME_ZONE.ymd(2021, 12, 31).and_hms(23, 59, 59);
    //     let reward_op = RewardOperationMock {
    //         tenant_id_option: Some(tenant_id.to_string()),
    //         too_many_requests: false,
    //         month_since_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 1)
    //             .and_hms(0, 0, 0)
    //             .timestamp(),
    //         month_until_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 31)
    //             .and_hms(23, 59, 59)
    //             .timestamp(),
    //         rewards_of_the_month: 0,
    //         year_since_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 1, 1)
    //             .and_hms(0, 0, 0)
    //             .timestamp(),
    //         year_until_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 31)
    //             .and_hms(23, 59, 59)
    //             .timestamp(),
    //         rewards_of_the_year: 0,
    //     };
    //     let tenant = create_dummy_tenant(tenant_id);
    //     let tenant_op = TenantOperationMock {
    //         tenant,
    //         too_many_requests: false,
    //     };
    //     let tenant_transfer_op = TenantTransferOperationMock {
    //         tenant_transfers: List {
    //             object: "list".to_string(),
    //             has_more: false,
    //             url: "/v1/tenant_transfers".to_string(),
    //             data: vec![],
    //             count: 0,
    //         },
    //         too_many_requests: true,
    //     };

    //     let result = handle_reward_req(
    //         account_id,
    //         reward_op,
    //         tenant_op,
    //         current_datetime,
    //         tenant_transfer_op,
    //     )
    //     .await
    //     .expect_err("failed to get Err");

    //     assert_eq!(StatusCode::TOO_MANY_REQUESTS, result.0);
    //     assert_eq!(
    //         err::Code::ReachPaymentPlatformRateLimit as u32,
    //         result.1 .0.code
    //     );
    // }

    // #[tokio::test]
    // async fn handle_reward_req_returns_reward_with_tenant_1tenant_transfer() {
    //     let account_id = 9853;
    //     let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
    //     let current_datetime = JAPANESE_TIME_ZONE.ymd(2021, 12, 31).and_hms(23, 59, 59);
    //     let reward_op = RewardOperationMock {
    //         tenant_id_option: Some(tenant_id.to_string()),
    //         too_many_requests: false,
    //         month_since_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 1)
    //             .and_hms(0, 0, 0)
    //             .timestamp(),
    //         month_until_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 31)
    //             .and_hms(23, 59, 59)
    //             .timestamp(),
    //         rewards_of_the_month: 2696,
    //         year_since_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 1, 1)
    //             .and_hms(0, 0, 0)
    //             .timestamp(),
    //         year_until_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 31)
    //             .and_hms(23, 59, 59)
    //             .timestamp(),
    //         rewards_of_the_year: 2696,
    //     };
    //     let tenant = create_dummy_tenant(tenant_id);
    //     let tenant_op = TenantOperationMock {
    //         tenant: tenant.clone(),
    //         too_many_requests: false,
    //     };
    //     let transfer_id = "ten_tr_920fdff2a571ace3441bd78b3";
    //     let tenant_transfer = create_dummy_tenant_transfer(transfer_id, tenant_id);
    //     let tenant_transfer_op = TenantTransferOperationMock {
    //         tenant_transfers: List {
    //             object: "list".to_string(),
    //             has_more: false,
    //             url: "/v1/tenant_transfers".to_string(),
    //             data: vec![tenant_transfer.clone()],
    //             count: 1,
    //         },
    //         too_many_requests: false,
    //     };

    //     let result = handle_reward_req(
    //         account_id,
    //         reward_op,
    //         tenant_op,
    //         current_datetime,
    //         tenant_transfer_op,
    //     )
    //     .await
    //     .expect("failed to get Ok");

    //     assert_eq!(StatusCode::OK, result.0);
    //     let bank_account = BankAccount {
    //         bank_code: tenant.bank_code.to_string(),
    //         branch_code: tenant.bank_branch_code.to_string(),
    //         account_type: tenant.bank_account_type.to_string(),
    //         account_number: tenant.bank_account_number.to_string(),
    //         account_holder_name: tenant.bank_account_holder_name.to_string(),
    //     };
    //     assert_eq!(Some(bank_account), result.1 .0.bank_account);
    //     assert_eq!(Some(2696), result.1 .0.rewards_of_the_month);
    //     assert_eq!(Some(2696), result.1 .0.rewards_of_the_year);
    //     // create_dummy_transfer1から導出される結果
    //     let transfer = Transfer {
    //         status: "pending".to_string(),
    //         amount: 2696,
    //         scheduled_date_in_jst: Ymd {
    //             year: 2022,
    //             month: 1,
    //             day: 31,
    //         },
    //         transfer_amount: None,
    //         transfer_date_in_jst: None,
    //         carried_balance: Some(0),
    //     };
    //     assert_eq!(vec![transfer], result.1 .0.latest_two_transfers);
    // }

    // fn create_dummy_tenant_transfer(transfer_id: &str, tenant_id: &str) -> TenantTransfer {
    //     let charge_id = "ch_7fb5aea258910da9a756985cbe51f";
    //     TenantTransfer {
    //         object: "tenant_transfer".to_string(),
    //         id: transfer_id.to_string(),
    //         livemode: false,
    //         created: 1641055119,
    //         amount: 2696,
    //         currency: "jpy".to_string(),
    //         status: "pending".to_string(),
    //         charges: List {
    //             object: "list".to_string(),
    //             has_more: false,
    //             url: format!("/v1/tenant_transfers/{}/charges", transfer_id),
    //             data: vec![create_dummy_charge(charge_id, tenant_id)],
    //             count: 1,
    //         },
    //         scheduled_date: "2022-01-31".to_string(),
    //         summary: Summary {
    //             charge_count: 1,
    //             charge_fee: 90,
    //             charge_gross: 4000,
    //             net: 3696,
    //             refund_amount: 1000,
    //             refund_count: 1,
    //             dispute_amount: 0,
    //             dispute_count: 0,
    //             total_platform_fee: 214,
    //         },
    //         term_start: 1638284400,
    //         term_end: 1640962800,
    //         transfer_amount: None,
    //         transfer_date: None,
    //         carried_balance: Some(0),
    //         tenant_id: tenant_id.to_string(),
    //     }
    // }

    // fn create_dummy_charge(charge_id: &str, tenant_id: &str) -> Charge {
    //     Charge {
    //         id: charge_id.to_string(),
    //         object: "charge".to_string(),
    //         livemode: false,
    //         created: 1639931415,
    //         amount: 4000,
    //         currency: "jpy".to_string(),
    //         paid: true,
    //         expired_at: None,
    //         captured: true,
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
    //         fee_rate: Some("3.00".to_string()),
    //         refunded: true,
    //         amount_refunded: 1000,
    //         refund_reason: Some("テスト".to_string()),
    //         subscription: None,
    //         metadata: None,
    //         platform_fee: None,
    //         tenant: Some(tenant_id.to_string()),
    //         platform_fee_rate: Some("10.15".to_string()),
    //         total_platform_fee: Some(214),
    //         three_d_secure_status: Some("verified".to_string()),
    //     }
    // }

    // #[tokio::test]
    // async fn handle_reward_req_returns_reward_with_tenant_2tenant_transfers() {
    //     let account_id = 9853;
    //     let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
    //     let reward_op = RewardOperationMock {
    //         tenant_id_option: Some(tenant_id.to_string()),
    //         too_many_requests: false,
    //         month_since_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 1)
    //             .and_hms(0, 0, 0)
    //             .timestamp(),
    //         month_until_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 31)
    //             .and_hms(23, 59, 59)
    //             .timestamp(),
    //         rewards_of_the_month: 2696,
    //         year_since_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 1, 1)
    //             .and_hms(0, 0, 0)
    //             .timestamp(),
    //         year_until_timestamp: JAPANESE_TIME_ZONE
    //             .ymd(2021, 12, 31)
    //             .and_hms(23, 59, 59)
    //             .timestamp(),
    //         rewards_of_the_year: 16000,
    //     };
    //     let tenant = create_dummy_tenant(tenant_id);
    //     let tenant_op = TenantOperationMock {
    //         tenant: tenant.clone(),
    //         too_many_requests: false,
    //     };
    //     let transfer_id1 = "ten_tr_920fdff2a571ace3441bd78b3";
    //     let tenant_transfer1 = create_dummy_tenant_transfer(transfer_id1, tenant_id);
    //     let transfer_id2 = "ten_tr_920fdff2a571ace3441bd78b4";
    //     let tenant_transfer2 = create_dummy_tenant_transfer(transfer_id2, tenant_id);
    //     let tenant_transfer_op = TenantTransferOperationMock {
    //         tenant_transfers: List {
    //             object: "list".to_string(),
    //             has_more: false,
    //             url: "/v1/tenant_transfers".to_string(),
    //             data: vec![tenant_transfer1.clone(), tenant_transfer2.clone()],
    //             count: 1,
    //         },
    //         too_many_requests: false,
    //     };
    //     let current_datetime = JAPANESE_TIME_ZONE.ymd(2021, 12, 31).and_hms(23, 59, 59);

    //     let result = handle_reward_req(
    //         account_id,
    //         reward_op,
    //         tenant_op,
    //         current_datetime,
    //         tenant_transfer_op,
    //     )
    //     .await
    //     .expect("failed to get Ok");

    //     assert_eq!(StatusCode::OK, result.0);
    //     let bank_account = BankAccount {
    //         bank_code: tenant.bank_code.to_string(),
    //         branch_code: tenant.bank_branch_code.to_string(),
    //         account_type: tenant.bank_account_type.to_string(),
    //         account_number: tenant.bank_account_number.to_string(),
    //         account_holder_name: tenant.bank_account_holder_name.to_string(),
    //     };
    //     assert_eq!(Some(bank_account), result.1 .0.bank_account);
    //     assert_eq!(Some(2696), result.1 .0.rewards_of_the_month);
    //     assert_eq!(Some(16000), result.1 .0.rewards_of_the_year);
    //     let transfer1 = Transfer {
    //         status: "pending".to_string(),
    //         amount: 2696,
    //         scheduled_date_in_jst: Ymd {
    //             year: 2022,
    //             month: 1,
    //             day: 31,
    //         },
    //         transfer_amount: None,
    //         transfer_date_in_jst: None,
    //         carried_balance: Some(0),
    //     };
    //     let transfer2 = Transfer {
    //         status: "pending".to_string(),
    //         amount: 2696,
    //         scheduled_date_in_jst: Ymd {
    //             year: 2022,
    //             month: 1,
    //             day: 31,
    //         },
    //         transfer_amount: None,
    //         transfer_date_in_jst: None,
    //         carried_balance: Some(0),
    //     };
    //     assert_eq!(vec![transfer1, transfer2], result.1 .0.latest_two_transfers);
    // }
}
