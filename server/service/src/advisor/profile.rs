// Copyright 2021 Ken Miura

use crate::common;
use diesel::RunQueryDsl;
use actix_session::Session;
use actix_web::{get, web, HttpResponse};
use crate::advisor::authentication::session_state_inner;
use crate::common::error;
use crate::common::error::unexpected;
use diesel::QueryDsl;
use serde::Serialize;
use chrono::Datelike;

#[get("/profile-information")]
async fn profile_information(
    session: Session,
    pool: web::Data<common::ConnectionPool>,
) -> Result<HttpResponse, error::Error> {
    let option_id = session_state_inner(&session)?;
    let id = option_id.expect("Failed to get id");
    
    let conn = pool.get().map_err(|err| {
        let e = error::Error::Unexpected(unexpected::Error::R2d2Err(err));
        log::error!("failed to login: {}", e);
        e
    })?;

    let result = web::block(move || {
        use db::schema::career_change_supporter_schema::advisor_account::dsl::{
            advisor_account
        };
        let result: Result<db::model::advisor::AccountQueryResult, diesel::result::Error> = advisor_account.find(id).first::<db::model::advisor::AccountQueryResult>(&conn);
        if let Err(err) = result {
            return Err(err);
        }
        Ok(result.expect("Failed to get account"))
    }).await;
    let adv_acc = result.expect("Failed to get data");

    Ok(HttpResponse::Ok().json(Account{
        email_address: adv_acc.email_address,
        last_name: adv_acc.last_name,
        first_name: adv_acc.first_name,
        last_name_furigana: adv_acc.last_name_furigana,
        first_name_furigana: adv_acc.first_name_furigana,
        year: adv_acc.date_of_birth.year(),
        month: adv_acc.date_of_birth.month(),
        day: adv_acc.date_of_birth.day(),
        telephone_number: adv_acc.telephone_number,
        prefecture: adv_acc.prefecture,
        city: adv_acc.city,
        address_line1: adv_acc.address_line1,
        address_line2: adv_acc.address_line2.expect("Failed to get addr2")
    }))
}

#[derive(Serialize)]
struct Account {
    email_address: String,
    last_name: String,
    first_name: String,
    last_name_furigana: String,
    first_name_furigana: String,
    year: i32,
    month: u32,
    day: u32,
    telephone_number: String,
    prefecture: String,
    city: String,
    address_line1: String,
    address_line2: String
}
