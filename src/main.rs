// Copyright 2021 Ken Miura

mod authentication;
mod error_codes;
mod models;
mod schema;
mod static_assets_host;
mod utils;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate diesel;

use actix_web::{cookie, error, get, middleware::Logger, web, App, HttpResponse, HttpServer};
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use dotenv::dotenv;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use std::env;
use time::Duration;

use actix_redis::RedisSession;
use actix_session::Session;

#[get("/profile-information")]
async fn profile_information(
    session: Session,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> HttpResponse {
    // TODO: Handle Result
    let session_info: Option<String> = session.get("email_address").unwrap_or(None);
    if session_info == None {
        return HttpResponse::from_error(error::ErrorUnauthorized("failed to authenticate"));
    }
    // セッションのttlがgetしただけで伸びるか確認する。

    let conn = pool.get().expect("failed to get connection");
    let email_address = session_info.expect("never happen");
    let user = web::block(move || utils::find_user_by_mail_address(&email_address, &conn)).await;
    let user_info = user.expect("error");

    match user_info {
        Some(user) => {
            let json_text = format!(
                "{{ \"id\": \"{}\", \"email_address\": \"{}\"}}",
                user.id, user.email_address
            );
            HttpResponse::Ok().body(json_text)
        }
        None => HttpResponse::from_error(error::ErrorUnauthorized("failed to authenticate")),
    }
}

const CACHE_SERVER_ADDR: &str = "127.0.0.1:6379";
const APPLICATION_SERVER_ADDR: &str = "127.0.0.1:8080";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool: Pool<ConnectionManager<PgConnection>> = Pool::builder()
        .build(manager)
        .expect("failed to create connection pool");

    // TODO: Check pattern encoder
    // TODO: 記録される時間がサーバ上の時間か、クライアントのリクエスト時の時間が確認する
    // TODO: ECS fargateとCloudWatchLogの連携を利用するために標準出力 (env_logger) を検討する
    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} {l} {t} - {m}{n}")))
        .build("log/output.log")
        .expect("never happens panic");

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))
        .expect("never happens panic");

    // TODO: Add error handling
    let _ = log4rs::init_config(config);

    // TODO: DOS攻撃を回避するために受け取るJSONデータのサイズ制限を追加する
    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .wrap(
                RedisSession::new(CACHE_SERVER_ADDR, &authentication::SESSION_SIGN_KEY)
                    .ttl(180)
                    .cookie_max_age(Duration::days(7))
                    // TODO: Add producion environment
                    //.cookie_secure(true)
                    .cookie_name("session")
                    .cookie_http_only(true)
                    // TODO: Consider LAX policy
                    .cookie_same_site(cookie::SameSite::Strict),
            )
            .service(
                actix_files::Files::new(static_assets_host::ASSETS_DIR, ".").show_files_listing(),
            )
            .service(static_assets_host::js)
            .service(static_assets_host::css)
            .service(static_assets_host::img)
            .service(static_assets_host::index)
            .service(authentication::auth_request)
            .service(authentication::registration_request)
            .service(authentication::logout_request)
            .service(authentication::session_state)
            .service(profile_information)
            .default_service(web::route().to(static_assets_host::serve_index))
            .data(pool.clone())
    })
    .bind(APPLICATION_SERVER_ADDR)?
    .run()
    .await
}
