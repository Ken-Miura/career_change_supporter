// Copyright 2021 Ken Miura

use actix_http::http;
use actix_web::{
    dev::Body, get, http::StatusCode, http::Uri, HttpRequest, HttpResponse, Result,
};
use std::fs;
use std::path::PathBuf;

pub(crate) static ASSETS_DIR: &str = "static";

#[get("/index.html")]
pub(crate) async fn index(req: HttpRequest) -> HttpResponse {
    // TODO: Add log accessing index
    serve_index(req)
}

pub(crate) fn serve_index(_req: HttpRequest) -> HttpResponse {
    // TODO: Add log what url user acccess (index accepts all the paths)
    let index_file = format!("{}/index.html", ASSETS_DIR);
    let parse_result: Result<PathBuf, _> = index_file.parse();
    if let Err(_) = parse_result {
        // TODO: Add log we failed to parse path to index
        return HttpResponse::InternalServerError().body("500 Internal Server Error");
    }
    let path = parse_result.expect("never happen err");
    let read_result = fs::read_to_string(path);
    match read_result {
        Ok(contents) => HttpResponse::Ok()
            .header(http::header::CONTENT_TYPE, "text/html")
            .body(contents),
        Err(_) => HttpResponse::NotFound().body("404 Page Not Found"),
    }
}

#[get("/js/*")]
pub(crate) async fn js(req: HttpRequest) -> HttpResponse {
    // TODO: Add log what file is requested
    let last_path = get_last_path(req.uri());
    let js_file = format!("{}/js/{}", ASSETS_DIR, last_path);
    let parse_result: Result<PathBuf, _> = js_file.parse();
    if let Err(_) = parse_result {
        // TODO: Add log what file we failed to parse
        return HttpResponse::with_body(StatusCode::NOT_FOUND, Body::Empty);
    }
    let path = parse_result.expect("never happen err");
    // TODO: Check if path validation is needed for security
    let read_result = fs::read_to_string(path);
    match read_result {
        Ok(contents) => HttpResponse::Ok()
            .header(http::header::CONTENT_TYPE, "application/javascript")
            .body(contents),
        Err(_) => HttpResponse::with_body(StatusCode::NOT_FOUND, Body::Empty),
    }
}

fn get_last_path(uri: &Uri) -> String {
    let path = uri.path();
    let paths: Vec<&str> = path.split("/").collect();
    paths[paths.len() - 1].to_string()
}

#[get("/css/*")]
pub(crate) async fn css(req: HttpRequest) -> HttpResponse {
    // TODO: Add log what file is requested
    let last_path = get_last_path(req.uri());
    let css_file = format!("{}/css/{}", ASSETS_DIR, last_path);
    let parse_result: Result<PathBuf, _> = css_file.parse();
    if let Err(_) = parse_result {
        // TODO: Add log what file we failed to parse
        return HttpResponse::with_body(StatusCode::NOT_FOUND, Body::Empty);
    }
    let path = parse_result.expect("never happen err");
    // TODO: Check if path validation is needed for security
    let read_result = fs::read_to_string(path);
    match read_result {
        Ok(contents) => HttpResponse::Ok()
            .header(http::header::CONTENT_TYPE, "text/css")
            .body(contents),
        Err(_) => HttpResponse::with_body(StatusCode::NOT_FOUND, Body::Empty),
    }
}

#[get("/img/*")]
pub(crate) async fn img(req: HttpRequest) -> HttpResponse {
    // TODO: Add log what file is requested
    let last_path = get_last_path(req.uri());
    let img_file = format!("{}/img/{}", ASSETS_DIR, last_path);
    let parse_result: Result<PathBuf, _> = img_file.parse();
    if let Err(_) = parse_result {
        // TODO: Add log what file we failed to parse
        return HttpResponse::with_body(StatusCode::NOT_FOUND, Body::Empty);
    }
    let path = parse_result.expect("never happen err");
    // TODO: Check if path validation is needed for security
    let read_result = fs::read(path);
    match read_result {
        Ok(contents) => HttpResponse::Ok()
            // TODO: Add code for other image types lik jpg
            .header(http::header::CONTENT_TYPE, "image/png")
            .body(contents),
        Err(_) => HttpResponse::with_body(StatusCode::NOT_FOUND, Body::Empty),
    }
}