// Copyright 2021 Ken Miura

use actix_http::http;
use actix_web::{http::header, HttpRequest, HttpResponse, Result};
use std::fs;
use std::path::PathBuf;

pub(super) const ASSETS_DIR: &str = "static";

pub(super) fn serve_index(req: HttpRequest) -> HttpResponse {
    log::info!("fn serve_index: requested path: {}", req.uri());
    let file_path_str = format!("{}/index.html", ASSETS_DIR);
    let parse_result: Result<PathBuf, _> = file_path_str.parse();
    if let Err(e) = parse_result {
        log::error!("failed to parse path ({}): {}", file_path_str, e);
        return HttpResponse::InternalServerError().body("500 Internal Server Error");
    }
    let path = parse_result.expect("never happens panic");
    let read_result = fs::read_to_string(path);
    match read_result {
        Ok(contents) => HttpResponse::Ok()
            .header(http::header::CONTENT_TYPE, "text/html")
            .body(contents),
        Err(e) => {
            log::error!("failed to read file ({}): {}", file_path_str, e);
            HttpResponse::NotFound().body("404 Page Not Found")
        }
    }
}

pub(super) fn redirect_to_user_app(req: HttpRequest) -> HttpResponse {
    log::info!("fn redirect_to_user_app: requested path: {}", req.uri());
    HttpResponse::PermanentRedirect()
        .header(header::LOCATION, "/user/user_app.html")
        .finish()
}

pub(super) fn redirect_to_advisor_app(req: HttpRequest) -> HttpResponse {
    log::info!("fn redirect_to_advisor_app: requested path: {}", req.uri());
    HttpResponse::PermanentRedirect()
        .header(header::LOCATION, "/advisor/advisor_app.html")
        .finish()
}
