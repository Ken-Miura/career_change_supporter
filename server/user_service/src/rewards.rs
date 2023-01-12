// Copyright 2021 Ken Miura

use axum::async_trait;
use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, Datelike, FixedOffset, NaiveDate, Utc};
use common::util::Ymd;
use common::JAPANESE_TIME_ZONE;
use common::{
    payment_platform::{
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

use crate::util::rewards::{
    calculate_rewards, create_start_and_end_date_time_of_current_month,
    create_start_and_end_date_time_of_current_year, PaymentInfo,
};
use crate::util::{self};
use crate::{
    err::{self, unexpected_err_resp},
    util::{bank_account::BankAccount, session::User, ACCESS_INFO},
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

        let (current_month_since, current_month_until) =
            create_start_and_end_date_time_of_current_month(&current_time)?;
        let payments_of_the_month = reward_op
            .filter_receipts_of_the_month_by_consultant_id(
                account_id,
                &current_month_since,
                &current_month_until,
            )
            .await?;
        let rewards_of_the_month = calculate_rewards(&payments_of_the_month)?;

        let (current_year_since, current_year_until) =
            create_start_and_end_date_time_of_current_year(&current_time)?;
        let payments_of_the_year = reward_op
            .filter_receipts_of_the_year_by_consultant_id(
                account_id,
                &current_year_since,
                &current_year_until,
            )
            .await?;
        let rewards_of_the_year = calculate_rewards(&payments_of_the_year)?;

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

    /// （startとendを含む）startからendまでの期間のPaymentInfoを取得する
    async fn filter_receipts_of_the_year_by_consultant_id(
        &self,
        consultant_id: i64,
        start: &DateTime<FixedOffset>,
        end: &DateTime<FixedOffset>,
    ) -> Result<Vec<PaymentInfo>, ErrResp>;

    /// （startとendを含む）startからendまでの期間のPaymentInfoを取得する
    async fn filter_receipts_of_the_month_by_consultant_id(
        &self,
        consultant_id: i64,
        start: &DateTime<FixedOffset>,
        end: &DateTime<FixedOffset>,
    ) -> Result<Vec<PaymentInfo>, ErrResp>;
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

    async fn filter_receipts_of_the_year_by_consultant_id(
        &self,
        consultant_id: i64,
        start: &DateTime<FixedOffset>,
        end: &DateTime<FixedOffset>,
    ) -> Result<Vec<PaymentInfo>, ErrResp> {
        util::rewards::filter_receipts_of_the_duration_by_consultant_id(
            &self.pool,
            consultant_id,
            start,
            end,
        )
        .await
    }

    async fn filter_receipts_of_the_month_by_consultant_id(
        &self,
        consultant_id: i64,
        start: &DateTime<FixedOffset>,
        end: &DateTime<FixedOffset>,
    ) -> Result<Vec<PaymentInfo>, ErrResp> {
        util::rewards::filter_receipts_of_the_duration_by_consultant_id(
            &self.pool,
            consultant_id,
            start,
            end,
        )
        .await
    }
}

// TODO: 事前準備に用意するデータに関して、データの追加、編集でvalidatorを実装した後、それを使ってチェックを行うよう修正する
#[cfg(test)]
mod tests {

    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::{DateTime, FixedOffset, TimeZone};
    use common::payment_platform::charge::Charge;
    use common::payment_platform::customer::Card;
    use common::payment_platform::tenant::ReviewedBrands;
    use common::payment_platform::tenant_transfer::Summary;
    use common::util::Ymd;
    use common::JAPANESE_TIME_ZONE;
    use common::{
        payment_platform::{
            tenant::{CreateTenant, TenantOperation, UpdateTenant},
            tenant_transfer::{
                Query as SearchTenantTransfersQuery, TenantTransfer, TenantTransferOperation,
            },
            ErrorDetail, ErrorInfo, List,
        },
        ErrResp,
    };

    use crate::err::Code;
    use crate::rewards::{handle_reward_req, Transfer};
    use crate::util::bank_account::BankAccount;
    use crate::util::rewards::{
        create_start_and_end_date_time_of_current_month,
        create_start_and_end_date_time_of_current_year, PaymentInfo,
    };

    use super::RewardOperation;

    struct RewardOperationMock {
        account_id: i64,
        tenant_id_option: Option<String>,
        current_date_time: DateTime<FixedOffset>,
        payments_of_the_month: Vec<PaymentInfo>,
        payments_of_the_year: Vec<PaymentInfo>,
    }

    #[async_trait]
    impl RewardOperation for RewardOperationMock {
        async fn find_tenant_id_by_account_id(
            &self,
            account_id: i64,
        ) -> Result<Option<String>, ErrResp> {
            assert_eq!(self.account_id, account_id);
            Ok(self.tenant_id_option.clone())
        }

        async fn filter_receipts_of_the_month_by_consultant_id(
            &self,
            consultant_id: i64,
            start: &DateTime<FixedOffset>,
            end: &DateTime<FixedOffset>,
        ) -> Result<Vec<PaymentInfo>, ErrResp> {
            assert_eq!(self.account_id, consultant_id);
            let (s, e) = create_start_and_end_date_time_of_current_month(&self.current_date_time)?;
            assert_eq!(*start, s);
            assert_eq!(*end, e);
            Ok(self.payments_of_the_month.clone())
        }

        async fn filter_receipts_of_the_year_by_consultant_id(
            &self,
            consultant_id: i64,
            start: &DateTime<FixedOffset>,
            end: &DateTime<FixedOffset>,
        ) -> Result<Vec<PaymentInfo>, ErrResp> {
            assert_eq!(self.account_id, consultant_id);
            let (s, e) = create_start_and_end_date_time_of_current_year(&self.current_date_time)?;
            assert_eq!(*start, s);
            assert_eq!(*end, e);
            Ok(self.payments_of_the_year.clone())
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

    #[tokio::test]
    async fn handle_reward_req_returns_empty_rewards() {
        let account_id = 9853;
        let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
        let current_date_time = JAPANESE_TIME_ZONE.ymd(2021, 12, 31).and_hms(23, 59, 59);
        let reward_op = RewardOperationMock {
            account_id,
            tenant_id_option: None,
            current_date_time,
            payments_of_the_month: vec![],
            payments_of_the_year: vec![],
        };
        let tenant = create_dummy_tenant(tenant_id);
        let tenant_op = TenantOperationMock {
            tenant,
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

        let result = handle_reward_req(
            account_id,
            reward_op,
            tenant_op,
            current_date_time,
            tenant_transfer_op,
        )
        .await
        .expect("failed to get Ok");

        assert_eq!(StatusCode::OK, result.0);
        assert_eq!(None, result.1 .0.bank_account);
        assert_eq!(None, result.1 .0.rewards_of_the_month);
        assert_eq!(None, result.1 .0.rewards_of_the_year);
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
    async fn handle_reward_req_fail_tenant_too_many_requests() {
        let account_id = 9853;
        let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
        let current_date_time = JAPANESE_TIME_ZONE.ymd(2021, 12, 31).and_hms(23, 59, 59);
        let reward_op = RewardOperationMock {
            account_id,
            tenant_id_option: Some(tenant_id.to_string()),
            current_date_time,
            payments_of_the_month: vec![],
            payments_of_the_year: vec![],
        };
        let tenant = create_dummy_tenant(tenant_id);
        let tenant_op = TenantOperationMock {
            tenant,
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

        let result = handle_reward_req(
            account_id,
            reward_op,
            tenant_op,
            current_date_time,
            tenant_transfer_op,
        )
        .await
        .expect_err("failed to get Err");

        assert_eq!(StatusCode::TOO_MANY_REQUESTS, result.0);
        assert_eq!(Code::ReachPaymentPlatformRateLimit as u32, result.1 .0.code);
    }

    #[tokio::test]
    async fn handle_reward_req_fail_tenant_transfers_too_many_requests() {
        let account_id = 9853;
        let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
        let current_date_time = JAPANESE_TIME_ZONE.ymd(2021, 12, 31).and_hms(23, 59, 59);
        let reward_op = RewardOperationMock {
            account_id,
            tenant_id_option: Some(tenant_id.to_string()),
            current_date_time,
            payments_of_the_month: vec![],
            payments_of_the_year: vec![],
        };
        let tenant = create_dummy_tenant(tenant_id);
        let tenant_op = TenantOperationMock {
            tenant,
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

        let result = handle_reward_req(
            account_id,
            reward_op,
            tenant_op,
            current_date_time,
            tenant_transfer_op,
        )
        .await
        .expect_err("failed to get Err");

        assert_eq!(StatusCode::TOO_MANY_REQUESTS, result.0);
        assert_eq!(Code::ReachPaymentPlatformRateLimit as u32, result.1 .0.code);
    }

    #[tokio::test]
    async fn handle_reward_req_returns_reward_with_tenant_1tenant_transfer() {
        let account_id = 9853;
        let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
        let current_date_time = JAPANESE_TIME_ZONE.ymd(2021, 12, 31).and_hms(23, 59, 59);
        let reward_op = RewardOperationMock {
            account_id,
            tenant_id_option: Some(tenant_id.to_string()),
            current_date_time,
            payments_of_the_month: vec![PaymentInfo {
                fee_per_hour_in_yen: 5003,
                platform_fee_rate_in_percentage: "30.0".to_string(),
            }],
            payments_of_the_year: vec![PaymentInfo {
                fee_per_hour_in_yen: 5003,
                platform_fee_rate_in_percentage: "30.0".to_string(),
            }],
        };
        let tenant = create_dummy_tenant(tenant_id);
        let tenant_op = TenantOperationMock {
            tenant: tenant.clone(),
            too_many_requests: false,
        };
        let transfer_id = "ten_tr_920fdff2a571ace3441bd78b3";
        let tenant_transfer = create_dummy_tenant_transfer(transfer_id, tenant_id);
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

        let result = handle_reward_req(
            account_id,
            reward_op,
            tenant_op,
            current_date_time,
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
        assert_eq!(Some(3503), result.1 .0.rewards_of_the_month);
        assert_eq!(Some(3503), result.1 .0.rewards_of_the_year);
        // create_dummy_transfer1から導出される結果
        let transfer = Transfer {
            status: "pending".to_string(),
            amount: 3503,
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

    fn create_dummy_tenant_transfer(transfer_id: &str, tenant_id: &str) -> TenantTransfer {
        let charge_id = "ch_7fb5aea258910da9a756985cbe51f";
        TenantTransfer {
            object: "tenant_transfer".to_string(),
            id: transfer_id.to_string(),
            livemode: false,
            created: 1641055119,
            amount: 3503,
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
                charge_fee: 150,
                charge_gross: 5003,
                net: 3503,
                refund_amount: 0,
                refund_count: 0,
                dispute_amount: 0,
                dispute_count: 0,
                total_platform_fee: 1350,
            },
            term_start: 1638284400,
            term_end: 1640962800,
            transfer_amount: None,
            transfer_date: None,
            carried_balance: Some(0),
            tenant_id: tenant_id.to_string(),
        }
    }

    fn create_dummy_charge(charge_id: &str, tenant_id: &str) -> Charge {
        Charge {
            id: charge_id.to_string(),
            object: "charge".to_string(),
            livemode: false,
            created: 1639931415,
            amount: 5003,
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
            refunded: false,
            amount_refunded: 0,
            refund_reason: Some("テスト".to_string()),
            subscription: None,
            metadata: None,
            platform_fee: None,
            tenant: Some(tenant_id.to_string()),
            platform_fee_rate: Some("30.0".to_string()),
            total_platform_fee: Some(1350),
            three_d_secure_status: Some("verified".to_string()),
        }
    }

    #[tokio::test]
    async fn handle_reward_req_returns_reward_with_tenant_2tenant_transfers() {
        let account_id = 9853;
        let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
        let current_date_time = JAPANESE_TIME_ZONE.ymd(2021, 12, 31).and_hms(23, 59, 59);
        let reward_op = RewardOperationMock {
            account_id,
            tenant_id_option: Some(tenant_id.to_string()),
            current_date_time,
            payments_of_the_month: vec![PaymentInfo {
                fee_per_hour_in_yen: 5003,
                platform_fee_rate_in_percentage: "30.0".to_string(),
            }],
            payments_of_the_year: vec![
                PaymentInfo {
                    fee_per_hour_in_yen: 5003,
                    platform_fee_rate_in_percentage: "30.0".to_string(),
                },
                PaymentInfo {
                    fee_per_hour_in_yen: 4008,
                    platform_fee_rate_in_percentage: "30.0".to_string(),
                },
            ],
        };
        let tenant = create_dummy_tenant(tenant_id);
        let tenant_op = TenantOperationMock {
            tenant: tenant.clone(),
            too_many_requests: false,
        };
        let transfer_id1 = "ten_tr_920fdff2a571ace3441bd78b3";
        let tenant_transfer1 = create_dummy_tenant_transfer(transfer_id1, tenant_id);
        let transfer_id2 = "ten_tr_920fdff2a571ace3441bd78b4";
        let tenant_transfer2 = create_dummy_tenant_transfer(transfer_id2, tenant_id);
        let tenant_transfer_op = TenantTransferOperationMock {
            tenant_transfers: List {
                object: "list".to_string(),
                has_more: false,
                url: "/v1/tenant_transfers".to_string(),
                data: vec![tenant_transfer1.clone(), tenant_transfer2.clone()],
                count: 1,
            },
            too_many_requests: false,
        };

        let result = handle_reward_req(
            account_id,
            reward_op,
            tenant_op,
            current_date_time,
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
        assert_eq!(Some(3503), result.1 .0.rewards_of_the_month);
        assert_eq!(Some(3503 + 2806), result.1 .0.rewards_of_the_year);
        let transfer1 = Transfer {
            status: "pending".to_string(),
            amount: 3503,
            scheduled_date_in_jst: Ymd {
                year: 2022,
                month: 1,
                day: 31,
            },
            transfer_amount: None,
            transfer_date_in_jst: None,
            carried_balance: Some(0),
        };
        let transfer2 = Transfer {
            status: "pending".to_string(),
            amount: 3503,
            scheduled_date_in_jst: Ymd {
                year: 2022,
                month: 1,
                day: 31,
            },
            transfer_amount: None,
            transfer_date_in_jst: None,
            carried_balance: Some(0),
        };
        assert_eq!(vec![transfer1, transfer2], result.1 .0.latest_two_transfers);
    }
}
