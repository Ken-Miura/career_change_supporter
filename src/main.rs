// Copyright 2021 Ken Miura

mod models;
mod schema;
mod static_assets_host;

#[macro_use]
extern crate diesel;

use actix_web::{error, post, web, App, HttpResponse, HttpServer, Result};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use dotenv::dotenv;
use serde::Deserialize;
use serde::Serialize;
use std::env;

#[post("/auth-request")]
async fn auth_request(
    info: web::Json<AuthInfo>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> HttpResponse {
    // TODO: Validate email address and password
    let mail_addr = info.email_address.clone();
    // TODO: hash password
    let pwd = info.password.clone();

    let conn = pool.get().expect("failed to get connection");

    let user = web::block(move || find_user_by_mail_address(&mail_addr, &conn)).await;

    let info = user.expect("error");
    let mut auth_res = false;
    match info {
        Some(user) => {
            use ring::hmac;
            let key = hmac::Key::new(hmac::HMAC_SHA512, &KEY_VALUE);
            let result = hmac::verify(&key, pwd.as_bytes(), &user.hashed_password);
            match result {
                Ok(_) => auth_res = true,
                Err(_) => auth_res = false,
            }
        }
        None => {}
    }

    if auth_res {
        let contents = "{ \"result\": \"OK\" }";
        HttpResponse::Ok().body(contents)
    } else {
        HttpResponse::from_error(error::ErrorUnauthorized("failed to authenticate"))
    }
}

#[derive(Deserialize)]
struct AuthInfo {
    email_address: String,
    password: String,
}

fn find_user_by_mail_address(
    mail_addr: &String,
    conn: &PgConnection,
) -> Result<Option<models::User>, diesel::result::Error> {
    use self::schema::my_project_schema::user::dsl::*;
    let result = user
        .filter(email_address.eq(mail_addr))
        .first::<models::User>(conn)
        .optional()?;
    Ok(result)
}

const KEY_VALUE: [u8; 4] = [0, 1, 2, 3];

#[post("/registration-request")]
async fn registration_request(
    info: web::Json<RegistrationInfo>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> HttpResponse {
    if !check_if_email_address_format_is_valid(&info.email_address) {
        return HttpResponse::from_error(error::ErrorBadRequest("failed to register account"));
    }

    // ランダムパスワード生成＋ハッシュ化
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    let rand_password: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16) // TODO: Consider enough length
        .map(char::from)
        .collect();
    use ring::hmac;
    let key = hmac::Key::new(hmac::HMAC_SHA512, &KEY_VALUE);
    let hashed_password = hmac::sign(&key, rand_password.as_bytes());

    // トランザクションで、既存のDBにメールアドレスがあるかチェック＋登録
    // TODO: メールアドレスにUnique制約を追加するのか、トランザクションを利用するのか確認する
    let conn = pool.get().expect("failed to get connection");
    let result =
        web::block(move || register_account(&info.email_address, hashed_password.as_ref(), &conn))
            .await;

    match result {
        Ok(num) => print!("{}", num),
        Err(err) => print!("{}", err), // reach here if unique violation 
    }

    // 登録されているならメールアドレス宛にパスワードを送信

    // Debug output
    use core::fmt::Write;
    let mut s = String::with_capacity(64);
    for byte in hashed_password.as_ref() {
        write!(s, "{:02X}", byte);
    }
    let json_text = format!(
        "{{ \"rand_password\": \"{}\", \"hashed_password\": \"{}\" }}",
        rand_password, s
    );
    HttpResponse::Ok().body(json_text)
}

use self::models::Account;

fn register_account(
    mail_addr: &String,
    hashed_pwd: &[u8],
    conn: &PgConnection,
) -> Result<usize, diesel::result::Error> {
    use schema::my_project_schema::user;
    let new_account = Account {
        email_address: mail_addr,
        hashed_password: hashed_pwd,
    };

    let result = diesel::insert_into(user::table)
        .values(&new_account)
        .execute(conn);
    result
}

#[derive(Serialize, Deserialize)]
struct RegistrationInfo {
    email_address: String,
}

// TODO: Improve name and signature
fn check_if_email_address_format_is_valid(mail_addr: &String) -> bool {
    const MAX_LENGTH: usize = 254;
    if mail_addr.len() > MAX_LENGTH {
        return false;
    }
    // TODO: Add regular expression check
    // TODO: Investigate regular expression
    //const EMAIL_REGEXP: &str = "^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$";
    return true;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool: Pool<ConnectionManager<PgConnection>> = Pool::builder()
        .build(manager)
        .expect("failed to create connection pool");

    HttpServer::new(move || {
        App::new()
            .service(
                actix_files::Files::new(static_assets_host::ASSETS_DIR, ".").show_files_listing(),
            )
            .service(static_assets_host::js)
            .service(static_assets_host::css)
            .service(static_assets_host::img)
            .service(static_assets_host::index)
            .service(auth_request)
            .service(registration_request)
            .default_service(web::route().to(static_assets_host::serve_index))
            .data(pool.clone())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
