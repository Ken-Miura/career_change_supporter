// Copyright 2021 Ken Miura

use actix_files::NamedFile;
use actix_http::http;
use actix_web::{get, http::StatusCode, http::Uri, HttpRequest, HttpResponse, Result};
use once_cell::sync::Lazy;
use std::fs;
use std::path::PathBuf;

pub(super) static USER_ASSETS_DIR: Lazy<String> =
    Lazy::new(|| format!("static{}user", std::path::MAIN_SEPARATOR));

#[get("/user_app.html")]
async fn user_app(req: HttpRequest) -> HttpResponse {
    serve_user_app(req)
}

// https://host_name/temporary_accounts?id=temporary_account_idでアクセスしたときのために利用する
// 該当しないURLにアクセスした際は、serve_indexにルーティングされる設定だが、今後temporary_accountsの別ルートが間違って追加されないように明示的に関数を作っておく
#[get("/temporary-accounts")]
async fn temporary_accounts(req: HttpRequest) -> HttpResponse {
    serve_user_app(req)
}

pub(super) fn serve_user_app(req: HttpRequest) -> HttpResponse {
    log::info!("fn serve_user_app: requested path: {}", req.uri());
    let file_path_str = format!("{}/user_app.html", USER_ASSETS_DIR.to_string());
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

#[get("/js/*")]
async fn js(req: HttpRequest) -> Result<NamedFile, HttpResponse> {
    log::info!("fn js: requested path: {}", req.uri());
    let last_path = get_last_path(req.uri());
    // TODO: js.mapファイルをjsパス以下に含ませないようにする（またはmapファイルは返却しないようにする）
    let file_path_str = format!("{}/js/{}", USER_ASSETS_DIR.to_string(), last_path);
    let f = get_named_file(&file_path_str)?;
    Ok(f)
}

fn get_last_path(uri: &Uri) -> String {
    let path = uri.path();
    let paths: Vec<&str> = path.split('/').collect();
    paths[paths.len() - 1].to_string()
}

fn get_named_file(file_path_str: &str) -> Result<NamedFile, HttpResponse> {
    let path = file_path_str
        .parse::<PathBuf>()
        .map_err::<HttpResponse, _>(|e| {
            log::error!(
                "unexpected error: failed to parse path ({}): {}",
                file_path_str,
                e
            );
            HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR).finish()
        })?;
    // TODO: Check if path validation is needed for security
    let f = NamedFile::open(path).map_err(|e| {
        log::error!(
            "unexpected error: failed to read file ({}): {}",
            file_path_str,
            e
        );
        HttpResponse::build(StatusCode::NOT_FOUND).finish()
    })?;
    Ok(f)
}

#[get("/css/*")]
async fn css(req: HttpRequest) -> Result<NamedFile, HttpResponse> {
    log::info!("fn css: requested path: {}", req.uri());
    let last_path = get_last_path(req.uri());
    let file_path_str = format!("{}/css/{}", USER_ASSETS_DIR.to_string(), last_path);
    let f = get_named_file(&file_path_str)?;
    Ok(f)
}

#[get("/img/*")]
async fn img(req: HttpRequest) -> Result<NamedFile, HttpResponse> {
    log::info!("fn img: requested path: {}", req.uri());
    let last_path = get_last_path(req.uri());
    let file_path_str = format!("{}/img/{}", USER_ASSETS_DIR.to_string(), last_path);
    let f = get_named_file(&file_path_str)?;
    Ok(f)
}

#[get("/favicon.ico")]
async fn favicon_ico() -> Result<NamedFile, HttpResponse> {
    log::info!("favicon.ico requested");
    let file_path_str = format!("{}/favicon.ico", USER_ASSETS_DIR.to_string());
    let f = get_named_file(&file_path_str)?;
    Ok(f)
}
