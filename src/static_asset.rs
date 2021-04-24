// Copyright 2021 Ken Miura

use actix_files::NamedFile;
use actix_http::http;
use actix_web::{get, http::StatusCode, http::Uri, HttpRequest, HttpResponse, Result};
use std::fs;
use std::path::PathBuf;

pub(crate) const ASSETS_DIR: &str = "static";

#[get("/index.html")]
pub(crate) async fn index(req: HttpRequest) -> HttpResponse {
    serve_index(req)
}

// https://host_name/temporary_accounts?id=temporary_account_idでアクセスしたときのために利用する
// 該当しないURLにアクセスした際は、serve_indexにルーティングされる設定だが、今後temporary_accountsの別ルートが間違って追加されないように明示的に関数を作っておく
#[get("/temporary-accounts")]
pub(crate) async fn temporary_accounts(req: HttpRequest) -> HttpResponse {
    serve_index(req)
}

pub(crate) fn serve_index(req: HttpRequest) -> HttpResponse {
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

#[get("/js/*")]
pub(crate) async fn js(req: HttpRequest) -> Result<NamedFile, HttpResponse> {
    log::info!("fn js: requested path: {}", req.uri());
    let last_path = get_last_path(req.uri());
    // TODO: js.mapファイルをjsパス以下に含ませないようにする（またはmapファイルは返却しないようにする）
    let file_path_str = format!("{}/js/{}", ASSETS_DIR, last_path);
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
pub(crate) async fn css(req: HttpRequest) -> Result<NamedFile, HttpResponse> {
    log::info!("fn css: requested path: {}", req.uri());
    let last_path = get_last_path(req.uri());
    let file_path_str = format!("{}/css/{}", ASSETS_DIR, last_path);
    let f = get_named_file(&file_path_str)?;
    Ok(f)
}

#[get("/img/*")]
pub(crate) async fn img(req: HttpRequest) -> Result<NamedFile, HttpResponse> {
    log::info!("fn img: requested path: {}", req.uri());
    let last_path = get_last_path(req.uri());
    let file_path_str = format!("{}/img/{}", ASSETS_DIR, last_path);
    let f = get_named_file(&file_path_str)?;
    Ok(f)
}

#[get("/favicon.ico")]
pub(crate) async fn favicon_ico() -> Result<NamedFile, HttpResponse> {
    log::info!("favicon.ico requested");
    let file_path_str = format!("{}/favicon.ico", ASSETS_DIR);
    let f = get_named_file(&file_path_str)?;
    Ok(f)
}
