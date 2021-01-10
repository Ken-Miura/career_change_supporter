// Copyright 2021 Ken Miura
use actix_files;
use actix_files::NamedFile;
use actix_http::http;
use actix_web::{HttpRequest, HttpResponse, Result};
use std::fs;
use std::path::PathBuf;

async fn index(_req: HttpRequest) -> Result<NamedFile> {
    let path: PathBuf = "./static/index.html".parse().unwrap();
    Ok(NamedFile::open(path)?)
}

async fn js(req: HttpRequest) -> HttpResponse {
    let uri = req.uri();
    let path_str = uri.path();
    let v: Vec<&str> = path_str.split("/").collect();
    let last = v[v.len() - 1];
    let path: PathBuf = (format!("{}{}", "./static/", last)).parse().unwrap();
    let contents = fs::read_to_string(path).unwrap();
    HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "application/javascript")
        .body(contents)
}

async fn wasm(req: HttpRequest) -> HttpResponse {
    let uri = req.uri();
    let path_str = uri.path();
    let v: Vec<&str> = path_str.split("/").collect();
    let last = v[v.len() - 1];
    let path: PathBuf = (format!("{}{}", "./static/", last)).parse().unwrap();
    let contents = fs::read(path).unwrap();
    HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "application/wasm")
        .body(contents)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{web, App, HttpServer};

    HttpServer::new(|| {
        App::new()
            .service(actix_files::Files::new("/static", ".").show_files_listing()) // staticディレクトリ以下のファイルのサーブを許可する。
            .route("/wasm.js", web::get().to(js))
            .route("/wasm_bg.wasm", web::get().to(wasm))
            .route("/index.html", web::get().to(index))
            .route("/*", web::get().to(index)) // あらゆるパスにマッチする
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
