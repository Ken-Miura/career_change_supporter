// Copyright 2021 Ken Miura

mod err;
mod handlers;

use crate::handlers::ROOT_PATH;
use crate::handlers::health::get_health;
use crate::handlers::session::authentication::authenticated_handlers::awaiting_payment::awaiting_payment_by_consultation_id::get_awaiting_payment_by_consultation_id;
use crate::handlers::session::authentication::authenticated_handlers::awaiting_withdrawal::awaiting_withdrawal_by_consultation_id::get_awaiting_withdrawal_by_consultation_id;
use crate::handlers::session::authentication::authenticated_handlers::left_awaiting_withdrawal::list::get_left_awaiting_withdrawals;
use crate::handlers::session::authentication::authenticated_handlers::left_awaiting_withdrawal::post::post_left_awaiting_withdrawal;
use crate::handlers::session::authentication::authenticated_handlers::receipt_of_consultation::list::get_receipts_of_consultation;
use crate::handlers::session::authentication::authenticated_handlers::receipt_of_consultation::post::post_receipt_of_consultation;
use crate::handlers::session::authentication::authenticated_handlers::receipt_of_consultation::receipt_of_consultation_by_consultation_id::get_receipt_of_consultation_by_consultation_id;
use crate::handlers::session::authentication::authenticated_handlers::refunded_payment::refund_from_awaiting_withdrawal::post_refund_from_awaiting_withdrawal;
use crate::handlers::session::authentication::authenticated_handlers::refunded_payment::refunded_payment_by_consultation_id::get_refunded_payment_by_consultation_id;
use crate::handlers::session::authentication::authenticated_handlers::{KEY_TO_TRANSFER_FEE_IN_YEN, KEY_TO_PLATFORM_FEE_RATE_IN_PERCENTAGE};
use crate::handlers::session::authentication::authenticated_handlers::awaiting_payment::expired_list::get_expired_awaiting_payments;
use crate::handlers::session::authentication::authenticated_handlers::awaiting_withdrawal::list::get_awaiting_withdrawals;
use crate::handlers::session::authentication::authenticated_handlers::awaiting_withdrawal::post::post_awaiting_withdrawal;
use crate::handlers::session::authentication::authenticated_handlers::career_request::create_request::career_images::get_career_images;
use crate::handlers::session::authentication::authenticated_handlers::career_request::create_request::approval::post_create_career_request_approval;
use crate::handlers::session::authentication::authenticated_handlers::career_request::create_request::detail::get_create_career_request_detail;
use crate::handlers::session::authentication::authenticated_handlers::career_request::create_request::rejection::post_create_career_request_rejection;
use crate::handlers::session::authentication::authenticated_handlers::career_request::create_request::list::get_create_career_requests;
use crate::handlers::session::authentication::authenticated_handlers::consultation::consultant_rating_by_consultation_id::get_consultant_rating_by_consultation_id;
use crate::handlers::session::authentication::authenticated_handlers::consultation::consultation_by_consultation_id::get_consultation_by_consultation_id;
use crate::handlers::session::authentication::authenticated_handlers::consultation::user_rating_by_consultation_id::get_user_rating_by_consultation_id;
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
use crate::handlers::session::authentication::authenticated_handlers::maintenance::planned_maintenances::get_planned_maintenances;
use crate::handlers::session::authentication::authenticated_handlers::maintenance::set_maintenance_req::post_set_maintenance_req;
use crate::handlers::session::authentication::authenticated_handlers::neglected_payment::list::get_neglected_payments;
use crate::handlers::session::authentication::authenticated_handlers::neglected_payment::post::post_neglected_payment;
use crate::handlers::session::authentication::authenticated_handlers::news::delete_news_req::post_delete_news_req;
use crate::handlers::session::authentication::authenticated_handlers::news::latest_news::get_latest_news;
use crate::handlers::session::authentication::authenticated_handlers::news::set_news_req::post_set_news_req;
use crate::handlers::session::authentication::authenticated_handlers::refunded_payment::list::get_refunded_payments;
use crate::handlers::session::authentication::authenticated_handlers::refunded_payment::refund_from_awaiting_payment::post_refund_from_awaiting_payment;
use crate::handlers::session::authentication::authenticated_handlers::user_account::agreements_by_user_account_id::get_agreements_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::career_creation::approval_records::get_career_creation_approval_records;
use crate::handlers::session::authentication::authenticated_handlers::user_account::career_creation::rejection_records::get_career_creation_rejection_records;
use crate::handlers::session::authentication::authenticated_handlers::user_account::careers_by_user_account_id::get_careers_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::consultation_reqs_by_consultant_id::get_consultation_reqs_by_consultant_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::consultation_reqs_by_user_account_id::get_consultation_reqs_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::consultations_by_consultant_id::get_consultations_by_consultant_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::consultations_by_user_account_id::get_consultations_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::disable_mfa_req::post_disable_mfa_req;
use crate::handlers::session::authentication::authenticated_handlers::user_account::disable_user_account_req::post_disable_user_account_req;
use crate::handlers::session::authentication::authenticated_handlers::user_account::enable_user_account_req::post_enable_user_account_req;
use crate::handlers::session::authentication::authenticated_handlers::user_account::fee_per_hour_in_yen_by_user_account_id::get_fee_per_hour_in_yen_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::identity_creation::approval_record::get_identity_creation_approval_record;
use crate::handlers::session::authentication::authenticated_handlers::user_account::identity_creation::rejection_records::get_identity_creation_rejection_records;
use crate::handlers::session::authentication::authenticated_handlers::user_account::identity_option_by_user_account_id::get_identity_option_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::identity_update::approval_records::get_identity_update_approval_records;
use crate::handlers::session::authentication::authenticated_handlers::user_account::identity_update::rejection_records::get_identity_update_rejection_records;
use crate::handlers::session::authentication::authenticated_handlers::user_account::rating_info_by_consultant_id::get_rating_info_by_consultant_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::rating_info_by_user_account_id::get_rating_info_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::bank_account_by_user_account_id::get_bank_account_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::user_account::user_account_retrieval_by_email_address::post_user_account_retrieval_by_email_address;
use crate::handlers::session::authentication::authenticated_handlers::user_account::user_account_retrieval_by_user_account_id::post_user_account_retrieval_by_user_account_id;
use crate::handlers::session::authentication::authenticated_handlers::awaiting_payment::list::get_awaiting_payments;
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
use common::admin::{KEY_TO_DB_ADMIN_NAME, KEY_TO_DB_ADMIN_PASSWORD, KEY_TO_ADMIN_TOTP_ISSUER};
use common::db::{KEY_TO_DB_HOST, KEY_TO_DB_PORT, KEY_TO_DB_NAME, construct_db_url};
use common::log::{LOG_LEVEL, init_log};
use common::opensearch::{
    create_client, KEY_TO_OPENSEARCH_ENDPOINT_URI, KEY_TO_OPENSEARCH_PASSWORD,
    KEY_TO_OPENSEARCH_USERNAME, KEY_TO_OPENSEARCH_AUTH,
};
use common::redis::{KEY_TO_REDIS_HOST, KEY_TO_REDIS_PORT, construct_redis_url};
use common::smtp::{
    KEY_TO_SYSTEM_EMAIL_ADDRESS, KEY_TO_INQUIRY_EMAIL_ADDRESS, KEY_TO_AWS_SES_REGION, KEY_TO_AWS_SES_ENDPOINT_URI, SmtpClient, AWS_SES_REGION, AWS_SES_ACCESS_KEY_ID, AWS_SES_SECRET_ACCESS_KEY, AWS_SES_ENDPOINT_URI,
};
use common::storage::{
     KEY_TO_AWS_S3_REGION, KEY_TO_AWS_S3_ENDPOINT_URI,
     KEY_TO_IDENTITY_IMAGES_BUCKET_NAME, KEY_TO_CAREER_IMAGES_BUCKET_NAME, AWS_S3_REGION, AWS_S3_ACCESS_KEY_ID, AWS_S3_SECRET_ACCESS_KEY, AWS_S3_ENDPOINT_URI, StorageClient,
};
use common::util::{check_env_vars};
use common::{AppState, RequestLogElements, create_key_for_singed_cookie, KEY_TO_USE_ECS_TASK_ROLE, USE_ECS_TASK_ROLE};
use dotenv::dotenv;
use entity::sea_orm::{ConnectOptions, Database};
use once_cell::sync::Lazy;
use std::env::set_var;
use std::env::var;
use tower::ServiceBuilder;
use tower_http::trace::{DefaultOnResponse, TraceLayer};
use tower_http::LatencyUnit;
use tracing::{Level, Span};
use uuid::Uuid;

