// Copyright 2021 Ken Miura

use crate::schema::career_change_supporter_schema::administrator_account;
use crate::schema::career_change_supporter_schema::advisor_reg_req_approved;
use crate::schema::career_change_supporter_schema::advisor_reg_req_rejected;

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

#[derive(Insertable)]
#[table_name = "advisor_reg_req_approved"]
pub struct AdvisorRegReqApproved<'a> {
    pub email_address: &'a str,
    pub last_name: &'a str,
    pub first_name: &'a str,
    pub last_name_furigana: &'a str,
    pub first_name_furigana: &'a str,
    pub telephone_number: &'a str,
    pub year_of_birth: i16,
    pub month_of_birth: i16,
    pub day_of_birth: i16,
    pub prefecture: &'a str,
    pub city: &'a str,
    pub address_line1: &'a str,
    pub address_line2: Option<&'a str>,
    pub image1: &'a str,
    pub image2: Option<&'a str>,
    pub associated_advisor_account_id: Option<i32>,
    pub approved_time: &'a chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Queryable)]
pub struct AdvisorRegReqApprovedResult {
    pub advisor_reg_req_approved_id: i32,
    pub email_address: String,
    pub last_name: String,
    pub first_name: String,
    pub last_name_furigana: String,
    pub first_name_furigana: String,
    pub telephone_number: String,
    pub year_of_birth: i16,
    pub month_of_birth: i16,
    pub day_of_birth: i16,
    pub prefecture: String,
    pub city: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub image1: String,
    pub image2: Option<String>,
    pub associated_advisor_account_id: Option<i32>,
    pub approved_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Insertable)]
#[table_name = "advisor_reg_req_rejected"]
pub struct AdvisorRegReqRejected<'a> {
    pub email_address: &'a str,
    pub last_name: &'a str,
    pub first_name: &'a str,
    pub last_name_furigana: &'a str,
    pub first_name_furigana: &'a str,
    pub telephone_number: &'a str,
    pub year_of_birth: i16,
    pub month_of_birth: i16,
    pub day_of_birth: i16,
    pub prefecture: &'a str,
    pub city: &'a str,
    pub address_line1: &'a str,
    pub address_line2: Option<&'a str>,
    pub reject_reason: &'a str,
    pub rejected_time: &'a chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Queryable)]
pub struct AdvisorRegReqRejectedResult {
    pub advisor_reg_req_rejected_id: i32,
    pub email_address: String,
    pub last_name: String,
    pub first_name: String,
    pub last_name_furigana: String,
    pub first_name_furigana: String,
    pub telephone_number: String,
    pub year_of_birth: i16,
    pub month_of_birth: i16,
    pub day_of_birth: i16,
    pub prefecture: String,
    pub city: String,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub reject_reason: String,
    pub rejected_time: chrono::DateTime<chrono::Utc>,
}
