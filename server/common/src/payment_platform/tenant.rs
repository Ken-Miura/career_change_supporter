// Copyright 2021 Ken Miura

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    access_info::AccessInfo,
    err::{Error, ErrorInfo},
};

use axum::async_trait;

const TENANTS_OPERATION_PATH: &str = "/v1/tenants";

/// PAY.JP APIにおけるテナントを示す <https://pay.jp/docs/api/#tenant%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88>
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
    /// 一つのオブジェクトには最大20キーまで保存でき、キーは40文字まで、バリューは500文字までの文字列
    /// <https://pay.jp/docs/api/?java#metadata>
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
    async fn find_tenant_by_tenant_id(&self, tenant_id: &str) -> Result<Tenant, Error>;
}

pub struct TenantOperationImpl<'a> {
    access_info: &'a AccessInfo,
}

impl<'a> TenantOperationImpl<'a> {
    pub fn new(access_info: &'a AccessInfo) -> Self {
        Self { access_info }
    }
}

#[async_trait]
impl<'a> TenantOperation for TenantOperationImpl<'a> {
    async fn find_tenant_by_tenant_id(&self, tenant_id: &str) -> Result<Tenant, Error> {
        let operation_url = self.access_info.base_url() + TENANTS_OPERATION_PATH + "/" + tenant_id;
        let username = self.access_info.username();
        let password = self.access_info.password();
        let client = reqwest::Client::new();
        let resp = client
            .get(operation_url)
            .basic_auth(username, Some(password))
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
        let tenant = resp
            .json::<Tenant>()
            .await
            .map_err(|e| Error::RequestProcessingError(Box::new(e)))?;
        return Ok(tenant);
    }
}