const KEY_TO_SOCKET: &str = "SOCKET_FOR_ADMIN_APP";
const KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_ADMIN_APP: &str = "KEY_OF_SIGNED_COOKIE_FOR_ADMIN_APP";

/// アプリケーションの動作に必須の環境変数をすべて列挙し、
/// 起動直後に存在をチェックする
static ENV_VARS: Lazy<Vec<String>> = Lazy::new(|| {
    vec![
        KEY_TO_DB_HOST.to_string(),
        KEY_TO_DB_PORT.to_string(),
        KEY_TO_DB_NAME.to_string(),
        KEY_TO_DB_ADMIN_NAME.to_string(),
        KEY_TO_DB_ADMIN_PASSWORD.to_string(),
        KEY_TO_SOCKET.to_string(),
        KEY_TO_REDIS_HOST.to_string(),
        KEY_TO_REDIS_PORT.to_string(),
        KEY_TO_AWS_S3_REGION.to_string(),
        KEY_TO_AWS_S3_ENDPOINT_URI.to_string(),
        KEY_TO_IDENTITY_IMAGES_BUCKET_NAME.to_string(),
        KEY_TO_CAREER_IMAGES_BUCKET_NAME.to_string(),
        KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_ADMIN_APP.to_string(),
        KEY_TO_OPENSEARCH_ENDPOINT_URI.to_string(),
        KEY_TO_OPENSEARCH_AUTH.to_string(),
        KEY_TO_OPENSEARCH_USERNAME.to_string(),
        KEY_TO_OPENSEARCH_PASSWORD.to_string(),
        KEY_TO_ADMIN_TOTP_ISSUER.to_string(),
        KEY_TO_SYSTEM_EMAIL_ADDRESS.to_string(),
        KEY_TO_INQUIRY_EMAIL_ADDRESS.to_string(),
        KEY_TO_AWS_SES_REGION.to_string(),
        KEY_TO_AWS_SES_ENDPOINT_URI.to_string(),
        KEY_TO_USE_ECS_TASK_ROLE.to_string(),
        KEY_TO_TRANSFER_FEE_IN_YEN.to_string(),
        KEY_TO_PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
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
    let log_conf = format!(
        "admin_service={},common={},tower_http={},sea_orm={}",
        LOG_LEVEL.as_str(),
        LOG_LEVEL.as_str(),
        LOG_LEVEL.as_str(),
        LOG_LEVEL.as_str()
    );
    set_var("RUST_LOG", log_conf);
    init_log();

    let database_url = construct_db_url(
        KEY_TO_DB_HOST,
        KEY_TO_DB_PORT,
        KEY_TO_DB_NAME,
        KEY_TO_DB_ADMIN_NAME,
        KEY_TO_DB_ADMIN_PASSWORD,
    );
    let mut opt = ConnectOptions::new(database_url.clone());
    opt.max_connections(num_of_cpus)
        .min_connections(num_of_cpus)
        .sqlx_logging(true);
    let pool = Database::connect(opt)
        .await
        .expect("failed to connect database");

    let redis_url = construct_redis_url(KEY_TO_REDIS_HOST, KEY_TO_REDIS_PORT);
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
    let opensearch_auth = var(KEY_TO_OPENSEARCH_AUTH)
        .unwrap_or_else(|_| {
            panic!(
                "Not environment variable found: environment variable \"{}\" must be set",
                KEY_TO_OPENSEARCH_AUTH
            )
        })
        .parse::<bool>()
        .unwrap_or_else(|_| panic!("failed to parse {}", KEY_TO_OPENSEARCH_AUTH));
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
        opensearch_auth,
        opensearch_username.as_str(),
        opensearch_password.as_str(),
    )
    .expect("failed to create OpenSearch client");

    let key_for_signed_cookie =
        create_key_for_singed_cookie(KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_ADMIN_APP);

    let smtp_client = if *USE_ECS_TASK_ROLE {
        SmtpClient::new_with_ecs_task_role(AWS_SES_REGION.as_str(), AWS_SES_ENDPOINT_URI.as_str())
            .await
    } else {
        SmtpClient::new(
            AWS_SES_REGION.as_str(),
            AWS_SES_ACCESS_KEY_ID.as_str(),
            AWS_SES_SECRET_ACCESS_KEY.as_str(),
            AWS_SES_ENDPOINT_URI.as_str(),
        )
        .await
    };

    let storage_client = if *USE_ECS_TASK_ROLE {
        StorageClient::new_with_ecs_task_role(AWS_S3_REGION.as_str(), AWS_S3_ENDPOINT_URI.as_str())
            .await
    } else {
        StorageClient::new(
            AWS_S3_REGION.as_str(),
            AWS_S3_ACCESS_KEY_ID.as_str(),
            AWS_S3_SECRET_ACCESS_KEY.as_str(),
            AWS_S3_ENDPOINT_URI.as_str(),
        )
        .await
    };

    let state = AppState {
        store,
        index_client,
        pool,
        key_for_signed_cookie,
        smtp_client,
        storage_client,
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
                    "/bank-account-by-user-account-id",
                    get(get_bank_account_by_user_account_id),
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
                    "/disable-user-account-req",
                    post(post_disable_user_account_req),
                )
                .route(
                    "/enable-user-account-req",
                    post(post_enable_user_account_req),
                )
                .route(
                    "/disable-mfa-req",
                    post(post_disable_mfa_req),
                )
                .route(
                    "/consultation-by-consultation-id",
                    get(get_consultation_by_consultation_id),
                )
                .route(
                    "/user-rating-by-consultation-id",
                    get(get_user_rating_by_consultation_id),
                )
                .route(
                    "/consultant-rating-by-consultation-id",
                    get(get_consultant_rating_by_consultation_id),
                )
                .route(
                    "/receipt-of-consultation-by-consultation-id",
                    get(get_receipt_of_consultation_by_consultation_id),
                )
                .route(
                    "/refunded-payment-by-consultation-id",
                    get(get_refunded_payment_by_consultation_id),
                )
                .route(
                    "/planned-maintenances",
                    get(get_planned_maintenances),
                )
                .route(
                    "/set-maintenance-req",
                    post(post_set_maintenance_req),
                )
                .route(
                    "/latest-news",
                    get(get_latest_news),
                )
                .route(
                    "/set-news-req",
                    post(post_set_news_req),
                )
                .route(
                    "/delete-news-req",
                    post(post_delete_news_req),
                )
                .route(
                    "/health",
                    get(get_health),
                )
                .route(
                    "/awaiting-payments",
                    get(get_awaiting_payments),
                )
                .route(
                    "/expired-awaiting-payments",
                    get(get_expired_awaiting_payments),
                )
                .route(
                    "/awaiting-withdrawal",
                    post(post_awaiting_withdrawal),
                )
                .route(
                    "/awaiting-withdrawals",
                    get(get_awaiting_withdrawals),
                )
                .route(
                    "/refund-from-awaiting-payment",
                    post(post_refund_from_awaiting_payment),
                )
                .route(
                    "/refund-from-awaiting-withdrawal",
                    post(post_refund_from_awaiting_withdrawal),
                )
                .route(
                    "/refunded-payments",
                    get(get_refunded_payments),
                )
                .route(
                    "/neglected-payment",
                    post(post_neglected_payment),
                )
                .route(
                    "/neglected-payments",
                    get(get_neglected_payments),
                )
                .route(
                    "/receipt-of-consultation",
                    post(post_receipt_of_consultation),
                )
                .route(
                    "/receipts-of-consultation",
                    get(get_receipts_of_consultation),
                )
                .route(
                    "/left-awaiting-withdrawal",
                    post(post_left_awaiting_withdrawal),
                )
                .route(
                    "/left-awaiting-withdrawals",
                    get(get_left_awaiting_withdrawals),
                )
                .route(
                    "/awaiting-payment-by-consultation-id",
                    get(get_awaiting_payment_by_consultation_id),
                )
                .route(
                    "/awaiting-withdrawal-by-consultation-id",
                    get(get_awaiting_withdrawal_by_consultation_id),
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
