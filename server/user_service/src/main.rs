// Copyright 2021 Ken Miura

mod account;
mod agreement;
mod consultant;
mod consultation_request;
mod consultation_room;
mod consultations;
mod err;
mod login;
mod logout;
mod mfa;
mod news;
mod password;
mod personal_info;
mod rating;
mod refresh;
mod request_consultation;
mod util;

use crate::account::accounts::post_accounts;
use crate::account::temp_accounts::post_temp_accounts;
use crate::agreement::post_agreement;
use crate::consultant::detail::get_consultant_detail;
use crate::consultant::search::post_consultants_search;
use crate::consultation_request::acceptance::post_consultation_request_acceptance;
use crate::consultation_request::detail::get_consultation_request_detail;
use crate::consultation_request::list::get_consultation_requests;
use crate::consultation_request::rejection::post_consultation_request_rejection;
use crate::consultation_room::consultant_side_info::get_consultant_side_info;
use crate::consultation_room::user_side_info::get_user_side_info;
use crate::consultations::get_consultations;
use crate::login::post_login;
use crate::logout::post_logout;
use crate::mfa::setting_change::enable_mfa_req::post_enable_mfa_req;
use crate::mfa::temp_secret::get::get_temp_mfa_secret;
use crate::mfa::temp_secret::post::post_temp_mfa_secret;
use crate::news::get_news;
use crate::password::change_req::post_password_change_req;
use crate::password::update::post_password_update;
use crate::personal_info::profile::career::post::MAX_CAREER_IMAGE_SIZE_IN_BYTES;
use crate::personal_info::profile::career::{delete, get, post};
use crate::personal_info::profile::fee_per_hour_in_yen::post_fee_per_hour_in_yen;
use crate::personal_info::profile::get_profile;
use crate::personal_info::profile::identity::{post_identity, MAX_IDENTITY_IMAGE_SIZE_IN_BYTES};
use crate::personal_info::rewards::bank_account::post_bank_account;
use crate::personal_info::rewards::get_reward;
use crate::rating::{
    consultant_rating::post_consultant_rating, unrated_items::get_unrated_items,
    user_rating::post_user_rating,
};
use crate::refresh::get_refresh;
use crate::request_consultation::begin::post_begin_request_consultation;
use crate::request_consultation::fee_per_hour_in_yen_for_application::get_fee_per_hour_in_yen_for_application;
use crate::request_consultation::finish::post_finish_request_consultation;
use crate::util::terms_of_use::KEY_TO_TERMS_OF_USE_VERSION;
use crate::util::ROOT_PATH;
use async_redis_session::RedisSessionStore;
use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;
use axum_extra::extract::cookie::Key;
use common::opensearch::{
    create_client, KEY_TO_OPENSEARCH_ENDPOINT_URI, KEY_TO_OPENSEARCH_PASSWORD,
    KEY_TO_OPENSEARCH_USERNAME,
};
use common::payment_platform::{
    KEY_TO_PAYMENT_PLATFORM_API_PASSWORD, KEY_TO_PAYMENT_PLATFORM_API_URL,
    KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
};
use common::redis::KEY_TO_URL_FOR_REDIS_SERVER;
use common::smtp::{
    KEY_TO_SMTP_HOST, KEY_TO_SMTP_PASSWORD, KEY_TO_SMTP_PORT, KEY_TO_SMTP_USERNAME,
};
use common::storage::{
    KEY_TO_AWS_ACCESS_KEY_ID, KEY_TO_AWS_REGION, KEY_TO_AWS_S3_ENDPOINT_URI,
    KEY_TO_AWS_SECRET_ACCESS_KEY,
};
use common::util::check_env_vars;
use common::{AppState, RequestLogElements, KEY_TO_URL_FOR_FRONT_END};
use consultation_room::{KEY_TO_SKY_WAY_APPLICATION_ID, KEY_TO_SKY_WAY_SECRET_KEY};
use dotenv::dotenv;
use entity::sea_orm::{ConnectOptions, Database};
use hyper::{Body, Request};
use mfa::KEY_TO_TOTP_ISSUER;
use once_cell::sync::Lazy;
use std::env::set_var;
use std::env::var;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::{Level, Span};
use uuid::Uuid;

const KEY_TO_DATABASE_URL: &str = "DB_URL_FOR_USER_APP";
const KEY_TO_SOCKET: &str = "SOCKET_FOR_USER_APP";
const KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_USER_APP: &str = "KEY_OF_SIGNED_COOKIE_FOR_USER_APP";

