// Copyright 2021 Ken Miura

use axum::{handler::get, AddExtensionLayer, Router};
use diesel::{r2d2::ConnectionManager, r2d2::Pool, PgConnection};
use dotenv::dotenv;
use std::env::set_var;
use std::env::var;
use std::{net::SocketAddr};

const KEY_TO_DATABASE_URL: &str = "DB_URL_FOR_ADMIN_ACCOUNT_APP";

fn main() {
    let num = num_cpus::get();
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num)
        .enable_all()
        .build()
        .expect("failed to build Runtime")
        .block_on(main_internal(num as u32))
}

async fn main_internal(num_of_cpus: u32) {
    let _ = dotenv().ok();
    set_var("RUST_LOG", "user_service=debug,tower_http=debug");
    tracing_subscriber::fmt::init();

    let database_url = var(KEY_TO_DATABASE_URL).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_DATABASE_URL
        )
    });
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool: Pool<ConnectionManager<PgConnection>> = Pool::builder()
        .max_size(num_of_cpus)
        .build(manager)
        .expect("failed to build connection pool");

    let app = Router::new()
        .nest(
            "/api/users",
            Router::new().route("/hello", get(handler)),
        )
        .layer(AddExtensionLayer::new(pool));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    let _ = axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .expect("failed to serve app");
}

async fn handler() -> &'static str {
    "<h1>Hello, World!</h1>"
}