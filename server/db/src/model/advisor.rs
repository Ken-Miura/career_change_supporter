// Copyright 2021 Ken Miura

use crate::schema::career_change_supporter_schema::advisor_registration_request;
#[derive(Insertable)]
#[table_name = "advisor_registration_request"]
pub struct RegistrationRequest<'a> {
    pub advisor_registration_request_id: &'a str,
    pub email_address: &'a str,
    pub created_at: &'a chrono::DateTime<chrono::Utc>,
}
