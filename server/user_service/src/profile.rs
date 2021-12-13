// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
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
        career_info::{dsl::career_info, user_account_id},
        identity_info::dsl::identity_info,
        tenant::dsl::tenant,
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
    util::{session::User, unexpected_err_resp, ACCESS_INFO},
};

pub(crate) async fn get_profile(
    User { account_id }: User,
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<ProfileResult> {
    let profile_op = ProfileOperationImpl::new(conn);
    let tenant_op = TenantOperationImpl::new(&ACCESS_INFO);
    let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
    let tenant_transfer_op = TenantTransferOperationImpl::new(&ACCESS_INFO);
    get_profile_internal(
        account_id,
        profile_op,
        tenant_op,
        charge_op,
        tenant_transfer_op,
    )
    .await
}

async fn get_profile_internal(
    account_id: i32,
    profile_op: impl ProfileOperation,
    tenant_op: impl TenantOperation,
    charge_op: impl ChargeOperation,
    tenant_transfer_op: impl TenantTransferOperation,
) -> RespResult<ProfileResult> {
    let account = profile_op.find_user_account_by_user_account_id(account_id)?;
    let identity_info_option = profile_op.find_identity_info_by_user_account_id(account_id)?;
    let careers = profile_op.filter_career_info_by_user_account_id(account_id)?;
    let tenant_option = profile_op.find_tenant_by_user_account_id(account_id)?;

    let profile_result = ProfileResult {
        email_address: "test@test.com".to_string(),
        identity: None,
        careers: None,
        fee_per_hour_in_yen: None,
        bank_account: None,
        profit: None,
        last_time_transfer: None,
        most_recent_transfer: None,
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

#[derive(Serialize, Debug)]
pub(crate) struct Transfer {
    pub status: String,
    pub amount: i32,
    pub scheduled_date_in_jst: Option<Ymd>,
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
            Ok(new_pwd) => Ok(new_pwd),
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
        let result = identity_info.find(id).first::<IdentityInfo>(&self.conn);
        match result {
            Ok(id_info) => Ok(Some(id_info)),
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
        let result = career_info
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
        let result = tenant.find(id).first::<Tenant>(&self.conn);
        match result {
            Ok(t) => Ok(Some(t)),
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
        todo!()
    }
}
