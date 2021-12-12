// Copyright 2021 Ken Miura

use chrono::DateTime;
use chrono::NaiveDate;
use chrono::Utc;

use crate::schema::ccs_schema::career_info;
use crate::schema::ccs_schema::consulting_fee;
use crate::schema::ccs_schema::identity_info;
use crate::schema::ccs_schema::new_password;
use crate::schema::ccs_schema::tenant;
use crate::schema::ccs_schema::terms_of_use;
use crate::schema::ccs_schema::user_account;
use crate::schema::ccs_schema::user_temp_account;

#[derive(Insertable, Debug)]
#[table_name = "user_temp_account"]
pub struct NewTempAccount<'a> {
    pub user_temp_account_id: &'a str,
    pub email_address: &'a str,
    pub hashed_password: &'a [u8],
    pub created_at: &'a DateTime<Utc>,
}

#[derive(Clone, Queryable)]
pub struct TempAccount {
    pub user_temp_account_id: String,
    pub email_address: String,
    pub hashed_password: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[table_name = "user_account"]
pub struct NewAccount<'a> {
    pub email_address: &'a str,
    pub hashed_password: &'a [u8],
    pub last_login_time: Option<&'a DateTime<Utc>>,
    pub created_at: &'a DateTime<Utc>,
}

#[derive(Clone, Queryable)]
pub struct Account {
    pub user_account_id: i32,
    pub email_address: String,
    pub hashed_password: Vec<u8>,
    pub last_login_time: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[table_name = "terms_of_use"]
pub struct NewTermsOfUse<'a> {
    pub user_account_id: &'a i32,
    pub ver: &'a i32,
    pub email_address: &'a str,
    pub agreed_at: &'a DateTime<Utc>,
}

#[derive(Clone, Queryable)]
pub struct TermsOfUse {
    pub user_account_id: i32,
    pub ver: i32,
    pub email_address: String,
    pub agreed_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[table_name = "new_password"]
pub struct NewNewPassword<'a> {
    pub new_password_id: &'a str,
    pub email_address: &'a str,
    pub hashed_password: &'a [u8],
    pub created_at: &'a DateTime<Utc>,
}

#[derive(Clone, Queryable)]
pub struct NewPassword {
    pub new_password_id: String,
    pub email_address: String,
    pub hashed_password: Vec<u8>,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[table_name = "identity_info"]
pub struct NewIdentityInfo<'a> {
    pub user_account_id: &'a i32,
    pub last_name: &'a str,
    pub first_name: &'a str,
    pub last_name_furigana: &'a str,
    pub first_name_furigana: &'a str,
    pub sex: &'a str,
    pub date_of_birth: &'a NaiveDate,
    pub prefecture: &'a str,
    pub city: &'a str,
    pub address_line1: &'a str,
    pub address_line2: Option<&'a str>,
    pub telephone_number: &'a str,
}

#[derive(Clone, Queryable)]
pub struct IdentityInfo {
    pub user_account_id: i32,
    pub last_name: String,
    pub first_name: String,
    pub last_name_furigana: String,
    pub first_name_furigana: String,
    pub sex: String,
    pub date_of_birth: NaiveDate,
    pub prefecture: String,
    pub city: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub telephone_number: String,
}

#[derive(Insertable, Debug)]
#[table_name = "career_info"]
pub struct NewCareerInfo<'a> {
    pub user_account_id: &'a i32,
    pub company_name: &'a str,
    pub department_name: Option<&'a str>,
    pub office: Option<&'a str>,
    pub career_start_date: &'a NaiveDate,
    pub career_end_date: Option<&'a NaiveDate>,
    pub contract_type: &'a str,
    pub profession: Option<&'a str>,
    pub annual_income_in_man_yen: Option<&'a i32>,
    pub is_manager: &'a bool,
    pub position_name: Option<&'a str>,
    pub is_new_graduate: &'a bool,
    pub note: Option<&'a str>,
}

#[derive(Clone, Queryable)]
pub struct CareerInfo {
    pub career_info_id: i32,
    pub user_account_id: i32,
    pub company_name: String,
    pub department_name: Option<String>,
    pub office: Option<String>,
    pub career_start_date: NaiveDate,
    pub career_end_date: Option<NaiveDate>,
    pub contract_type: String,
    pub profession: Option<String>,
    pub annual_income_in_man_yen: Option<i32>,
    pub is_manager: bool,
    pub position_name: Option<String>,
    pub is_new_graduate: bool,
    pub note: Option<String>,
}

#[derive(Insertable)]
#[table_name = "consulting_fee"]
pub struct NewConsultingFee<'a> {
    pub user_account_id: &'a i32,
    pub fee_per_hour_in_yen: &'a i32,
}

#[derive(Clone, Queryable)]
pub struct ConsultingFee {
    pub user_account_id: i32,
    pub fee_per_hour_in_yen: i32,
}

#[derive(Insertable)]
#[table_name = "tenant"]
pub struct NewTenant<'a> {
    pub user_account_id: &'a i32,
    pub tenant_id: &'a str,
}

#[derive(Clone, Queryable)]
pub struct Tenant {
    pub user_account_id: i32,
    pub tenant_id: String,
}
