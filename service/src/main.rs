// Copyright 2021 Ken Miura

mod advisor;
mod common;
mod schema;
mod static_asset;
mod user;

// TODO: #[macro_use]なしでdieselのマクロが使えるように変更が入った際に取り除く
// https://github.com/diesel-rs/diesel/issues/1764
#[macro_use]
extern crate diesel;

use actix_web::{middleware::Logger, web, App, HttpServer};
use dotenv::dotenv;
use log::LevelFilter;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

const APPLICATION_SERVER_ADDR: &str = "127.0.0.1:8080";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

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
            // NOTE: /user (suffixに"/"なし) にアクセスした際に404となるので、
            // /userにアクセスしてきた際に/user/user_app.htmlにリダイレクトする
            .service(web::resource("/user").to(static_asset::redirect_to_user_app))
            .configure(user::user_config)
            // NOTE: 上記のNOTEと同様の理由で記載
            .service(web::resource("/advisor").to(static_asset::redirect_to_advisor_app))
            .configure(advisor::advisor_config)
            // NOTE: 下記のrefに従い、"/"は最後に記載する
            // ref: https://docs.rs/actix-files/0.5.0/actix_files/struct.Files.html#implementation-notes
            .service(
                actix_files::Files::new("/", static_asset::ASSETS_DIR.to_string())
                    .prefer_utf8(true)
                    .index_file("index.html")
                    .default_handler(web::route().to(static_asset::serve_index)),
            )
    })
    .bind(APPLICATION_SERVER_ADDR)?
    .run()
    .await
}
