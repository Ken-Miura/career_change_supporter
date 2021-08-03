// Copyright 2021 Ken Miura

use crate::advisor::authentication::check_advisor_session_state;
use crate::common;
use crate::common::error;
use crate::common::error::unexpected;
use actix_session::Session;
use actix_web::{client, get, http::StatusCode, web, HttpResponse};
use chrono::Datelike;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use openssl::ssl::{SslConnector, SslMethod};
use serde::Serialize;

#[get("/profile-information")]
async fn profile_information(
    session: Session,
    pool: web::Data<common::ConnectionPool>,
) -> Result<HttpResponse, error::Error> {
    let option_id = check_advisor_session_state(&session)?;
    let id = option_id.expect("Failed to get id");

    let conn = pool.get().map_err(|err| {
        let e = error::Error::Unexpected(unexpected::Error::R2d2Err(err));
        log::error!("failed to login: {}", e);
        e
    })?;

    let result = web::block(move || {
        use db::schema::career_change_supporter_schema::advisor_account::dsl::advisor_account;
        let result: Result<db::model::advisor::AccountQueryResult, diesel::result::Error> =
            advisor_account
                .find(id)
                .first::<db::model::advisor::AccountQueryResult>(&conn);
        if let Err(err) = result {
            return Err(err);
        }
        Ok(result.expect("Failed to get account"))
    })
    .await;
    let adv_acc = result.expect("Failed to get data");
    let val = adv_acc.sex.clone();
    let sex = if val == "male" { "男性" } else { "女性" };
    match adv_acc.tenant_id {
        Some(t_id) => {
            // TODO: どのような設定に気をつけなければならないか確認する
            // https://github.com/actix/examples/tree/master/security/awc_https
            let ssl_builder = SslConnector::builder(SslMethod::tls()).unwrap();
            let client_builder = client::ClientBuilder::new();
            let secret_key = common::PAYJP_TEST_SECRET_KEY.to_string();
            let password = common::PAYJP_TEST_PASSWORD.to_string();
            let client = client_builder
                .connector(client::Connector::new().ssl(ssl_builder.build()).finish())
                .basic_auth(&secret_key, Some(&password))
                .finish();

            let result = client
                .get(format!("https://api.pay.jp/v1/tenants/{}", t_id))
                .send()
                .await; // <- Wait for response

            // https://github.com/actix/actix-web/issues/536#issuecomment-579380701
            let mut response = result.expect("test");
            let result = response.json::<crate::advisor::account::Tenant>().await;
            if response.status() != StatusCode::OK {
                let result = response.json::<crate::advisor::account::ErrorSt>().await;
                let tenant = result.expect("test");
                log::info!("{:?}", tenant);
                panic!("Err");
            }
            let tenant = result.expect("test");
            Ok(HttpResponse::Ok().json(Account {
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
                address_line2: adv_acc.address_line2.expect("Failed to get addr2"),
                sex: sex.to_string(),
                bank_code: tenant.bank_code,
                bank_branch_code: tenant.bank_branch_code,
                bank_account_number: tenant.bank_account_number,
                bank_account_holder_name: tenant.bank_account_holder_name,
                advice_fee_in_yen: adv_acc.advice_fee_in_yen,
            }))
        }
        None => Ok(HttpResponse::Ok().json(Account {
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
            address_line2: adv_acc.address_line2.expect("Failed to get addr2"),
            sex: sex.to_string(),
            bank_code: "no bank code found".to_string(),
            bank_branch_code: "no bank branch code found".to_string(),
            bank_account_number: "no bank account number found".to_string(),
            bank_account_holder_name: "no account holder name found".to_string(),
            advice_fee_in_yen: adv_acc.advice_fee_in_yen,
        })),
    }
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
    address_line2: String,
    sex: String,
    bank_code: String,
    bank_branch_code: String,
    bank_account_number: String,
    bank_account_holder_name: String,
    advice_fee_in_yen: Option<i32>,
}
