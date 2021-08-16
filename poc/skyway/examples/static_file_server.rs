//! Run with
//!
//! ```not_rust
//! cargo run --example static_file_server
//! ```

use axum::{prelude::*, response::Json,routing::nest};
use http::StatusCode;
use std::net::SocketAddr;
use tower_http::{services::ServeDir, trace::TraceLayer};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::add_extension::AddExtensionLayer;

#[tokio::main]
async fn main() {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "static_file_server=debug,tower_http=debug")
    }
    tracing_subscriber::fmt::init();

    let state = Arc::new(Mutex::new(State{ peer_id: None }));
    let app = nest(
        "/static",
        axum::service::get(ServeDir::new(".")).handle_error(|error: std::io::Error| {
            Ok::<_, std::convert::Infallible>((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Unhandled internal error: {}", error),
            ))
        }),
    )
    .route("/peer-id", post(handle_peer_id))
    .layer(AddExtensionLayer::new(state))
    .layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Serialize, Deserialize)]
struct PeerId {
    peer_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct State {
    peer_id: Option<String>,
}

type SharedState = Arc<Mutex<State>>;


async fn handle_peer_id(extract::Json(peer): extract::Json<PeerId>, extract::Extension(state): extract::Extension<SharedState>) -> Json<PeerId> {
    let mut state = state.lock().await;
    let s = &state.peer_id;
    match s {
        Some(remote_peer_id) => {
            return Json(PeerId { peer_id: Some(remote_peer_id.clone()) });
        },
        None => {
            let id = peer.peer_id.expect("failed to get peer id");
            state.peer_id = Some(id);
            return Json(PeerId { peer_id: None });
        }
    }
}