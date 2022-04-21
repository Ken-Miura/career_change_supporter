// Copyright 2021 Ken Miura

mod accounts;
mod agreement;
mod err;
mod identity;
mod login;
mod logout;
mod password_change_req;
mod password_update;
mod profile;
mod refresh;
mod rewards;
mod temp_accounts;
mod util;

use crate::accounts::post_accounts;
use crate::agreement::post_agreement;
use crate::identity::post_identity;
use crate::login::post_login;
use crate::logout::post_logout;
use crate::password_change_req::post_password_change_req;
use crate::password_update::post_password_update;
use crate::profile::get_profile;
use crate::refresh::get_refresh;
use crate::rewards::get_reward;
use crate::temp_accounts::post_temp_accounts;
use crate::util::session::KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_USER_APP;
use crate::util::terms_of_use::KEY_TO_TERMS_OF_USE_VERSION;
use crate::util::ROOT_PATH;
use async_redis_session::RedisSessionStore;
use axum::extract::Extension;
use axum::routing::{get, post};
use axum::Router;
use common::payment_platform::{
    KEY_TO_PAYMENT_PLATFORM_API_PASSWORD, KEY_TO_PAYMENT_PLATFORM_API_URL,
    KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
};
use common::redis::KEY_TO_URL_FOR_REDIS_SERVER;
use common::smtp::KEY_TO_SOCKET_FOR_SMTP_SERVER;
use common::storage::{
    KEY_TO_AWS_ACCESS_KEY_ID, KEY_TO_AWS_REGION, KEY_TO_AWS_S3_ENDPOINT_URI,
    KEY_TO_AWS_SECRET_ACCESS_KEY,
};
use common::util::check_env_vars;
use common::KEY_TO_URL_FOR_FRONT_END;
use dotenv::dotenv;
use entity::sea_orm::{ConnectOptions, Database};
use hyper::{Body, Request};
use once_cell::sync::Lazy;
use std::env::set_var;
use std::env::var;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;
use tracing::Level;
use uuid::Uuid;

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
        KEY_TO_URL_FOR_REDIS_SERVER.to_string(),
        KEY_TO_TERMS_OF_USE_VERSION.to_string(),
        KEY_TO_PAYMENT_PLATFORM_API_URL.to_string(),
        KEY_TO_PAYMENT_PLATFORM_API_USERNAME.to_string(),
        KEY_TO_PAYMENT_PLATFORM_API_PASSWORD.to_string(),
        KEY_TO_AWS_S3_ENDPOINT_URI.to_string(),
        KEY_TO_AWS_ACCESS_KEY_ID.to_string(),
        KEY_TO_AWS_SECRET_ACCESS_KEY.to_string(),
        KEY_TO_AWS_REGION.to_string(),
        KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_USER_APP.to_string(),
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
        "user_service=debug,common=debug,tower_http=debug,sea_orm=debug",
    );
    tracing_subscriber::fmt::init();

    let database_url = var(KEY_TO_DATABASE_URL).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_DATABASE_URL
        )
    });
    let mut opt = ConnectOptions::new(database_url.clone());
    opt.max_connections(num_of_cpus)
        .min_connections(num_of_cpus)
        .sqlx_logging(true);
    let pool = Database::connect(opt)
        .await
        .expect("failed to connect database");

    let redis_url = var(KEY_TO_URL_FOR_REDIS_SERVER).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_URL_FOR_REDIS_SERVER
        )
    });
    let store = RedisSessionStore::new(redis_url).expect("failed to connect redis");

    let app = Router::new()
        .nest(
            ROOT_PATH,
            Router::new()
                .route("/temp-accounts", post(post_temp_accounts))
                .route("/accounts", post(post_accounts))
                .route("/login", post(post_login))
                .route("/logout", post(post_logout))
                .route("/refresh", get(get_refresh))
                .route("/agreement", post(post_agreement))
                .route("/password-change-req", post(post_password_change_req))
                .route("/password-update", post(post_password_update))
                .route("/profile", get(get_profile))
                .route("/rewards", get(get_reward))
                .route("/identity", post(post_identity)),
        )
        .layer(
            ServiceBuilder::new()
                //.layer(TraceLayer::new_for_http())
                .layer(
                    TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                        let x = request
                            .headers()
                            .get("X-Forwarded-For")
                            .map(|hv| match hv.to_str() {
                                Ok(s) => s.to_string(),
                                Err(e) => format!("{}", e),
                            })
                            .unwrap_or_else(|| "None".to_string());
                        let req_id = Uuid::new_v4().simple().to_string();
                        tracing::span!(
                            Level::INFO,
                            "req",
                            request_id = &tracing::field::display(req_id),
                            x_fowarded_for = &tracing::field::display(x)
                        )
                    }),
                )
                .layer(CookieManagerLayer::new())
                .layer(Extension(store))
                .layer(Extension(pool)),
        );

    let socket = var(KEY_TO_SOCKET).unwrap_or_else(|_| {
            panic!(
                "Not environment variable found: environment variable \"{}\" (example value: \"0.0.0.0:3000\") must be set",
                KEY_TO_SOCKET
            )
        });
    let addr = socket
        .parse()
        .unwrap_or_else(|_| panic!("failed to parse socket: {}", socket));
    tracing::info!("listening on {}", addr);
    let _ = axum::Server::bind(&addr)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .expect("failed to serve app");
}
