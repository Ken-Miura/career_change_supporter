// Copyright 2021 Ken Miura

use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error as StdError, fmt::Display};

use super::{
    access_info::AccessInfo,
    err::{Error, ErrorInfo},
    list::List,
};

use axum::async_trait;

const CHARGES_OPERATION_PATH: &str = "/v1/charges";

/// PAY.JP APIにおけるCharge (支払い) を示す <https://pay.jp/docs/api/#charge%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88>
#[derive(Serialize, Deserialize, Debug)]
pub struct Charge {
    pub id: String,
    pub object: String,
    pub livemode: bool,
    pub created: u64,
    pub amount: u32,
    pub currency: String,
    pub paid: bool,
    pub expired_at: Option<u64>,
    pub captured: bool,
    pub captured_at: u64,
    pub card: Card,
    pub customer: Option<String>,
    pub description: Option<String>,
    pub failure_code: Option<String>,
    pub failure_message: Option<String>,
    pub fee_rate: String,
    pub refunded: bool,
    pub amount_refunded: u32,
    pub refund_reason: Option<String>,
    pub subscription: Option<String>,
    /// 一つのオブジェクトには最大20キーまで保存でき、キーは40文字まで、バリューは500文字までの文字列
    /// <https://pay.jp/docs/api/?java#metadata>
    pub metadata: Option<HashMap<String, String>>,
    pub platform_fee: Option<u32>,
    pub tenant: Option<String>,
    pub platform_fee_rate: Option<String>,
    pub total_platform_fee: Option<u32>,
}

/// [Charge] 内で利用される型
/// 支払いに利用されたクレジットカードを示す
/// <https://pay.jp/docs/api/#card%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88>
#[derive(Serialize, Deserialize, Debug)]
pub struct Card {
    object: String,
    id: String,
    created: u64,
    name: Option<String>,
    last4: String,
    exp_month: u32,
    exp_year: u32,
    brand: String,
    cvc_check: String,
    fingerprint: String,
    address_state: Option<String>,
    address_city: Option<String>,
    address_line1: Option<String>,
    address_line2: Option<String>,
    country: Option<String>,
    address_zip: Option<String>,
    address_zip_check: String,
    /// 一つのオブジェクトには最大20キーまで保存でき、キーは40文字まで、バリューは500文字までの文字列
    /// <https://pay.jp/docs/api/?java#metadata>
    pub metadata: Option<HashMap<String, String>>,
}

#[async_trait]
pub trait ChargeOperation {
    async fn search_charges(&self, query: &Query) -> Result<List<Charge>, Error>;
}

#[derive(Serialize, Debug)]
pub struct Query {
    limit: Option<u32>,
    offset: Option<u32>,
    since: Option<u64>,
    until: Option<u64>,
    customer: Option<String>,
    subscription: Option<String>,
    tenant: Option<String>,
}

impl Query {
    pub fn build() -> QueryBuilder {
        QueryBuilder::new()
    }

    fn new(
        limit: Option<u32>,
        offset: Option<u32>,
        since: Option<u64>,
        until: Option<u64>,
        customer: Option<String>,
        subscription: Option<String>,
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
        Ok(Query {
            limit,
            offset,
            since,
            until,
            customer,
            subscription,
            tenant,
        })
    }

    pub fn limit(&self) -> Option<u32> {
        self.limit
    }

    pub fn offset(&self) -> Option<u32> {
        self.offset
    }

    pub fn since(&self) -> Option<u64> {
        self.since
    }

    pub fn until(&self) -> Option<u64> {
        self.until
    }

    pub fn customer(&self) -> Option<String> {
        self.customer.clone()
    }

    pub fn subscription(&self) -> Option<String> {
        self.subscription.clone()
    }

    pub fn tenant(&self) -> Option<String> {
        self.tenant.clone()
    }
}

/// [Query] 生成時に返却される可能性のあるエラー
#[derive(Debug)]
pub enum InvalidParamError {
    Limit(u32),
    SinceExceedsUntil { since: u64, until: u64 },
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
        }
    }
}

impl StdError for InvalidParamError {}

pub struct QueryBuilder {
    limit: Option<u32>,
    offset: Option<u32>,
    since: Option<u64>,
    until: Option<u64>,
    customer: Option<String>,
    subscription: Option<String>,
    tenant: Option<String>,
}

impl QueryBuilder {
    fn new() -> Self {
        Self {
            limit: None,
            offset: None,
            since: None,
            until: None,
            customer: None,
            subscription: None,
            tenant: None,
        }
    }

    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn offset(mut self, offset: u32) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn since(mut self, since: u64) -> Self {
        self.since = Some(since);
        self
    }

    pub fn until(mut self, until: u64) -> Self {
        self.until = Some(until);
        self
    }

    pub fn customer(mut self, customer: &str) -> Self {
        self.customer = Some(customer.to_string());
        self
    }

    pub fn subscription(mut self, subscription: &str) -> Self {
        self.subscription = Some(subscription.to_string());
        self
    }

    pub fn tenant(mut self, tenant: &str) -> Self {
        self.tenant = Some(tenant.to_string());
        self
    }

    pub fn finish(self) -> Result<Query, InvalidParamError> {
        Query::new(
            self.limit,
            self.offset,
            self.since,
            self.until,
            self.customer,
            self.subscription,
            self.tenant,
        )
    }
}

pub struct ChargeOperationImpl<'a> {
    access_info: &'a AccessInfo,
}

impl<'a> ChargeOperationImpl<'a> {
    pub fn new(access_info: &'a AccessInfo) -> Self {
        Self { access_info }
    }
}

#[async_trait]
impl<'a> ChargeOperation for ChargeOperationImpl<'a> {
    async fn search_charges(&self, query: &Query) -> Result<List<Charge>, Error> {
        tracing::info!("search_charges: query = {:?}", query);
        let operation_url = self.access_info.base_url() + CHARGES_OPERATION_PATH;
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
        let charge_list = resp
            .json::<List<Charge>>()
            .await
            .map_err(|e| Error::RequestProcessingError(Box::new(e)))?;
        return Ok(charge_list);
    }
}
