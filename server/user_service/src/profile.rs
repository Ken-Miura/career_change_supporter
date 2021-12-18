// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
use chrono::{DateTime, Datelike, Duration, FixedOffset, TimeZone, Utc};
use common::{
    model::user::{Account, CareerInfo, ConsultingFee, IdentityInfo, Tenant},
    payment_platform::{
        charge::{ChargeOperation, ChargeOperationImpl, Query as SearchChargesQuery},
        tenant::{TenantOperation, TenantOperationImpl},
        tenant_transfer::{
            Query as SearchTenantTransfersQuery, TenantTransferOperation,
            TenantTransferOperationImpl,
        },
    },
    schema::ccs_schema::{
        career_info::{dsl::career_info as career_info_table, user_account_id},
        consulting_fee::dsl::consulting_fee as consulting_fee_table,
        identity_info::dsl::identity_info as identity_info_table,
        tenant::dsl::tenant as tenant_table,
        user_account::dsl::user_account,
    },
    ApiError, DatabaseConnection, ErrResp, RespResult, MAX_NUM_OF_CAREER_INFO_PER_USER_ACCOUNT,
};
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    result::Error::NotFound,
    ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl,
};
use serde::Serialize;

use crate::{
    err_code::NO_ACCOUNT_FOUND,
    util::{session::User, unexpected_err_resp, ACCESS_INFO, JAPANESE_TIME_ZONE},
};

