// Copyright 2021 Ken Miura

use serde::{Deserialize, Serialize};
use std::{error::Error as StdError, fmt::Display};

use super::{
    access_info::AccessInfo,
    charge::Charge,
    err::{Error, ErrorInfo},
    list::List,
};

use axum::async_trait;

const TENANT_TRANSFER_OPERATION_PATH: &str = "/v1/tenant_transfers";

/// PAY.JP APIにおけるTenant Transfer (入金) を示す<br>
/// 参考<br>
/// transfer: <https://pay.jp/docs/api/?shell#transfer%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88><br>
/// tenant transfer: <https://pay.jp/docs/api/?shell#tenant_transfer%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88><br>
#[derive(Serialize, Deserialize, Debug)]
pub struct TenantTransfer {
    pub object: String,
    pub id: String,
    pub livemode: bool,
    pub created: i64,
    pub amount: u32,
    pub currency: String,
    pub status: String,
    pub charges: List<Charge>,
    pub scheduled_date: String, // String?
    pub summary: Summary,
    pub term_start: i64,
    pub term_end: i64,
    pub transfer_amount: Option<u32>,
    pub transfer_date: Option<String>, // String?
    pub carried_balance: u32,
    pub tenant_id: Option<String>,
}

// TODO: 負の数があり得るのか確認する
#[derive(Serialize, Deserialize, Debug)]
pub struct Summary {
    pub charge_count: u32,
    pub charge_fee: u32,
    pub charge_gross: u32,
    pub net: u32,
    pub refund_amount: u32,
    pub refund_count: u32,
    pub dispute_amount: u32,
    pub dispute_count: u32,
    pub platform_charge_fee: Option<u32>,
    pub total_platform_fee: u32,
}

/// 入金 <https://pay.jp/docs/api/?shell#tenant-transfer-%E5%85%A5%E9%87%91> に関連する操作を提供する
#[async_trait]
pub trait TenantTransferOperation {
    async fn search_tenant_transfers(&self, query: &Query) -> Result<List<TenantTransfer>, Error>;
}

/// 入金リストを取得 <https://pay.jp/docs/api/?shell#%E3%83%86%E3%83%8A%E3%83%B3%E3%83%88%E3%81%AE%E5%85%A5%E9%87%91%E3%83%AA%E3%82%B9%E3%83%88%E3%82%92%E5%8F%96%E5%BE%97> の際に渡すクエリ<br>
/// 複数値がセットされた場合、AND検索となる。値が空の場合、（limit=10の制限の中で）すべての値を取得する
#[derive(Serialize, Debug)]
pub struct Query {
    limit: Option<u32>,
    offset: Option<u32>,
    since: Option<i64>,
    until: Option<i64>,
    since_scheduled_date: Option<i64>,
    until_scheduled_date: Option<i64>,
    status: Option<String>,
    transfer: Option<String>,
    tenant: Option<String>,
}

impl Query {
    /// クエリを生成するための[QueryBuilder]を生成する
    pub fn build() -> QueryBuilder {
        QueryBuilder::new()
    }

    // NOTE: 可能な限り提供されるPAY.JPのAPIに沿った形にしたいため、引数が多いが許容する
    #[allow(clippy::too_many_arguments)]
    fn new(
        limit: Option<u32>,
        offset: Option<u32>,
        since: Option<i64>,
        until: Option<i64>,
        since_scheduled_date: Option<i64>,
        until_scheduled_date: Option<i64>,
        status: Option<String>,
        transfer: Option<String>,
        tenant: Option<String>,
    ) -> Result<Self, InvalidParamError> {
        if let Some(l) = limit {
            if !(1..=100).contains(&l) {
                return Err(InvalidParamError::Limit(l));
            };
        };
        if let Some(s) = since {
            if let Some(u) = until {
                if s > u {
                    return Err(InvalidParamError::SinceExceedsUntil { since: s, until: u });
                };
            };
        };
        if let Some(s) = since_scheduled_date {
            if let Some(u) = until_scheduled_date {
                if s > u {
                    return Err(
                        InvalidParamError::SinceScheduledDateExceedsUntilScheduledDate {
                            since_scheduled_date: s,
                            until_scheduled_date: u,
                        },
                    );
                };
            };
        };
        Ok(Query {
            limit,
            offset,
            since,
            until,
            since_scheduled_date,
            until_scheduled_date,
            status,
            transfer,
            tenant,
        })
    }

    pub fn limit(&self) -> Option<u32> {
        self.limit
    }

    pub fn offset(&self) -> Option<u32> {
        self.offset
    }

    pub fn since(&self) -> Option<i64> {
        self.since
    }

    pub fn until(&self) -> Option<i64> {
        self.until
    }

    pub fn since_scheduled_date(&self) -> Option<i64> {
        self.since_scheduled_date
    }

    pub fn until_scheduled_date(&self) -> Option<i64> {
        self.until_scheduled_date
    }

    pub fn status(&self) -> Option<String> {
        self.status.clone()
    }

    pub fn transfer(&self) -> Option<String> {
        self.transfer.clone()
    }

    pub fn tenant(&self) -> Option<String> {
        self.tenant.clone()
    }
}

