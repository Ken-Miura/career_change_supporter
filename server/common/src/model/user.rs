// Copyright 2021 Ken Miura

// use crate::schema::career_change_supporter_schema::user_account;
use crate::schema::ccs_schema::user_temp_account;

#[derive(Insertable, Debug)]
#[table_name = "user_temp_account"]
pub struct NewTempAccount<'a> {
    pub user_temp_account_id: &'a str,
    pub email_address: &'a str,
    pub hashed_password: &'a [u8],
    pub created_at: &'a chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Queryable)]
pub struct TempAccount {
    pub user_temp_account_id: String,
    pub email_address: String,
    pub hashed_password: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

// #[derive(Insertable)]
// #[table_name = "user_account"]
// pub struct Account<'a> {
//     pub email_address: &'a str,
//     pub hashed_password: &'a [u8],
//     pub last_login_time: Option<&'a chrono::DateTime<chrono::Utc>>,
// }

// #[derive(Clone, Queryable)]
// pub struct AccountQueryResult {
//     pub user_account_id: i32,
//     pub email_address: String,
//     pub hashed_password: Vec<u8>,
//     pub last_login_time: Option<chrono::DateTime<chrono::Utc>>,
// }
