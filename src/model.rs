// Copyright 2021 Ken Miura

use super::schema::my_project_schema::user_account;
use super::schema::my_project_schema::user_temporary_account;

#[derive(Insertable)]
#[table_name = "user_temporary_account"]
pub struct TemporaryAccount<'a> {
    pub user_temporary_account_id: &'a str,
    pub email_address: &'a str,
    pub hashed_password: &'a [u8],
    pub created_at: &'a chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Queryable)]
pub struct TemporaryAccountQueryResult {
    pub user_temporary_account_id: String,
    pub email_address: String,
    pub hashed_password: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Insertable)]
#[table_name = "user_account"]
pub struct Account<'a> {
    pub email_address: &'a str,
    pub hashed_password: &'a [u8],
    pub last_login_time: Option<&'a chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Queryable)]
pub struct AccountQueryResult {
    pub user_account_id: i32,
    pub email_address: String,
    pub hashed_password: Vec<u8>,
    pub last_login_time: Option<chrono::DateTime<chrono::Utc>>,
}