/// アプリケーションの動作に必須の環境変数をすべて列挙し、
/// 起動直後に存在をチェックする
static ENV_VARS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        KEY_TO_DATABASE_URL.to_string(),
        KEY_TO_SOCKET.to_string(),
        KEY_TO_SMTP_HOST.to_string(),
        KEY_TO_SMTP_PORT.to_string(),
        KEY_TO_SMTP_USERNAME.to_string(),
        KEY_TO_SMTP_PASSWORD.to_string(),
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
        KEY_TO_OPENSEARCH_ENDPOINT_URI.to_string(),
        KEY_TO_OPENSEARCH_USERNAME.to_string(),
        KEY_TO_OPENSEARCH_PASSWORD.to_string(),
        KEY_TO_SKY_WAY_APPLICATION_ID.to_string(),
        KEY_TO_SKY_WAY_SECRET_KEY.to_string(),
        KEY_TO_TOTP_ISSUER.to_string(),
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

    let opensearch_url = var(KEY_TO_OPENSEARCH_ENDPOINT_URI).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_OPENSEARCH_ENDPOINT_URI
        )
    });
    let opensearch_username = var(KEY_TO_OPENSEARCH_USERNAME).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_OPENSEARCH_USERNAME
        )
    });
    let opensearch_password = var(KEY_TO_OPENSEARCH_PASSWORD).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_OPENSEARCH_PASSWORD
        )
    });
    let index_client = create_client(
        opensearch_url.as_str(),
        opensearch_username.as_str(),
        opensearch_password.as_str(),
    )
    .expect("failed to create OpenSearch client");

    let key_for_signed_cookie =
        create_key_for_singed_cookie(KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_USER_APP);

    let state = AppState {
        store,
        index_client,
        pool,
        key_for_signed_cookie,
    };

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
                .merge(Router::new().route("/identity", post(post_identity).layer(DefaultBodyLimit::max(MAX_IDENTITY_IMAGE_SIZE_IN_BYTES * 2 + 1024 * 1024))))
                .merge(Router::new().route("/career", post(post::career).get(get::career).delete(delete::career)).layer(DefaultBodyLimit::max(MAX_CAREER_IMAGE_SIZE_IN_BYTES * 2 + 1024 * 1024)))
                .route("/fee-per-hour-in-yen", post(post_fee_per_hour_in_yen))
                .route("/bank-account", post(post_bank_account))
                .route("/consultants-search", post(post_consultants_search))
                .route("/consultant-detail", get(get_consultant_detail))
                .route("/fee-per-hour-in-yen-for-application", get(get_fee_per_hour_in_yen_for_application))
                .route("/begin-request-consultation", post(post_begin_request_consultation))
                .route("/finish-request-consultation", post(post_finish_request_consultation))
                .route("/consultation-requests", get(get_consultation_requests))
                .route("/consultation-request-detail", get(get_consultation_request_detail))
                .route("/consultation-request-rejection", post(post_consultation_request_rejection))
                .route("/consultation-request-acceptance", post(post_consultation_request_acceptance))
                .route("/consultations", get(get_consultations))
                .route("/user-side-info", get(get_user_side_info))
                .route("/consultant-side-info", get(get_consultant_side_info))
                .route("/unrated-items", get(get_unrated_items))
                .route("/consultant-rating", post(post_consultant_rating))
                .route("/user-rating", post(post_user_rating))
                .route("/news", get(get_news))
                .route("/temp-mfa-secret", post(post_temp_mfa_secret).get(get_temp_mfa_secret))
                .route("/enable-mfa-req", post(post_enable_mfa_req))
                .with_state(state),
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
                            let req = RequestLogElements::new(request);
                            tracing::info!(
                                "started processing request (method={}, uri={}, version={}, headers={{x-forwarded-for: {}, x-real-ip: {}, forwarded: {}, user-agent: {}}})",
                                req.method(),
                                req.uri(),
                                req.version(),
                                req.x_forwarded_for(),
                                req.x_real_ip(),
                                req.forwarded(),
                                req.user_agent()
                            );
                        })
                        .on_response(
                            DefaultOnResponse::new()
                                .level(Level::INFO)
                                .latency_unit(LatencyUnit::Micros),
                        ),
                )
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
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to serve app");
}

fn create_key_for_singed_cookie(env_var_key: &str) -> Key {
    let key_str = var(env_var_key).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            env_var_key
        )
    });
    let size = key_str.len();
    if size < 64 {
        panic!(
            "Size of \"{}\" value regarded as utf-8 encoding must be at least 64 bytes",
            env_var_key
        )
    };
    Key::from(key_str.as_bytes())
}
