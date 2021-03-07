// Copyright 2021 Ken Miura

mod models;
mod schema;
mod static_assets_host;

#[macro_use]
extern crate diesel;

use actix_web::{
    error, post, web, App, HttpResponse,
    HttpServer, Result,
};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use dotenv::dotenv;
use serde::Deserialize;
use std::env;

#[derive(Deserialize)]
struct AuthInfo {
    mailaddress: String,
    password: String,
}

fn find_user_by_mail_address(
    mailaddress: &String,
    conn: &PgConnection,
) -> Result<Option<models::User>, diesel::result::Error> {
    use self::schema::user_data::user::dsl::*;
    let result = user.filter(mail_addr.eq(mailaddress))
        .first::<models::User>(conn)
    .optional()?;
    Ok(result)
}

#[post("/auth-request")]
async fn auth_request(info: web::Json<AuthInfo>, pool: web::Data<Pool<ConnectionManager<PgConnection>>>) -> HttpResponse {
    let mailaddress = info.mailaddress.clone();
    let password = info.password.clone();

    let conn = pool.get().expect("failed to get connection");

    let user = web::block(move || find_user_by_mail_address(&mailaddress, &conn)).await;

    let info = user.expect("error");
    let mut auth_res = false;
    match info {
        Some(user) => {
            auth_res = password == user.hashed_pass;
        },
        None => {}
    }

    if auth_res {
        let contents = "{ \"result\": \"OK\" }";
        HttpResponse::Ok().body(contents)
    } else {
        HttpResponse::from_error(error::ErrorUnauthorized("err: T"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager =
        ConnectionManager::<PgConnection>::new(&database_url);
    let pool: Pool<ConnectionManager<PgConnection>> = Pool::builder().build(manager).expect("failed to create connection pool");

    HttpServer::new(move || {
        App::new()
            .service(actix_files::Files::new(static_assets_host::ASSETS_DIR, ".").show_files_listing())
            .service(static_assets_host::js)
            .service(static_assets_host::css)
            .service(static_assets_host::img)
            .service(static_assets_host::index)
            .service(auth_request)
            .default_service(web::route().to(static_assets_host::serve_index))
            .data(pool.clone())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
