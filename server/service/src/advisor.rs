// Copyright 2021 Ken Miura

mod authentication;
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
const ADVISOR_SESSION_SIGN_KEY: [u8; 32] = [1; 32];

pub(super) fn advisor_config(cfg: &mut web::ServiceConfig) {
    let database_url =
        env::var("ADVISOR_APP_DATABASE_URL").expect("ADVISOR_APP_DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool: common::ConnectionPool = Pool::builder()
        /*
         * NOTE: actixのarchitectureより、1 Actor（1スレッド）ごとにconnection poolを作成して割り当てる。
         * 1スレッドあたり1コネクションで十分と思われるため、max_sizeを1に設定する。
         */
        .max_size(1)
        .build(manager)
        .expect("never fails to create connection pool");

    cfg.service(
        web::scope("/advisor/")
            .wrap(
                RedisSession::new(common::CACHE_SERVER_ADDR, &ADVISOR_SESSION_SIGN_KEY)
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
                    .cookie_path("/advisor/"),
            )
            .service(crate::advisor::authentication::session_state)
            // NOTE: 下記のrefに従い、"/"は最後に記載する
            // ref: https://docs.rs/actix-files/0.5.0/actix_files/struct.Files.html#implementation-notes
            .service(
                actix_files::Files::new(
                    "/",
                    crate::advisor::static_asset::ADVISOR_ASSETS_DIR.to_string(),
                )
                .prefer_utf8(true)
                .index_file("advisor_app.html")
                .default_handler(web::route().to(crate::advisor::static_asset::serve_advisor_app)),
            )
            .data(pool),
    );
}
