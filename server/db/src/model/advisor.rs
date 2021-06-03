// Copyright 2021 Ken Miura

use crate::schema::career_change_supporter_schema::advisor_account_creation_request;
use crate::schema::career_change_supporter_schema::advisor_registration_request;

#[derive(Insertable)]
#[table_name = "advisor_registration_request"]
pub struct RegistrationRequest<'a> {
    pub advisor_registration_request_id: &'a str,
    pub email_address: &'a str,
    pub created_at: &'a chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Queryable)]
pub struct RegistrationRequestQueryResult {
    pub advisor_registration_request_id: String,
    pub email_address: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Insertable)]
#[table_name = "advisor_account_creation_request"]
pub struct AccountCreationRequest<'a> {
    pub email_address: &'a str,
    pub hashed_password: &'a [u8],
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
    pub requested_time: &'a chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Queryable)]
pub struct AccountCreationRequestResult {
    pub advisor_acc_request_id: i32,
    pub email_address: String,
    pub hashed_password: Vec<u8>,
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
    pub requested_time: chrono::DateTime<chrono::Utc>,
}
