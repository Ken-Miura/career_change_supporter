// Copyright 2021 Ken Miura

use axum::{
    handler::{get, Handler},
    http::StatusCode,
    response::IntoResponse,
    service, Router,
};
use std::{convert::Infallible, io, net::SocketAddr};
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "service=debug,tower_http=debug");
    }
    tracing_subscriber::fmt::init();

    // our API routes
    let api_routes = Router::new().route("/users", get(|| async { "users#index" }));

    let app = Router::new()
        .nest(
            "/",
            Router::new()
                .route(
                    "/",
                    service::get(ServeFile::new("service/static/index.html"))
                        .handle_error(handle_io_error),
                )
                .route(
                    "/index.html",
                    service::get(ServeFile::new("service/static/index.html"))
                        .handle_error(handle_io_error),
                ),
        )
        .nest(
            "/user",
            Router::new()
                .route(
                    "/",
                    service::get(ServeDir::new("service/static/user"))
                        .handle_error(handle_io_error),
                )
                .or(
                    service::any(ServeFile::new("service/static/user/user_app.html"))
                        .handle_error(handle_io_error),
                ),
        )
        .nest(
            "/advisor",
            Router::new()
                .route(
                    "/",
                    service::get(ServeDir::new("service/static/advisor"))
                        .handle_error(handle_io_error),
                )
                .or(
                    service::any(ServeFile::new("service/static/advisor/advisor_app.html"))
                        .handle_error(handle_io_error),
                ),
        )
        // serve the API at `/api/*`
        .nest("/api", api_routes)
        .or(handler_404.into_service());

    // run
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn handle_io_error(error: io::Error) -> Result<impl IntoResponse, Infallible> {
    Ok((
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Unhandled error: {}", error),
    ))
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
}
