// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
use chrono::Datelike;
use common::{
    model::user::{Account, CareerInfo, ConsultingFee, IdentityInfo},
    schema::ccs_schema::{
        career_info::{dsl::career_info as career_info_table, user_account_id},
        consulting_fee::dsl::consulting_fee as consulting_fee_table,
        identity_info::dsl::identity_info as identity_info_table,
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
    util::{session::User, unexpected_err_resp, Career, Identity, Ymd},
};

pub(crate) async fn get_profile(
    User { account_id }: User,
    DatabaseConnection(conn): DatabaseConnection,
) -> RespResult<ProfileResult> {
    let profile_op = ProfileOperationImpl::new(conn);
    get_profile_internal(account_id, profile_op).await
}

async fn get_profile_internal(
    account_id: i32,
    profile_op: impl ProfileOperation,
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
    Ok((
        StatusCode::OK,
        Json(
            ProfileResult::email_address(account.email_address)
                .identity(identity)
                .careers(careers)
                .fee_per_hour_in_yen(fee_per_hour_in_yen)
                .finish(),
        ),
    ))
}

#[derive(Serialize, Debug)]
pub(crate) struct ProfileResult {
    pub email_address: String,
    pub identity: Option<Identity>,
    pub careers: Vec<Career>,
    pub fee_per_hour_in_yen: Option<i32>,
}

impl ProfileResult {
    fn email_address(email_address: String) -> ProfileResultBuilder {
        ProfileResultBuilder {
            email_address,
            identity: None,
            careers: vec![],
            fee_per_hour_in_yen: None,
        }
    }
}

struct ProfileResultBuilder {
    email_address: String,
    identity: Option<Identity>,
    careers: Vec<Career>,
    fee_per_hour_in_yen: Option<i32>,
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

    fn finish(self) -> ProfileResult {
        ProfileResult {
            email_address: self.email_address,
            identity: self.identity,
            careers: self.careers,
            fee_per_hour_in_yen: self.fee_per_hour_in_yen,
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
        telephone_number: identity_info.telephone_number,
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
        id: career_info.career_info_id,
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

trait ProfileOperation {
    fn find_user_account_by_user_account_id(&self, id: i32) -> Result<Account, ErrResp>;
    fn find_identity_info_by_user_account_id(
        &self,
        id: i32,
    ) -> Result<Option<IdentityInfo>, ErrResp>;
    fn filter_career_info_by_user_account_id(&self, id: i32) -> Result<Vec<CareerInfo>, ErrResp>;
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

#[cfg(test)]
mod tests {
    // use async_session::async_trait;
    // use axum::{http::StatusCode, Json};
    // use chrono::{TimeZone, Utc};
    // use common::{
    //     payment_platform::{
    //         charge::{Charge, ChargeOperation, Query as SearchChargesQuery},
    //         tenant::{Tenant, TenantOperation},
    //         tenant_transfer::{
    //             Query as SearchTenantTransfersQuery, TenantTransfer, TenantTransferOperation,
    //         },
    //         Error, List,
    //     },
    //     ApiError,
    // };

    // use crate::{err_code::NO_ACCOUNT_FOUND, util::JAPANESE_TIME_ZONE};

    // use super::{get_profile_internal, ProfileOperation};

    // struct ProfileOperationMock {
    //     account: common::model::user::Account,
    //     identity_info_option: Option<common::model::user::IdentityInfo>,
    //     careers_info: Vec<common::model::user::CareerInfo>,
    //     tenant_option: Option<common::model::user::Tenant>,
    //     consulting_fee_option: Option<common::model::user::ConsultingFee>,
    // }

    // impl ProfileOperation for ProfileOperationMock {
    //     fn find_user_account_by_user_account_id(
    //         &self,
    //         id: i32,
    //     ) -> Result<common::model::user::Account, common::ErrResp> {
    //         if self.account.user_account_id != id {
    //             return Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: NO_ACCOUNT_FOUND,
    //                 }),
    //             ));
    //         }
    //         Ok(self.account.clone())
    //     }

    //     fn find_identity_info_by_user_account_id(
    //         &self,
    //         _id: i32,
    //     ) -> Result<Option<common::model::user::IdentityInfo>, common::ErrResp> {
    //         Ok(self.identity_info_option.clone())
    //     }

    //     fn filter_career_info_by_user_account_id(
    //         &self,
    //         _id: i32,
    //     ) -> Result<Vec<common::model::user::CareerInfo>, common::ErrResp> {
    //         Ok(self.careers_info.clone())
    //     }

    //     fn find_consulting_fee_by_user_account_id(
    //         &self,
    //         _id: i32,
    //     ) -> Result<Option<common::model::user::ConsultingFee>, common::ErrResp> {
    //         Ok(self.consulting_fee_option.clone())
    //     }
    // }

    // struct TenantOperationMock<'a> {
    //     tenant_id: &'a str,
    // }

    // #[async_trait]
    // impl<'a> TenantOperation for TenantOperationMock<'a> {
    //     async fn get_tenant_by_tenant_id(&self, tenant_id: &str) -> Result<Tenant, Error> {
    //         todo!()
    //     }
    // }

    // struct ChargeOperationMock<'a> {
    //     query: &'a SearchChargesQuery,
    // }

    // #[async_trait]
    // impl<'a> ChargeOperation for ChargeOperationMock<'a> {
    //     async fn search_charges(&self, query: &SearchChargesQuery) -> Result<List<Charge>, Error> {
    //         todo!()
    //     }
    // }

    // struct TenantTransferOperationMock<'a> {
    //     query: &'a SearchTenantTransfersQuery,
    // }

    // #[async_trait]
    // impl<'a> TenantTransferOperation for TenantTransferOperationMock<'a> {
    //     async fn search_tenant_transfers(
    //         &self,
    //         query: &SearchTenantTransfersQuery,
    //     ) -> Result<List<TenantTransfer>, Error> {
    //         todo!()
    //     }
    // }

    #[tokio::test]
    async fn success_return_profile() {
        // let account_id = 51351;
        // let profile_op = ProfileOperationMock { account_id };
        // let tenant_id = "c8f0aa44901940849cbdb8b3e7d9f305";
        // let tenant_op = TenantOperationMock { tenant_id };
        // let search_charges_query = SearchChargesQuery::build()
        //     .finish()
        //     .expect("failed to get Ok");
        // let charge_op = ChargeOperationMock {
        //     query: &search_charges_query,
        // };
        // let current_datetime = Utc
        //     .ymd(2021, 12, 31)
        //     .and_hms(7, 0, 0)
        //     .with_timezone(&JAPANESE_TIME_ZONE.to_owned());
        // let search_tenant_transfers_query = SearchTenantTransfersQuery::build()
        //     .finish()
        //     .expect("failed to get Ok");
        // let tenant_transfer_op = TenantTransferOperationMock {
        //     query: &search_tenant_transfers_query,
        // };

        // let result = get_profile_internal(
        //     account_id,
        //     profile_op,
        //     tenant_op,
        //     charge_op,
        //     current_datetime,
        //     tenant_transfer_op,
        // )
        // .await
        // .expect("failed to get Ok");
    }
}
