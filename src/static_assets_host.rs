// Copyright 2021 Ken Miura

use actix_http::http;
use actix_web::{dev::Body, get, http::StatusCode, http::Uri, HttpRequest, HttpResponse, Result};
use std::fs;
use std::path::PathBuf;

pub(crate) static ASSETS_DIR: &str = "static";

#[get("/index.html")]
pub(crate) async fn index(req: HttpRequest) -> HttpResponse {
    serve_index(req)
}

pub(crate) fn serve_index(req: HttpRequest) -> HttpResponse {
    log::info!("fn serve_index: requested path: {}", req.uri());
    let index_file = format!("{}/index.html", ASSETS_DIR);
    let parse_result: Result<PathBuf, _> = index_file.parse();
    if let Err(e) = parse_result {
        log::error!(
            "failed to parse path to index.html (path: {}): {}",
            index_file,
            e
        );
        return HttpResponse::InternalServerError().body("500 Internal Server Error");
    }
    let path = parse_result.expect("never happens panic");
    let read_result = fs::read_to_string(path);
    match read_result {
        Ok(contents) => HttpResponse::Ok()
            .header(http::header::CONTENT_TYPE, "text/html")
            .body(contents),
        Err(e) => {
            log::error!("failed to read index file (path: {}): {}", index_file, e);
            HttpResponse::NotFound().body("404 Page Not Found")
        }
    }
}

#[get("/js/*")]
pub(crate) async fn js(req: HttpRequest) -> HttpResponse {
    let last_path = get_last_path(req.uri());
    let js_file = format!("{}/js/{}", ASSETS_DIR, last_path);
    let parse_result: Result<PathBuf, _> = js_file.parse();
    if let Err(e) = parse_result {
        log::error!("failed to parse path to js (path: {}): {}", js_file, e);
        return HttpResponse::with_body(StatusCode::INTERNAL_SERVER_ERROR, Body::Empty);
    }
    let path = parse_result.expect("never happens panic");
    // TODO: Check if path validation is needed for security
    let read_result = fs::read_to_string(path);
    match read_result {
        Ok(contents) => HttpResponse::Ok()
            .header(http::header::CONTENT_TYPE, "application/javascript")
            .body(contents),
        Err(e) => {
            log::error!("failed to read js file (path: {}): {}", js_file, e);
            HttpResponse::with_body(StatusCode::NOT_FOUND, Body::Empty)
        }
    }
}

fn get_last_path(uri: &Uri) -> String {
    let path = uri.path();
    let paths: Vec<&str> = path.split('/').collect();
    paths[paths.len() - 1].to_string()
}

#[get("/css/*")]
pub(crate) async fn css(req: HttpRequest) -> HttpResponse {
    let last_path = get_last_path(req.uri());
    let css_file = format!("{}/css/{}", ASSETS_DIR, last_path);
    let parse_result: Result<PathBuf, _> = css_file.parse();
    if let Err(e) = parse_result {
        log::error!("failed to parse path to css (path: {}): {}", css_file, e);
        return HttpResponse::with_body(StatusCode::INTERNAL_SERVER_ERROR, Body::Empty);
    }
    let path = parse_result.expect("never happens panic");
    // TODO: Check if path validation is needed for security
    let read_result = fs::read_to_string(path);
    match read_result {
        Ok(contents) => HttpResponse::Ok()
            .header(http::header::CONTENT_TYPE, "text/css")
            .body(contents),
        Err(e) => {
            log::error!("failed to read css file (path: {}): {}", css_file, e);
            HttpResponse::with_body(StatusCode::NOT_FOUND, Body::Empty)
        }
    }
}

#[get("/img/*")]
pub(crate) async fn img(req: HttpRequest) -> HttpResponse {
    let last_path = get_last_path(req.uri());
    let img_file = format!("{}/img/{}", ASSETS_DIR, last_path);
    let parse_result: Result<PathBuf, _> = img_file.parse();
    if let Err(e) = parse_result {
        log::error!("failed to parse path to img (path: {}): {}", img_file, e);
        return HttpResponse::with_body(StatusCode::INTERNAL_SERVER_ERROR, Body::Empty);
    }
    let path = parse_result.expect("never happens panic");
    // TODO: Check if path validation is needed for security
    let read_result = fs::read(path);
    match read_result {
        Ok(contents) => HttpResponse::Ok()
            // TODO: Add code for other image types lik jpg
            .header(http::header::CONTENT_TYPE, "image/png")
            .body(contents),
        Err(e) => {
            log::error!("failed to read img file (path: {}): {}", img_file, e);
            HttpResponse::with_body(StatusCode::NOT_FOUND, Body::Empty)
        }
    }
}

#[get("/favicon.ico")]
pub(crate) async fn favicon_ico() -> HttpResponse {
    let fav_file = format!("{}/favicon.ico", ASSETS_DIR);
    let parse_result: Result<PathBuf, _> = fav_file.parse();
    if let Err(e) = parse_result {
        log::error!(
            "failed to parse path to favicon.ico (path: {}): {}",
            fav_file,
            e
        );
        return HttpResponse::with_body(StatusCode::INTERNAL_SERVER_ERROR, Body::Empty);
    }
    let path = parse_result.expect("never happens panic");
    // TODO: Check if path validation is needed for security
    let read_result = fs::read(path);
    match read_result {
        Ok(contents) => HttpResponse::Ok()
            // TODO: image/x-iconと比較し、どちらにすべきか検討する
            .header(http::header::CONTENT_TYPE, "image/vnd.microsoft.icon")
            .body(contents),
        Err(e) => {
            log::error!("failed to read favicon.ico (path: {}): {}", fav_file, e);
            HttpResponse::with_body(StatusCode::NOT_FOUND, Body::Empty)
        }
    }
}
