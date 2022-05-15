// Copyright 2021 Ken Miura

mod create_career_request;
mod create_identity_request;
mod err;
pub(crate) mod identity_by_user_account_id;
mod identity_images;
mod login;
mod logout;
mod refresh;
mod update_identity_request;
mod util;

use crate::create_career_request::create_career_request_detail::get_create_career_request_detail;
use crate::create_career_request::create_career_requests::get_create_career_requests;
use crate::create_identity_request::create_identity_request_approval::post_create_identity_request_approval;
use crate::create_identity_request::create_identity_request_detail::get_create_identity_request_detail;
use crate::create_identity_request::create_identity_request_rejection::post_create_identity_request_rejection;
use crate::create_identity_request::create_identity_requests::get_create_identity_requests;
use crate::create_identity_request::users_by_date_of_birth::get_users_by_date_of_birth;
use crate::identity_by_user_account_id::get_identity_by_user_account_id;
use crate::identity_images::get_identity_images;
use crate::login::post_login;
use crate::logout::post_logout;
use crate::refresh::get_refresh;
use crate::update_identity_request::update_identity_request_approval::post_update_identity_request_approval;
use crate::update_identity_request::update_identity_request_detail::get_update_identity_request_detail;
use crate::update_identity_request::update_identity_request_rejection::post_update_identity_request_rejection;
use crate::update_identity_request::update_identity_requests::get_update_identity_requests;
use crate::util::session::KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_ADMIN_APP;
use crate::util::ROOT_PATH;
use async_redis_session::RedisSessionStore;
use axum::body::Body;
use axum::extract::Extension;
use axum::http::Request;
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
use dotenv::dotenv;
use entity::sea_orm::{ConnectOptions, Database};
use once_cell::sync::Lazy;
use std::env::set_var;
use std::env::var;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::{Level, Span};
use uuid::Uuid;

const KEY_TO_DATABASE_URL: &str = "DB_URL_FOR_ADMIN_APP";
const KEY_TO_SOCKET: &str = "SOCKET_FOR_ADMIN_APP";

/// アプリケーションの動作に必須の環境変数をすべて列挙し、
/// 起動直後に存在をチェックする
static ENV_VARS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        KEY_TO_DATABASE_URL.to_string(),
        KEY_TO_SOCKET.to_string(),
        KEY_TO_SOCKET_FOR_SMTP_SERVER.to_string(),
        KEY_TO_URL_FOR_REDIS_SERVER.to_string(),
        KEY_TO_PAYMENT_PLATFORM_API_URL.to_string(),
        KEY_TO_PAYMENT_PLATFORM_API_USERNAME.to_string(),
        KEY_TO_PAYMENT_PLATFORM_API_PASSWORD.to_string(),
        KEY_TO_AWS_S3_ENDPOINT_URI.to_string(),
        KEY_TO_AWS_ACCESS_KEY_ID.to_string(),
        KEY_TO_AWS_SECRET_ACCESS_KEY.to_string(),
        KEY_TO_AWS_REGION.to_string(),
        KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_ADMIN_APP.to_string(),
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
        "admin_service=debug,common=debug,tower_http=debug,sea_orm=debug",
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
                .route("/login", post(post_login))
                .route("/logout", post(post_logout))
                .route("/refresh", get(get_refresh))
                .route(
                    "/create-identity-requests",
                    get(get_create_identity_requests),
                )
                .route(
                    "/create-identity-request-detail",
                    get(get_create_identity_request_detail),
                )
                .route("/users-by-date-of-birth", get(get_users_by_date_of_birth))
                .route(
                    "/identity-images/:user_account_id/:image_name",
                    get(get_identity_images),
                )
                .route(
                    "/create-identity-request-approval",
                    post(post_create_identity_request_approval),
                )
                .route(
                    "/create-identity-request-rejection",
                    post(post_create_identity_request_rejection),
                )
                .route(
                    "/update-identity-requests",
                    get(get_update_identity_requests),
                )
                .route(
                    "/update-identity-request-detail",
                    get(get_update_identity_request_detail),
                )
                .route(
                    "/identity-by-user-account-id",
                    get(get_identity_by_user_account_id),
                )
                .route(
                    "/update-identity-request-approval",
                    post(post_update_identity_request_approval),
                )
                .route(
                    "/update-identity-request-rejection",
                    post(post_update_identity_request_rejection),
                )                .route(
                    "/create-career-requests",
                    get(get_create_career_requests),
                )
                .route(
                    "/create-career-request-detail",
                    get(get_create_career_request_detail),
                ),
        )
        .layer(
            ServiceBuilder::new()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|_request: &Request<Body>| {
                        let req_id = Uuid::new_v4().simple().to_string();
                        tracing::span!(
                            Level::INFO,
                            "req",
                            id = &tracing::field::display(req_id),
                        )
                    })
                    .on_request(|request: &Request<Body>, _span: &Span| {
                        let req_log = RequestLog::new(request);
                        tracing::info!(
                            "started processing request (method={}, uri={}, version={:?}, headers={{x-forwarded-for: {}, x-real-ip: {}, forwarded: {}, user-agent: {}}})",
                            req_log.method,
                            req_log.uri,
                            req_log.version,
                            req_log.x_forwarded_for,
                            req_log.x_real_ip,
                            req_log.forwarded,
                            req_log.user_agent
                        );
                    })
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    ),
            )
                .layer(CookieManagerLayer::new())
                .layer(Extension(store))
                .layer(Extension(pool)),
        );

    let socket = var(KEY_TO_SOCKET).unwrap_or_else(|_| {
            panic!(
                "Not environment variable found: environment variable \"{}\" (example value: \"0.0.0.0:3001\") must be set",
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

struct RequestLog {
    method: String,
    uri: String,
    version: String,
    x_forwarded_for: String,
    x_real_ip: String,
    forwarded: String,
    user_agent: String,
}

impl RequestLog {
    fn new(request: &Request<Body>) -> Self {
        let method = request.method();
        let uri = request.uri();
        let version = request.version();
        let headers = request.headers();
        let x_forwarded_for = headers
            .get("x-forwarded-for")
            .map(|hv| match hv.to_str() {
                Ok(s) => s.to_string(),
                Err(e) => format!("{}", e),
            })
            .unwrap_or_else(|| "None".to_string());
        let x_real_ip = headers
            .get("x-real-ip")
            .map(|hv| match hv.to_str() {
                Ok(s) => s.to_string(),
                Err(e) => format!("{}", e),
            })
            .unwrap_or_else(|| "None".to_string());
        let forwarded = headers
            .get("forwarded")
            .map(|hv| match hv.to_str() {
                Ok(s) => s.to_string(),
                Err(e) => format!("{}", e),
            })
            .unwrap_or_else(|| "None".to_string());
        let user_agent = request
            .headers()
            .get("user-agent")
            .map(|hv| match hv.to_str() {
                Ok(s) => s.to_string(),
                Err(e) => format!("{}", e),
            })
            .unwrap_or_else(|| "None".to_string());
        RequestLog {
            method: format!("{}", method),
            uri: format!("{}", uri),
            version: format!("{:?}", version),
            x_forwarded_for,
            x_real_ip,
            forwarded,
            user_agent,
        }
    }
}
