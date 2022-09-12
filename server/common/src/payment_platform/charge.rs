// Copyright 2021 Ken Miura

use serde::{Deserialize, Serialize};
use std::{error::Error as StdError, fmt::Display};

use super::{
    customer::Card,
    AccessInfo, List, Metadata, {Error, ErrorInfo},
};

use axum::async_trait;

const CHARGES_OPERATION_PATH: &str = "/v1/charges";

/// [chargeオブジェクト](https://pay.jp/docs/api/#charge%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88)を示す構造体
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Charge {
    pub id: String,
    pub object: String,
    pub livemode: bool,
    pub created: i64,
    pub amount: i32,
    pub currency: String,
    pub paid: bool,
    pub expired_at: Option<i64>,
    pub captured: bool,
    pub captured_at: Option<i64>,
    pub card: Option<Card>,
    pub customer: Option<String>,
    pub description: Option<String>,
    pub failure_code: Option<String>,
    pub failure_message: Option<String>,
    pub fee_rate: Option<String>,
    pub refunded: bool,
    pub amount_refunded: i32,
    pub refund_reason: Option<String>,
    pub subscription: Option<String>,
    pub metadata: Option<Metadata>,
    pub platform_fee: Option<u32>,
    pub tenant: Option<String>,
    pub platform_fee_rate: Option<String>,
    pub total_platform_fee: Option<i32>,
}

#[async_trait]
pub trait ChargeOperation {
    // NOTE: 単体テストのために&selfでなく、&mut selfとしている。単体テストでの利用時に&selfを利用可能な解決策が見つかった場合、&selfに変更
    /// [支払いリストを取得](https://pay.jp/docs/api/?shell#%E6%94%AF%E6%89%95%E3%81%84%E3%83%AA%E3%82%B9%E3%83%88%E3%82%92%E5%8F%96%E5%BE%97)
    async fn search_charges(&mut self, query: &Query) -> Result<List<Charge>, Error>;

    /// [支払いを作成](https://pay.jp/docs/api/#%E6%94%AF%E6%89%95%E3%81%84%E3%82%92%E4%BD%9C%E6%88%90)
    async fn create_charge(&self, create_charge: &CreateCharge) -> Result<Charge, Error>;
}

/// [支払いリストを取得](https://pay.jp/docs/api/?shell#%E6%94%AF%E6%89%95%E3%81%84%E3%83%AA%E3%82%B9%E3%83%88%E3%82%92%E5%8F%96%E5%BE%97)の際に渡すクエリ
///
/// 複数値がセットされた場合、AND検索となる。値が空の場合、（limit=10の制限の中で）すべての値を取得しようと試みる
#[derive(Serialize, Debug)]
pub struct Query {
    limit: Option<u32>,
    offset: Option<u32>,
    since: Option<i64>,
    until: Option<i64>,
    customer: Option<String>,
    subscription: Option<String>,
    tenant: Option<String>,
}

impl Query {
    /// クエリを生成するための[QueryBuilder]を生成する
    pub fn build() -> QueryBuilder {
        QueryBuilder::new()
    }

