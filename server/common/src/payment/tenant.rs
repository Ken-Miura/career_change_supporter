// Copyright 2021 Ken Miura

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::err::PaymentError;

use axum::async_trait;

/// PAY.JP APIにおけるテナントを示す (https://pay.jp/docs/api/#tenant%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88)
#[derive(Serialize, Deserialize, Debug)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub object: String,
    pub livemode: bool,
    pub created: u64,
    pub platform_fee_rate: String,
    pub payjp_fee_included: bool,
    pub minimum_transfer_amount: u32,
    pub bank_code: String,
    pub bank_branch_code: String,
    pub bank_account_type: String,
    pub bank_account_number: String,
    pub bank_account_holder_name: String,
    pub bank_account_status: String,
    pub currencies_supported: Vec<String>,
    pub default_currency: String,
    pub reviewed_brands: Vec<ReviewedBrands>,
    pub metadata: Option<HashMap<String, String>>,
}

/// [Tenant] 内で利用される型
/// 申請情報を提出済のブランド
#[derive(Serialize, Deserialize, Debug)]
pub struct ReviewedBrands {
    pub brand: String,
    pub status: String,
    pub available_date: Option<u64>,
}

#[async_trait]
pub trait TenantOperation {
    async fn find_tenant_by_tenant_id(&self, tenant_id: &str) -> Result<Tenant, PaymentError>;
}

pub struct TenantOperationImpl {
    endpoint_url: String,
    username: String,
    password: String,
}

impl TenantOperationImpl {
    fn new(endpoint_url: String, username: String, password: String) -> Self {
        Self {
            endpoint_url,
            username,
            password,
        }
    }
}

// PaymentErrorは、enumにしてrequest_processing_errとresponse_errに分ける？
// request_processing_err = reqwest err
// response_err (payment api err?) = payment err
#[async_trait]
impl TenantOperation for TenantOperationImpl {
    async fn find_tenant_by_tenant_id(&self, tenant_id: &str) -> Result<Tenant, PaymentError> {
        let client = reqwest::Client::new();
        let resp = client
            .get(self.endpoint_url.clone())
            .basic_auth(self.username.clone(), Some(self.password.clone()))
            .send()
            .await;
        let resp = resp.expect("msg");
        if resp.status().is_success() {
            let tenant = resp.json::<Tenant>().await;
            let tenant = tenant.expect("msg");
            return Ok(tenant);
        } else {
            let err = resp.json::<PaymentError>().await;
            let err = err.expect("msg");
            return Err(err);
        }
    }
}
