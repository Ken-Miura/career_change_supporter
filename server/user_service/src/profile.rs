// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
use common::{
    payment_platform::tenant::{TenantOperation, TenantOperationImpl},
    DatabaseConnection, RespResult,
};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection,
};
use serde::Serialize;

use crate::util::{session::User, ACCESS_INFO};

pub(crate) async fn get_profile(
    User { account_id }: User,
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<ProfileResult> {
    // TODO: profileの実装
    let tenant_op = TenantOperationImpl::new(&ACCESS_INFO);
    let result = tenant_op
        .find_tenant_by_tenant_id("c8f0aa44901940849cbdb8b3e7d9f305")
        .await;
    match result {
        Ok(tenant) => {
            tracing::info!("{}", tenant.bank_account_holder_name);
        }
        Err(err) => tracing::info!("err: {}", err),
    };
    let op = ProfileOperationImpl::new(conn);
    get_profile_internal(account_id, op).await
}

async fn get_profile_internal(
    account_id: i32,
    op: impl ProfileOperation,
) -> RespResult<ProfileResult> {
    tracing::info!("id: {}", account_id);
    let profile_result = ProfileResult {
        email_address: "test@test.com".to_string(),
        identity: None,
        careers: None,
        fee_per_hour_in_yen: None,
        bank_account: None,
        profit: None,
    };
    Ok((StatusCode::OK, Json(profile_result)))
}

#[derive(Serialize, Debug)]
pub(crate) struct ProfileResult {
    email_address: String,
    identity: Option<Identity>,
    careers: Option<Vec<Career>>,
    fee_per_hour_in_yen: Option<i32>,
    bank_account: Option<BankAccount>,
    profit: Option<u32>, // プラットフォーム利用の取り分は引く。振込手数料は引かない。
}

#[derive(Serialize, Debug)]
pub(crate) struct Identity {
    pub last_name: String,
    pub first_name: String,
    pub last_name_furigana: String,
    pub first_name_furigana: String,
    pub sex: String,
    pub date_of_birth: Ymd,
    pub prefecture: String,
    pub city: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
}

#[derive(Serialize, Debug)]
pub(crate) struct Ymd {
    pub year: u32,
    pub month: u32,
    pub day: u32,
}

#[derive(Serialize, Debug)]
pub(crate) struct Career {
    pub company_name: String,
    pub department_name: Option<String>,
    pub office: Option<String>,
    pub career_start_date: Ymd,
    pub career_end_date: Option<Ymd>,
    pub contract_type: String,
    pub profession: Option<String>,
    pub annual_income_in_man_yen: Option<i32>,
    pub is_manager: bool,
    pub position_name: Option<String>,
    pub is_new_graduate: bool,
    pub note: Option<String>,
}

#[derive(Serialize, Debug)]
pub(crate) struct BankAccount {
    pub bank_code: String, // 明確な仕様は見つからなかったが数字4桁が最も普及しているように見える
    pub branch_code: String,
    pub account_type: String,
    pub account_number: String,
    pub account_holder_name: String,
}

trait ProfileOperation {
    // DBの分離レベルにはREAD COMITTEDを想定。
    // その想定の上でトランザクションが必要かどうかを検討し、操作を分離して実装
}

struct ProfileOperationImpl {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl ProfileOperationImpl {
    fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { conn }
    }
}

impl ProfileOperation for ProfileOperationImpl {}
