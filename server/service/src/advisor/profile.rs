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
    }))
}

#[derive(Serialize)]
struct Account {
    email_address: String,
    // last_name: String,
    // first_name: String,
    // last_name_furigana: String,
    // first_name_furigana: String,
    // year: i16,
    // month: i16,
    // day: i16,
    // telephone_number: String,
    // prefecture: String,
    // city: String,
    // addressline1: String,
    // addressline2: String
}