    fn new(
        limit: Option<u32>,
        offset: Option<u32>,
        since: Option<i64>,
        until: Option<i64>,
        customer: Option<String>,
        subscription: Option<String>,
        tenant: Option<String>,
    ) -> Result<Self, InvalidQueryParamError> {
        if let Some(l) = limit {
            if !(1..=100).contains(&l) {
                return Err(InvalidQueryParamError::Limit(l));
            };
        };
        if let Some(s) = since {
            if let Some(u) = until {
                if s > u {
                    return Err(InvalidQueryParamError::SinceExceedsUntil { since: s, until: u });
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

    pub fn since(&self) -> Option<i64> {
        self.since
    }

    pub fn until(&self) -> Option<i64> {
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
pub enum InvalidQueryParamError {
    Limit(u32),
    SinceExceedsUntil { since: i64, until: i64 },
}

impl Display for InvalidQueryParamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InvalidQueryParamError::Limit(limit) => {
                write!(f, "limit must be 1 or more, or 100 or less: {}", limit)
            }
            InvalidQueryParamError::SinceExceedsUntil { since, until } => write!(
                f,
                "since timestamp exeeds until timestamp (since: {}, until: {})",
                since, until
            ),
        }
    }
}

impl StdError for InvalidQueryParamError {}

/// [Query]を生成するためのヘルパー
pub struct QueryBuilder {
    limit: Option<u32>,
    offset: Option<u32>,
    since: Option<i64>,
    until: Option<i64>,
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

    /// [Query]に設定するcustomerをセットする
    pub fn customer(mut self, customer: &str) -> Self {
        self.customer = Some(customer.to_string());
        self
    }

    /// [Query]に設定するsubscriptionをセットする
    pub fn subscription(mut self, subscription: &str) -> Self {
        self.subscription = Some(subscription.to_string());
        self
    }

    /// [Query]に設定するtenantをセットする
    pub fn tenant(mut self, tenant: &str) -> Self {
        self.tenant = Some(tenant.to_string());
        self
    }

    /// [Query]を生成する
    /// # Errors
    /// * `InvalidQueryParamError::Limit` - [QueryBuilder]にセットしたリミットが0以下、もしくは101以上の場合
    /// * `InvalidQueryParamError::SinceExceedsUntil` - [QueryBuilder]にセットしたsinceがuntilより大きい場合
    pub fn finish(self) -> Result<Query, InvalidQueryParamError> {
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

/// [支払いを作成](https://pay.jp/docs/api/#%E6%94%AF%E6%89%95%E3%81%84%E3%82%92%E4%BD%9C%E6%88%90)の際に渡す構造体
#[derive(Serialize, Debug)]
pub struct CreateCharge {
    amount: Option<i32>,
    currency: Option<String>,
    product: Option<String>,
    customer: Option<String>,
    card: Option<String>,
    description: Option<String>,
    capture: Option<bool>,
    expiry_days: Option<u32>,
    metadata: Option<Metadata>,
    platform_fee: Option<u32>,
    tenant: Option<String>,
    three_d_secure: Option<bool>,
}

impl CreateCharge {
    fn new(
        amount: Option<i32>,
        currency: Option<String>,
        product: Option<String>,
        customer: Option<String>,
        card: Option<String>,
        description: Option<String>,
        capture: Option<bool>,
        expiry_days: Option<u32>,
        metadata: Option<Metadata>,
        platform_fee: Option<u32>,
        tenant: Option<String>,
        three_d_secure: Option<bool>,
    ) -> Result<Self, InvalidCreateChargeParamError> {
        todo!()
    }
}

/// [CreateCharge] 生成時に返却される可能性のあるエラー
#[derive(Debug)]
pub enum InvalidCreateChargeParamError {}

impl Display for InvalidCreateChargeParamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl StdError for InvalidCreateChargeParamError {}

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
    async fn search_charges(&mut self, query: &Query) -> Result<List<Charge>, Error> {
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

    async fn create_charge(&self, create_charge: &CreateCharge) -> Result<Charge, Error> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use crate::payment_platform::charge::InvalidQueryParamError;

    use super::Query;

    #[test]
    fn empty_query_allowed() {
        let result = Query::build().finish();
        let query = result.expect("failed to get Ok");
        assert_eq!(None, query.limit());
        assert_eq!(None, query.offset());
        assert_eq!(None, query.since());
        assert_eq!(None, query.until());
        assert_eq!(None, query.customer());
        assert_eq!(None, query.subscription());
        assert_eq!(None, query.tenant());
    }

    #[test]
    fn query_has_value_that_is_passed_on_query_builder() {
        let since = chrono::Utc.ymd(2021, 12, 9).and_hms(23, 00, 40).timestamp();
        let until = chrono::Utc.ymd(2021, 12, 9).and_hms(23, 00, 41).timestamp();
        let customer = "cus_4df4b5ed720933f4fb9e28857517";
        let subscription = "sub_567a1e44562932ec1a7682d746e0";
        let tenant = "ten_121673955bd7aa144de5a8f6c262";
        let result = Query::build()
            .limit(100)
            .offset(0)
            .since(since)
            .until(until)
            .customer(customer)
            .subscription(subscription)
            .tenant(tenant)
            .finish();
        let query = result.expect("failed to get Ok");
        assert_eq!(Some(100), query.limit());
        assert_eq!(Some(0), query.offset());
        assert_eq!(Some(since), query.since());
        assert_eq!(Some(until), query.until());
        assert_eq!(Some(customer.to_string()), query.customer());
        assert_eq!(Some(subscription.to_string()), query.subscription());
        assert_eq!(Some(tenant.to_string()), query.tenant());
    }

    #[test]
    fn query_accepts_limit_value_that_is_between_1_and_100() {
        let result = Query::build().limit(1).finish();
        result.expect("failed to get Ok");
        let result = Query::build().limit(100).finish();
        result.expect("failed to get Ok");
    }

    #[test]
    fn query_rejects_limit_value_that_is_0_or_less_and_101_or_more() {
        let result = Query::build().limit(0).finish();
        let err = result.expect_err("failed to get Err");
        match err {
            InvalidQueryParamError::Limit(l) => {
                assert_eq!(0, l);
            }
            InvalidQueryParamError::SinceExceedsUntil { since, until } => {
                panic!("SinceExceedsUntil{{ since: {}, until: {} }}", since, until)
            }
        }
        let result = Query::build().limit(101).finish();
        let err = result.expect_err("failed to get Err");
        match err {
            InvalidQueryParamError::Limit(l) => {
                assert_eq!(101, l);
            }
            InvalidQueryParamError::SinceExceedsUntil { since, until } => {
                panic!("SinceExceedsUntil{{ since: {}, until: {} }}", since, until)
            }
        }
    }

    #[test]
    fn query_accepts_same_since_and_until() {
        let since = chrono::Utc.ymd(2021, 12, 9).and_hms(23, 00, 40).timestamp();
        let until = since;
        let result = Query::build().since(since).until(until).finish();
        result.expect("failed to get Ok");
    }

    #[test]
    fn query_fail_to_create_query_when_since_exceeds_until() {
        let since_timestamp = chrono::Utc.ymd(2021, 12, 9).and_hms(23, 00, 40).timestamp();
        let until_timestamp = chrono::Utc.ymd(2021, 12, 9).and_hms(23, 00, 39).timestamp();
        let result = Query::build()
            .since(since_timestamp)
            .until(until_timestamp)
            .finish();
        let err = result.expect_err("failed to get Ok");
        match err {
            InvalidQueryParamError::Limit(l) => panic!("Limit: {}", l),
            InvalidQueryParamError::SinceExceedsUntil { since, until } => {
                assert_eq!(since, since_timestamp);
                assert_eq!(until, until_timestamp);
            }
        }
    }
}
