// Copyright 2021 Ken Miura

extern crate diesel;
extern crate dotenv;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("ADMINISTRATOR_TOOL_DATABASE_URL")
        .expect("ADMINISTRATOR_TOOL_DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let cmd = &args[1];
    if cmd == "create" {
        let id = &args[2];
        let password = &args[3];
        let hashed_pwd = bcrypt::hash(password, BCRYPT_COST).expect("Failed to hash password");
        let conn = establish_connection();
        use db::schema::career_change_supporter_schema::administrator_account;
        let acc = db::model::administrator::Account {
            email_address: &id,
            hashed_password: &hashed_pwd.as_bytes(),
            last_login_time: None,
        };
        let result = diesel::insert_into(administrator_account::table)
            .values(acc)
            .execute(&conn);
        result.expect("Failed to insert value");
    } else if cmd == "update" {
        let id = &args[2];
        let password = &args[3];
        let hashed_pwd = bcrypt::hash(password, BCRYPT_COST).expect("Failed to hash password");
        let conn = establish_connection();
        use db::schema::career_change_supporter_schema::administrator_account::dsl::{
            administrator_account, email_address, hashed_password,
        };
        let target = administrator_account.filter(email_address.eq(&id));
        let result = diesel::update(target)
            .set(hashed_password.eq(&hashed_pwd.as_bytes()))
            .execute(&conn);
        result.expect("Failed to update value");
    } else if cmd == "delete" {
        let id = &args[2];
        let conn = establish_connection();
        use db::schema::career_change_supporter_schema::administrator_account::dsl::{
            administrator_account, email_address,
        };
        let target = administrator_account.filter(email_address.eq(&id));
        let result = diesel::delete(target).execute(&conn);
        result.expect("Failed to delete value");
    } else {
        panic!("invalid command");
    }
}

const BCRYPT_COST: u32 = 7;
