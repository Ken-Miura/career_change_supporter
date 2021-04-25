// Copyright 2021 Ken Miura

use crate::common;
use actix_session::Session;
use actix_web::{get, http::StatusCode, web, HttpResponse};

#[get("/profile-information")]
async fn profile_information(
    _session: Session,
    _pool: web::Data<common::ConnectionPool>,
) -> HttpResponse {
    // TODO: Handle Result
    HttpResponse::build(StatusCode::OK).finish()
}
