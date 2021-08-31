// Copyright 2021 Ken Miura

mod err_code;
mod temp_accounts;
mod util;

use crate::temp_accounts::post_temp_accounts;
use axum::handler::post;
use axum::{AddExtensionLayer, Router};
use common::smtp::KEY_TO_SOCKET_FOR_SMTP_SERVER;
use common::util::check_env_vars;
use common::{ConnectionPool, KEY_TO_URL_FOR_FRONT_END};
use diesel::{r2d2::ConnectionManager, r2d2::Pool, PgConnection};
use dotenv::dotenv;
use once_cell::sync::Lazy;
use std::env::set_var;
use std::env::var;

const KEY_TO_DATABASE_URL: &str = "DB_URL_FOR_USER_APP";
const KEY_TO_SOCKET: &str = "SOCKET_FOR_USER_APP";

/// アプリケーションの動作に必須の環境変数をすべて列挙し、
/// 起動直後に存在をチェックする
static ENV_VARS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        KEY_TO_DATABASE_URL.to_string(),
        KEY_TO_SOCKET.to_string(),
        KEY_TO_SOCKET_FOR_SMTP_SERVER.to_string(),
        KEY_TO_URL_FOR_FRONT_END.to_string(),
    ]
});

fn main() {
    let _ = dotenv().ok();
    let result = check_env_vars(ENV_VARS.to_vec());
    if result.is_err() {
        println!("failed to resolve mandatory env vars (following env vars are needed)");
        println!("{:?}", result.unwrap_err());
        std::process::exit(1);
    }
    let num = num_cpus::get();
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num)
        .enable_all()
        .build()
        .expect("failed to build Runtime")
        .block_on(main_internal(num as u32))
}

async fn main_internal(num_of_cpus: u32) {
    set_var(
        "RUST_LOG",
        "user_service=debug,common=debug,tower_http=debug",
    );
    tracing_subscriber::fmt::init();

    let database_url = var(KEY_TO_DATABASE_URL).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_DATABASE_URL
        )
    });
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    // NOTE: bb8-dieselのcrate (https://crates.io/crates/bb8-diesel) がtokio 1.0系統に対応した後、r2d2からの移行を検討する
    let pool: ConnectionPool = Pool::builder()
        .max_size(num_of_cpus)
        .build(manager)
        .expect("failed to build connection pool");

    let app = Router::new()
        .nest(
            "/api/users",
            Router::new().route("/temp-accounts", post(post_temp_accounts)),
            //Router::new().route("/hello", post(|| async { "hello" })),
        )
        .layer(AddExtensionLayer::new(pool));

    let socket = var(KEY_TO_SOCKET).unwrap_or_else(|_| {
            panic!(
                "Not environment variable found: environment variable \"{}\" (example value: \"127.0.0.1:3000\") must be set",
                KEY_TO_SOCKET
            )
        });
    let addr = socket
        .parse()
        .unwrap_or_else(|_| panic!("failed to parse socket: {}", socket));
    tracing::info!("listening on {}", addr);
    let _ = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to serve app");
}
