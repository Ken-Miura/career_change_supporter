// Copyright 2021 Ken Miura

mod err;
mod handlers;

use crate::handlers::ROOT_PATH;
use crate::handlers::session::authentication::authenticated_handlers::career_request::create_request::career_images::get_career_images;
use crate::handlers::session::authentication::authenticated_handlers::career_request::create_request::approval::post_create_career_request_approval;
use crate::handlers::session::authentication::authenticated_handlers::career_request::create_request::detail::get_create_career_request_detail;
use crate::handlers::session::authentication::authenticated_handlers::career_request::create_request::rejection::post_create_career_request_rejection;
use crate::handlers::session::authentication::authenticated_handlers::career_request::create_request::list::get_create_career_requests;
use crate::handlers::session::authentication::authenticated_handlers::identity_by_user_account_id::get_identity_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::identity_request::create_request::approval::post_create_identity_request_approval;
use crate::handlers::session::authentication::authenticated_handlers::identity_request::create_request::detail::get_create_identity_request_detail;
use crate::handlers::session::authentication::authenticated_handlers::identity_request::create_request::rejection::post_create_identity_request_rejection;
use crate::handlers::session::authentication::authenticated_handlers::identity_request::create_request::list::get_create_identity_requests;
use crate::handlers::session::authentication::authenticated_handlers::identity_request::create_request::users_by_date_of_birth::get_users_by_date_of_birth;
use crate::handlers::session::authentication::authenticated_handlers::identity_request::identity_images::get_identity_images;
use crate::handlers::session::authentication::authenticated_handlers::identity_request::update_request::approval::post_update_identity_request_approval;
use crate::handlers::session::authentication::authenticated_handlers::identity_request::update_request::detail::get_update_identity_request_detail;
use crate::handlers::session::authentication::authenticated_handlers::identity_request::update_request::rejection::post_update_identity_request_rejection;
use crate::handlers::session::authentication::authenticated_handlers::identity_request::update_request::list::get_update_identity_requests;
use crate::handlers::session::authentication::authenticated_handlers::user_account::agreements_by_user_account_id::get_agreements_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::career_creation::approval_records::get_career_creation_approval_records;
use crate::handlers::session::authentication::authenticated_handlers::user_account::career_creation::rejection_records::get_career_creation_rejection_records;
use crate::handlers::session::authentication::authenticated_handlers::user_account::careers_by_user_account_id::get_careers_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::consultation_reqs_by_consultant_id::get_consultation_reqs_by_consultant_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::consultation_reqs_by_user_account_id::get_consultation_reqs_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::consultations_by_consultant_id::get_consultations_by_consultant_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::consultations_by_user_account_id::get_consultations_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::disable_mfa_req::post_disable_mfa_req;
use crate::handlers::session::authentication::authenticated_handlers::user_account::fee_per_hour_in_yen_by_user_account_id::get_fee_per_hour_in_yen_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::identity_creation::approval_record::get_identity_creation_approval_record;
use crate::handlers::session::authentication::authenticated_handlers::user_account::identity_creation::rejection_records::get_identity_creation_rejection_records;
use crate::handlers::session::authentication::authenticated_handlers::user_account::identity_option_by_user_account_id::get_identity_option_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::identity_update::approval_records::get_identity_update_approval_records;
use crate::handlers::session::authentication::authenticated_handlers::user_account::identity_update::rejection_records::get_identity_update_rejection_records;
use crate::handlers::session::authentication::authenticated_handlers::user_account::rating_info_by_consultant_id::get_rating_info_by_consultant_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::rating_info_by_user_account_id::get_rating_info_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::tenant_id_by_user_account_id::get_tenant_id_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::user_account_retrieval_by_email_address::post_user_account_retrieval_by_email_address;
use crate::handlers::session::authentication::authenticated_handlers::user_account::user_account_retrieval_by_user_account_id::post_user_account_retrieval_by_user_account_id;
use crate::handlers::session::authentication::login::post_login;
use crate::handlers::session::authentication::logout::post_logout;
use crate::handlers::session::authentication::authenticated_handlers::refresh::get_refresh;
use crate::handlers::session::authentication::pass_code::post_pass_code;
use async_fred_session::fred::pool::RedisPool;
use async_fred_session::fred::types::RedisConfig;
use async_fred_session::RedisSessionStore;
use axum::body::Body;
use axum::http::Request;
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
use common::{AppState, RequestLogElements};
use dotenv::dotenv;
use entity::sea_orm::{ConnectOptions, Database};
use handlers::session::authentication::pass_code::KEY_TO_ADMIN_TOTP_ISSUER;
use once_cell::sync::Lazy;
use std::env::set_var;
use std::env::var;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::{Level, Span};
use uuid::Uuid;

