// Copyright 2021 Ken Miura

use std::net::SocketAddr;

use axum::route;
use axum::routing::RoutingDsl;
use tower::{service_fn, BoxError, Service};
use tower_http::services::{ServeDir, ServeFile};

use http_body::combinators::BoxBody;

#[tokio::main]
async fn main() {
    let get_index = axum::service::get(service_fn(|req| async {
        let resp = ServeFile::new("service/static/index.html")
            .call(req)
            .await?;
        Ok::<_, BoxError>(resp)
    }));
    let get_user_app = axum::service::get(service_fn(|req| async {
        let resp = ServeDir::new("service/static/user").call(req).await?;
        let resp = resp.map(|body| BoxBody::new(body));
        if resp.status() == 404 {
            let resp = ServeFile::new("service/static/user/user_app.html")
                .call(())
                .await?;
            let resp = resp.map(|body| BoxBody::new(body));
            return Ok(resp);
        }
        Ok::<_, BoxError>(resp)
    }));
    let get_advisor_app = axum::service::get(service_fn(|req| async {
        let resp = ServeDir::new("service/static/advisor").call(req).await?;
        let resp = resp.map(|body| BoxBody::new(body));
        if resp.status() == 404 {
            let resp = ServeFile::new("service/static/advisor/advisor_app.html")
                .call(())
                .await?;
            let resp = resp.map(|body| BoxBody::new(body));
            return Ok(resp);
        }
        Ok::<_, BoxError>(resp)
    }));
    let app = route("/", get_index.clone())
        .route("/index.html", get_index)
        .nest("/user", get_user_app)
        .nest("/advisor", get_advisor_app);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
