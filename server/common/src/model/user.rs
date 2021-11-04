// Copyright 2021 Ken Miura

use chrono::DateTime;
use chrono::Utc;

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