const KEY_TO_DATABASE_URL: &str = "DB_URL_FOR_ADMIN_APP";
const KEY_TO_SOCKET: &str = "SOCKET_FOR_ADMIN_APP";
const KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_ADMIN_APP: &str = "KEY_OF_SIGNED_COOKIE_FOR_ADMIN_APP";

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
        KEY_TO_URL_FOR_REDIS_SERVER.to_string(),
        KEY_TO_PAYMENT_PLATFORM_API_URL.to_string(),
        KEY_TO_PAYMENT_PLATFORM_API_USERNAME.to_string(),
        KEY_TO_PAYMENT_PLATFORM_API_PASSWORD.to_string(),
        KEY_TO_AWS_S3_ENDPOINT_URI.to_string(),
        KEY_TO_AWS_ACCESS_KEY_ID.to_string(),
        KEY_TO_AWS_SECRET_ACCESS_KEY.to_string(),
        KEY_TO_AWS_REGION.to_string(),
        KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_ADMIN_APP.to_string(),
        KEY_TO_OPENSEARCH_ENDPOINT_URI.to_string(),
        KEY_TO_OPENSEARCH_USERNAME.to_string(),
        KEY_TO_OPENSEARCH_PASSWORD.to_string(),
        KEY_TO_ADMIN_TOTP_ISSUER.to_string(),
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
    let config = RedisConfig::from_url(redis_url.as_str()).expect("failed to create redis config");
    let redis_pool = RedisPool::new(config, None, None, num_of_cpus as usize)
        .expect("failed to create redis pool");
    let _ = redis_pool.connect();
    redis_pool
        .wait_for_connect()
        .await
        .expect("failed to connect redis");
    let store = RedisSessionStore::from_pool(redis_pool, None);

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
        create_key_for_singed_cookie(KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_ADMIN_APP);

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
                .route("/login", post(post_login))
                .route("/pass-code", post(post_pass_code))
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
                )
                .route(
                    "/career-images/:user_account_id/:image_name",
                    get(get_career_images),
                )
                .route(
                    "/create-career-request-approval",
                    post(post_create_career_request_approval),
                )
                .route(
                    "/create-career-request-rejection",
                    post(post_create_career_request_rejection),
                )
                .route(
                    "/user-account-retrieval-by-user-account-id",
                    post(post_user_account_retrieval_by_user_account_id),
                )
                .route(
                    "/user-account-retrieval-by-email-address",
                    post(post_user_account_retrieval_by_email_address),
                )
                .route(
                    "/agreements-by-user-account-id",
                    get(get_agreements_by_user_account_id),
                )
                .route(
                    "/identity-option-by-user-account-id",
                    get(get_identity_option_by_user_account_id),
                )
                .route(
                    "/careers-by-user-account-id",
                    get(get_careers_by_user_account_id),
                )
                .route(
                    "/fee-per-hour-in-yen-by-user-account-id",
                    get(get_fee_per_hour_in_yen_by_user_account_id),
                )
                .route(
                    "/tenant-id-by-user-account-id",
                    get(get_tenant_id_by_user_account_id),
                )
                .route(
                    "/consultation-reqs-by-user-account-id",
                    get(get_consultation_reqs_by_user_account_id),
                )
                .route(
                    "/consultation-reqs-by-consultant-id",
                    get(get_consultation_reqs_by_consultant_id),
                )
                .route(
                    "/consultations-by-user-account-id",
                    get(get_consultations_by_user_account_id),
                )
                .route(
                    "/consultations-by-consultant-id",
                    get(get_consultations_by_consultant_id),
                )
                .route(
                    "/rating-info-by-user-account-id",
                    get(get_rating_info_by_user_account_id),
                )
                .route(
                    "/rating-info-by-consultant-id",
                    get(get_rating_info_by_consultant_id),
                )
                .route(
                    "/identity-creation-approval-record",
                    get(get_identity_creation_approval_record),
                )
                .route(
                    "/identity-creation-rejection-records",
                    get(get_identity_creation_rejection_records),
                )
                .route(
                    "/identity-update-approval-records",
                    get(get_identity_update_approval_records),
                )
                .route(
                    "/identity-update-rejection-records",
                    get(get_identity_update_rejection_records),
                )
                .route(
                    "/career-creation-approval-records",
                    get(get_career_creation_approval_records),
                )
                .route(
                    "/career-creation-rejection-records",
                    get(get_career_creation_rejection_records),
                )
                .route(
                    "/disable-mfa-req",
                    post(post_disable_mfa_req),
                )
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
                "Not environment variable found: environment variable \"{}\" (example value: \"0.0.0.0:3001\") must be set",
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
