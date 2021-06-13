// Copyright 2021 Ken Miura

use crate::schema::career_change_supporter_schema::administrator_account;

#[derive(Insertable)]
#[table_name = "administrator_account"]
pub struct Account<'a> {
    pub email_address: &'a str,
    pub hashed_password: &'a [u8],
    pub last_login_time: Option<&'a chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Queryable)]
pub struct AccountQueryResult {
    pub administrator_account_id: i32,
    pub email_address: String,
    pub hashed_password: Vec<u8>,
    pub last_login_time: Option<chrono::DateTime<chrono::Utc>>,
}
