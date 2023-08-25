// Copyright 2021 Ken Miura

use serde::{Deserialize, Serialize};

use super::{
    with_querystring, AccessInfo, Metadata, {Error, ErrorInfo},
};

use axum::async_trait;

const TENANTS_OPERATION_PATH: &str = "/v1/tenants";

/// [tenantオブジェクト](https://pay.jp/docs/api/#tenant%E3%82%AA%E3%83%96%E3%82%B8%E3%82%A7%E3%82%AF%E3%83%88)を示す構造体
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Tenant {
    pub id: String,
    pub name: String,
    pub object: String,
    pub livemode: bool,
    pub created: i64,
    pub platform_fee_rate: String,
    pub payjp_fee_included: bool,
    pub minimum_transfer_amount: i32,
    pub bank_code: String,
    pub bank_branch_code: String,
    pub bank_account_type: String,
    pub bank_account_number: String,
    pub bank_account_holder_name: String,
    pub bank_account_status: String,
    pub currencies_supported: Vec<String>,
    pub default_currency: String,
    pub reviewed_brands: Vec<ReviewedBrands>,
    pub metadata: Option<Metadata>,
}

/// 申請情報を提出済のブランドを示す構造体
///
/// [Tenant] 内で利用される
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ReviewedBrands {
    pub brand: String,
    pub status: String,
    pub available_date: Option<i64>,
}

/// [テナントを作成](https://pay.jp/docs/api/?shell#%E3%83%86%E3%83%8A%E3%83%B3%E3%83%88%E3%82%92%E4%BD%9C%E6%88%90)の引数を示す構造体
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateTenant {
    pub name: String,
    pub id: String,
    pub platform_fee_rate: String,
    pub payjp_fee_included: bool,
    pub minimum_transfer_amount: i32,
    pub bank_code: String,
    pub bank_branch_code: String,
    pub bank_account_type: String,
    pub bank_account_number: String,
    pub bank_account_holder_name: String,
    pub metadata: Option<Metadata>,
}

/// [テナント情報を更新](https://pay.jp/docs/api/?shell#%E3%83%86%E3%83%8A%E3%83%B3%E3%83%88%E6%83%85%E5%A0%B1%E3%82%92%E6%9B%B4%E6%96%B0)の引数を示す構造体
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateTenant {
    pub name: String,
    pub platform_fee_rate: String,
    pub minimum_transfer_amount: i32,
    pub bank_code: String,
    pub bank_branch_code: String,
    pub bank_account_type: String,
    pub bank_account_number: String,
    pub bank_account_holder_name: String,
    pub metadata: Option<Metadata>,
}

/// [テナントを削除](https://pay.jp/docs/api/#%E3%83%86%E3%83%8A%E3%83%B3%E3%83%88%E3%82%92%E5%89%8A%E9%99%A4)の結果を示す構造体
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeleteTenantResult {
    pub deleted: bool,
    pub id: String,
}

#[async_trait]
pub trait TenantOperation {
    /// [テナントの情報を取得](https://pay.jp/docs/api/?shell#%E3%83%86%E3%83%8A%E3%83%B3%E3%83%88%E6%83%85%E5%A0%B1%E3%82%92%E5%8F%96%E5%BE%97)
    async fn get_tenant_by_tenant_id(&self, tenant_id: &str) -> Result<Tenant, Error>;
    /// [テナントを作成](https://pay.jp/docs/api/?shell#%E3%83%86%E3%83%8A%E3%83%B3%E3%83%88%E3%82%92%E4%BD%9C%E6%88%90)
    async fn create_tenant(&self, create_tenant: &CreateTenant) -> Result<Tenant, Error>;
    /// [テナント情報を更新](https://pay.jp/docs/api/?shell#%E3%83%86%E3%83%8A%E3%83%B3%E3%83%88%E6%83%85%E5%A0%B1%E3%82%92%E6%9B%B4%E6%96%B0)
    async fn update_tenant(
        &self,
        tenant_id: &str,
        update_tenant: &UpdateTenant,
    ) -> Result<Tenant, Error>;
    /// [テナントを削除](https://pay.jp/docs/api/#%E3%83%86%E3%83%8A%E3%83%B3%E3%83%88%E3%82%92%E5%89%8A%E9%99%A4)
    async fn delete_tenant(&self, tenant_id: &str) -> Result<DeleteTenantResult, Error>;
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
    async fn get_tenant_by_tenant_id(&self, tenant_id: &str) -> Result<Tenant, Error> {
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
            return Err(Error::ApiError(Box::new(err)));
        };
        let tenant = resp
            .json::<Tenant>()
            .await
            .map_err(|e| Error::RequestProcessingError(Box::new(e)))?;
        return Ok(tenant);
    }

    async fn create_tenant(&self, create_tenant: &CreateTenant) -> Result<Tenant, Error> {
        let operation_url = self.access_info.base_url() + TENANTS_OPERATION_PATH;
        let username = self.access_info.username();
        let password = self.access_info.password();
        let client = reqwest::Client::new();
        let client = with_querystring(client.post(operation_url), create_tenant)?;
        let resp = client
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
            return Err(Error::ApiError(Box::new(err)));
        };
        let tenant = resp
            .json::<Tenant>()
            .await
            .map_err(|e| Error::RequestProcessingError(Box::new(e)))?;
        return Ok(tenant);
    }

    async fn update_tenant(
        &self,
        tenant_id: &str,
        update_tenant: &UpdateTenant,
    ) -> Result<Tenant, Error> {
        let operation_url = self.access_info.base_url() + TENANTS_OPERATION_PATH + "/" + tenant_id;
        let username = self.access_info.username();
        let password = self.access_info.password();
        let client = reqwest::Client::new();
        let client = with_querystring(client.post(operation_url), update_tenant)?;
        let resp = client
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
            return Err(Error::ApiError(Box::new(err)));
        };
        let tenant = resp
            .json::<Tenant>()
            .await
            .map_err(|e| Error::RequestProcessingError(Box::new(e)))?;
        return Ok(tenant);
    }

    async fn delete_tenant(&self, tenant_id: &str) -> Result<DeleteTenantResult, Error> {
        let operation_url = self.access_info.base_url() + TENANTS_OPERATION_PATH + "/" + tenant_id;
        let username = self.access_info.username();
        let password = self.access_info.password();
        let client = reqwest::Client::new();
        let resp = client
            .delete(operation_url)
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
            return Err(Error::ApiError(Box::new(err)));
        };
        let delete_tenant_result = resp
            .json::<DeleteTenantResult>()
            .await
            .map_err(|e| Error::RequestProcessingError(Box::new(e)))?;
        return Ok(delete_tenant_result);
    }
}