pub(crate) async fn get_profile(
    User { account_id }: User,
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<ProfileResult> {
    let profile_op = ProfileOperationImpl::new(conn);
    let tenant_op = TenantOperationImpl::new(&ACCESS_INFO);
    let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
    let tenant_transfer_op = TenantTransferOperationImpl::new(&ACCESS_INFO);
    let current_datetime = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
    get_profile_internal(
        account_id,
        profile_op,
        tenant_op,
        charge_op,
        current_datetime,
        tenant_transfer_op,
    )
    .await
}

async fn get_profile_internal(
    account_id: i32,
    profile_op: impl ProfileOperation,
    tenant_op: impl TenantOperation,
    charge_op: impl ChargeOperation,
    current_time: DateTime<FixedOffset>,
    tenant_transfer_op: impl TenantTransferOperation,
) -> RespResult<ProfileResult> {
    let account = profile_op.find_user_account_by_user_account_id(account_id)?;
    let identity_info_option = profile_op.find_identity_info_by_user_account_id(account_id)?;
    if identity_info_option.is_none() {
        return Ok((
            StatusCode::OK,
            Json(ProfileResult::email_address(account.email_address).finish()),
        ));
    };
    let identity = identity_info_option.map(convert_identity_info_to_identity);
    let careers_info = profile_op.filter_career_info_by_user_account_id(account_id)?;
    let careers = careers_info
        .into_iter()
        .map(convert_career_info_to_career)
        .collect::<Vec<Career>>();
    let consulting_fee_option = profile_op.find_consulting_fee_by_user_account_id(account_id)?;
    let fee_per_hour_in_yen = consulting_fee_option.map(|c| c.fee_per_hour_in_yen);
    let tenant_option = profile_op.find_tenant_by_user_account_id(account_id)?;
    let payment_platform_results = if let Some(tenant) = tenant_option {
        let bank_account = get_bank_account_by_tenant_id(tenant_op, &tenant.tenant_id).await?;
        let profit =
            get_profit_of_current_month(charge_op, &tenant.tenant_id, current_time).await?;
        let (most_recent_transfer, last_time_transfer) =
            get_latest_two_tenant_transfers(tenant_transfer_op, &tenant.tenant_id).await?;
        (
            Some(bank_account),
            profit,
            last_time_transfer,
            most_recent_transfer,
        )
    } else {
        (None, None, None, None)
    };
    Ok((
        StatusCode::OK,
        Json(
            ProfileResult::email_address(account.email_address)
                .identity(identity)
                .careers(careers)
                .fee_per_hour_in_yen(fee_per_hour_in_yen)
                .bank_account(payment_platform_results.0)
                .profit(payment_platform_results.1)
                .last_time_transfer(payment_platform_results.2)
                .most_recent_transfer(payment_platform_results.3)
                .finish(),
        ),
    ))
}

#[derive(Serialize, Debug)]
pub(crate) struct ProfileResult {
    email_address: String,
    identity: Option<Identity>,
    careers: Vec<Career>,
    fee_per_hour_in_yen: Option<i32>,
    bank_account: Option<BankAccount>,
    profit: Option<u32>, // プラットフォーム利用の取り分は引く。振込手数料は引かない。
    last_time_transfer: Option<Transfer>,
    most_recent_transfer: Option<Transfer>,
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
    pub year: i32,
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

#[derive(Serialize, Debug)]
pub(crate) struct Transfer {
    pub status: String,
    pub amount: i32,
    pub scheduled_date_in_jst: Ymd,
}

impl ProfileResult {
    fn email_address(email_address: String) -> ProfileResultBuilder {
        ProfileResultBuilder {
            email_address,
            identity: None,
            careers: vec![],
            fee_per_hour_in_yen: None,
            bank_account: None,
            profit: None,
            last_time_transfer: None,
            most_recent_transfer: None,
        }
    }
}

struct ProfileResultBuilder {
    email_address: String,
    identity: Option<Identity>,
    careers: Vec<Career>,
    fee_per_hour_in_yen: Option<i32>,
    bank_account: Option<BankAccount>,
    profit: Option<u32>,
    last_time_transfer: Option<Transfer>,
    most_recent_transfer: Option<Transfer>,
}

impl ProfileResultBuilder {
    fn identity(mut self, identity: Option<Identity>) -> ProfileResultBuilder {
        self.identity = identity;
        self
    }

    fn careers(mut self, careers: Vec<Career>) -> ProfileResultBuilder {
        self.careers = careers;
        self
    }

    fn fee_per_hour_in_yen(mut self, fee_per_hour_in_yen: Option<i32>) -> ProfileResultBuilder {
        self.fee_per_hour_in_yen = fee_per_hour_in_yen;
        self
    }

    fn bank_account(mut self, bank_account: Option<BankAccount>) -> ProfileResultBuilder {
        self.bank_account = bank_account;
        self
    }

    fn profit(mut self, profit: Option<u32>) -> ProfileResultBuilder {
        self.profit = profit;
        self
    }

    fn last_time_transfer(mut self, last_time_transfer: Option<Transfer>) -> ProfileResultBuilder {
        self.last_time_transfer = last_time_transfer;
        self
    }

    fn most_recent_transfer(
        mut self,
        most_recent_transfer: Option<Transfer>,
    ) -> ProfileResultBuilder {
        self.most_recent_transfer = most_recent_transfer;
        self
    }

    fn finish(self) -> ProfileResult {
        ProfileResult {
            email_address: self.email_address,
            identity: self.identity,
            careers: self.careers,
            fee_per_hour_in_yen: self.fee_per_hour_in_yen,
            bank_account: self.bank_account,
            profit: self.profit,
            last_time_transfer: self.last_time_transfer,
            most_recent_transfer: self.most_recent_transfer,
        }
    }
}

fn convert_identity_info_to_identity(identity_info: IdentityInfo) -> Identity {
    let date = identity_info.date_of_birth;
    let ymd = Ymd {
        year: date.year(),
        month: date.month(),
        day: date.day(),
    };
    Identity {
        last_name: identity_info.last_name,
        first_name: identity_info.first_name,
        last_name_furigana: identity_info.last_name_furigana,
        first_name_furigana: identity_info.first_name_furigana,
        sex: identity_info.sex,
        date_of_birth: ymd,
        prefecture: identity_info.prefecture,
        city: identity_info.city,
        address_line1: identity_info.address_line1,
        address_line2: identity_info.address_line2,
    }
}

fn convert_career_info_to_career(career_info: CareerInfo) -> Career {
    let career_start_date = Ymd {
        year: career_info.career_start_date.year(),
        month: career_info.career_start_date.month(),
        day: career_info.career_start_date.day(),
    };
    let career_end_date = career_info.career_end_date.map(|end_date| Ymd {
        year: end_date.year(),
        month: end_date.month(),
        day: end_date.day(),
    });
    Career {
        company_name: career_info.company_name,
        department_name: career_info.department_name,
        office: career_info.office,
        career_start_date,
        career_end_date,
        contract_type: career_info.contract_type,
        profession: career_info.profession,
        annual_income_in_man_yen: career_info.annual_income_in_man_yen,
        is_manager: career_info.is_manager,
        position_name: career_info.position_name,
        is_new_graduate: career_info.is_new_graduate,
        note: career_info.note,
    }
}

async fn get_bank_account_by_tenant_id(
    tenant_op: impl TenantOperation,
    tenant_id: &str,
) -> Result<BankAccount, ErrResp> {
    let tenant = tenant_op
        .find_tenant_by_tenant_id(tenant_id)
        .await
        .map_err(|e| match e {
            common::payment_platform::err::Error::RequestProcessingError(err) => {
                tracing::error!("failed to process request: {}", err);
                unexpected_err_resp()
            }
            common::payment_platform::err::Error::ApiError(err) => {
                tracing::error!("failed to request tenant operation: {}", err);
                // TODO: このためのエラーコードを用意するか検討
                unexpected_err_resp()
            }
        })?;
    Ok(BankAccount {
        bank_code: tenant.bank_code,
        branch_code: tenant.bank_branch_code,
        account_type: tenant.bank_account_type,
        account_number: tenant.bank_account_number,
        account_holder_name: tenant.bank_account_holder_name,
    })
}

async fn get_profit_of_current_month(
    charge_op: impl ChargeOperation,
    tenant_id: &str,
    current_time: DateTime<FixedOffset>,
) -> Result<Option<u32>, ErrResp> {
    let current_year = current_time.year();
    let current_month = current_time.month();
    let since_timestamp = chrono::Utc
        .ymd(current_year, current_month, 1)
        .and_hms(0, 0, 0)
        .timestamp();
    let next_month = current_month + 1; // 12月のときを考える必要あり？
    let until_timestamp = (chrono::Utc
        .ymd(current_year, next_month, 1)
        .and_hms(23, 59, 59)
        - Duration::days(1))
    .timestamp();
    // let current_year = 2021;
    // let current_month = 12;
    // let since = chrono::Utc
    //     .ymd(current_year, current_month, 1)
    //     .and_hms(0, 0, 0);
    // println!("{}", since);
    // let mut n_y = current_year;
    // let next_month = if current_month == 12 {
    //     n_y = current_year + 1;
    //     1
    // } else {
    //     current_month + 1 // 12月のときを考える必要あり
    // };
    // let until = chrono::Utc
    //     .ymd(n_y, next_month, 1)
    //     .and_hms(23, 59, 59)
    //     - Duration::days(1);
    // println!("{}", until);
    let search_charges_query = SearchChargesQuery::build()
        .since(since_timestamp)
        .until(until_timestamp)
        .tenant(tenant_id)
        .finish()
        .map_err(|e| {
            tracing::error!("failed to build search charges query: {}", e);
            unexpected_err_resp()
        })?;
    let b = charge_op.search_charges(&search_charges_query).await;
    todo!()
}

async fn get_latest_two_tenant_transfers(
    tenant_transfer_op: impl TenantTransferOperation,
    tenant_id: &str,
) -> Result<(Option<Transfer>, Option<Transfer>), ErrResp> {
    // TODO: 2を定数化
    let search_tenant_transfers_query = SearchTenantTransfersQuery::build()
        .limit(2)
        .tenant(tenant_id)
        .finish()
        .expect("failed to get Ok");
    let c = tenant_transfer_op
        .search_tenant_transfers(&search_tenant_transfers_query)
        .await;
    todo!()
}

trait ProfileOperation {
    fn find_user_account_by_user_account_id(&self, id: i32) -> Result<Account, ErrResp>;
    fn find_identity_info_by_user_account_id(
        &self,
        id: i32,
    ) -> Result<Option<IdentityInfo>, ErrResp>;
    fn filter_career_info_by_user_account_id(&self, id: i32) -> Result<Vec<CareerInfo>, ErrResp>;
    fn find_tenant_by_user_account_id(&self, id: i32) -> Result<Option<Tenant>, ErrResp>;
    fn find_consulting_fee_by_user_account_id(
        &self,
        id: i32,
    ) -> Result<Option<ConsultingFee>, ErrResp>;
}

struct ProfileOperationImpl {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl ProfileOperationImpl {
    fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { conn }
    }
}

impl ProfileOperation for ProfileOperationImpl {
    fn find_user_account_by_user_account_id(&self, id: i32) -> Result<Account, ErrResp> {
        let result = user_account.find(id).first::<Account>(&self.conn);
        match result {
            Ok(account) => Ok(account),
            Err(e) => {
                if e == NotFound {
                    Err((
                        StatusCode::BAD_REQUEST,
                        Json(ApiError {
                            code: NO_ACCOUNT_FOUND,
                        }),
                    ))
                } else {
                    Err(unexpected_err_resp())
                }
            }
        }
    }

    fn find_identity_info_by_user_account_id(
        &self,
        id: i32,
    ) -> Result<Option<IdentityInfo>, ErrResp> {
        let result = identity_info_table
            .find(id)
            .first::<IdentityInfo>(&self.conn);
        match result {
            Ok(identity_info) => Ok(Some(identity_info)),
            Err(e) => {
                if e == NotFound {
                    Ok(None)
                } else {
                    Err(unexpected_err_resp())
                }
            }
        }
    }

    fn filter_career_info_by_user_account_id(&self, id: i32) -> Result<Vec<CareerInfo>, ErrResp> {
        let result = career_info_table
            .filter(user_account_id.eq(id))
            .limit(MAX_NUM_OF_CAREER_INFO_PER_USER_ACCOUNT)
            .load::<CareerInfo>(&self.conn)
            .map_err(|e| {
                tracing::error!("failed to filter career info by id {}: {}", id, e);
                unexpected_err_resp()
            })?;
        Ok(result)
    }

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

    fn find_consulting_fee_by_user_account_id(
        &self,
        id: i32,
    ) -> Result<Option<ConsultingFee>, ErrResp> {
        let result = consulting_fee_table
            .find(id)
            .first::<ConsultingFee>(&self.conn);
        match result {
            Ok(consulting_fee) => Ok(Some(consulting_fee)),
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