/// [Query] 生成時に返却される可能性のあるエラー
#[derive(Debug)]
pub enum InvalidParamError {
    Limit(u32),
    SinceExceedsUntil {
        since: i64,
        until: i64,
    },
    SinceScheduledDateExceedsUntilScheduledDate {
        since_scheduled_date: i64,
        until_scheduled_date: i64,
    },
}

impl Display for InvalidParamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidParamError::Limit(limit) => {
                write!(f, "limit must be 1 or more, or 100 or less: {}", limit)
            }
            InvalidParamError::SinceExceedsUntil { since, until } => write!(
                f,
                "since timestamp exeeds until timestamp (since: {}, until: {})",
                since, until
            ),
            InvalidParamError::SinceScheduledDateExceedsUntilScheduledDate { since_scheduled_date, until_scheduled_date } => write!(
                f,
                "since_scheduled_date timestamp exeeds until_scheduled_date timestamp (since_scheduled_date: {}, until_scheduled_date: {})",
                since_scheduled_date, until_scheduled_date
            ),
        }
    }
}

impl StdError for InvalidParamError {}

/// [Query]を生成するためのヘルパー
pub struct QueryBuilder {
    limit: Option<u32>,
    offset: Option<u32>,
    since: Option<i64>,
    until: Option<i64>,
    since_scheduled_date: Option<i64>,
    until_scheduled_date: Option<i64>,
    status: Option<String>,
    transfer: Option<String>,
    tenant: Option<String>,
}

impl QueryBuilder {
    fn new() -> Self {
        Self {
            limit: None,
            offset: None,
            since: None,
            until: None,
            since_scheduled_date: None,
            until_scheduled_date: None,
            status: None,
            transfer: None,
            tenant: None,
        }
    }

    /// [Query]に設定するlimitをセットする
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// [Query]に設定するoffsetをセットする
    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    /// [Query]に設定するsinceをセットする
    pub fn since(mut self, since: i64) -> Self {
        self.since = Some(since);
        self
    }

    /// [Query]に設定するuntilをセットする
    pub fn until(mut self, until: i64) -> Self {
        self.until = Some(until);
        self
    }

    /// [Query]に設定するsince_scheduled_dateをセットする
    pub fn since_scheduled_date(mut self, since_scheduled_date: i64) -> Self {
        self.since_scheduled_date = Some(since_scheduled_date);
        self
    }

    /// [Query]に設定するuntil_scheduled_dateをセットする
    pub fn until_scheduled_date(mut self, until_scheduled_date: i64) -> Self {
        self.until_scheduled_date = Some(until_scheduled_date);
        self
    }

    /// [Query]に設定するstatusをセットする
    pub fn status(mut self, status: &str) -> Self {
        self.status = Some(status.to_string());
        self
    }

    /// [Query]に設定するtransferをセットする
    pub fn transfer(mut self, transfer: &str) -> Self {
        self.transfer = Some(transfer.to_string());
        self
    }

    /// [Query]に設定するtenantをセットする
    pub fn tenant(mut self, tenant: &str) -> Self {
        self.tenant = Some(tenant.to_string());
        self
    }

    /// [Query]を生成する
    /// # Errors
    /// * `InvalidParamError::Limit` - [QueryBuilder]にセットしたリミットが0以下、もしくは101以上の場合
    /// * `InvalidParamError::SinceExceedsUntil` - [QueryBuilder]にセットしたsinceがuntilより大きい場合
    /// * `InvalidParamError::SinceScheduledDateExceedsUntilScheduledDate` - [QueryBuilder]にセットしたsince_scheduled_dateがuntil_scheduled_dateより大きい場合
    pub fn finish(self) -> Result<Query, InvalidParamError> {
        Query::new(
            self.limit,
            self.offset,
            self.since,
            self.until,
            self.since_scheduled_date,
            self.until_scheduled_date,
            self.status,
            self.transfer,
            self.tenant,
        )
    }
}

pub struct TenantTransferOperationImpl<'a> {
    access_info: &'a AccessInfo,
}

impl<'a> TenantTransferOperationImpl<'a> {
    pub fn new(access_info: &'a AccessInfo) -> Self {
        Self { access_info }
    }
}

#[async_trait]
impl<'a> TenantTransferOperation for TenantTransferOperationImpl<'a> {
    async fn search_tenant_transfers(&self, query: &Query) -> Result<List<TenantTransfer>, Error> {
        tracing::info!("search_tenant_transfers: query = {:?}", query);
        let operation_url = self.access_info.base_url() + TENANT_TRANSFER_OPERATION_PATH;
        let username = self.access_info.username();
        let password = self.access_info.password();
        let client = reqwest::Client::new();
        let resp = client
            .get(operation_url)
            .basic_auth(username, Some(password))
            .query(query)
            .send()
            .await
            .map_err(|e| Error::RequestProcessingError(Box::new(e)))?;
        let status_code = resp.status();
        if status_code.is_client_error() || status_code.is_server_error() {
            let err = resp
                .json::<ErrorInfo>()
                .await
                .map_err(|e| Error::RequestProcessingError(Box::new(e)))?;
            return Err(Error::ApiError(err));
        };
        let tenant_transfer_list = resp
            .json::<List<TenantTransfer>>()
            .await
            .map_err(|e| Error::RequestProcessingError(Box::new(e)))?;
        return Ok(tenant_transfer_list);
    }
}

#[cfg(test)]
mod tests {}
