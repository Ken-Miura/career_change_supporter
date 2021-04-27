// Copyright 2021 Ken Miura

mod account;
mod authentication;
mod model;
mod profile;
mod static_asset;

use actix_web::{cookie, web};
use time::Duration;

use actix_redis::RedisSession;

use std::env;

use crate::common;

use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

// TODO: Consider and change KEY
const USER_SESSION_SIGN_KEY: [u8; 32] = [1; 32];

pub(super) fn user_config(cfg: &mut web::ServiceConfig) {
    let database_url = env::var("USER_APP_DATABASE_URL").expect("USER_APP_DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool: common::ConnectionPool = Pool::builder()
        .build(manager)
        .expect("never fails to create connection pool");

    cfg.service(
        web::scope("/user/")
            .wrap(
                RedisSession::new(common::CACHE_SERVER_ADDR, &USER_SESSION_SIGN_KEY)
                    // TODO: 適切なTTLを設定する
                    .ttl(180)
                    .cookie_max_age(Duration::days(7))
                    // TODO: Add producion environment
                    //.cookie_secure(true)
                    .cookie_name("session")
                    .cookie_http_only(true)
                    // TODO: Consider LAX policy
                    .cookie_same_site(cookie::SameSite::Strict)
                    // NOTE: web::scopeで自動的に設定されるわけではないので明示的に指定する
                    .cookie_path("/user/"),
            )
            .service(crate::user::static_asset::temporary_accounts)
            .service(crate::user::account::temporary_account_creation)
            .service(crate::user::account::account_creation)
            .service(crate::user::authentication::login_request)
            .service(crate::user::authentication::logout_request)
            .service(crate::user::authentication::session_state)
            .service(crate::user::profile::profile_information)
            // NOTE: 下記のrefに従い、"/"は最後に記載する
            // ref: https://docs.rs/actix-files/0.5.0/actix_files/struct.Files.html#implementation-notes
            .service(
                actix_files::Files::new(
                    "/",
                    crate::user::static_asset::USER_ASSETS_DIR.to_string(),
                )
                .prefer_utf8(true)
                .index_file("user_app.html")
                .default_handler(web::route().to(crate::user::static_asset::serve_user_app)),
            )
            .data(pool),
    );
}
