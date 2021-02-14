// Copyright 2021 Ken Miura
use actix_files;
use actix_files::NamedFile;
use actix_http::http;
use actix_web::{error, web, HttpRequest, HttpResponse, Result};
use serde::Deserialize;
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
    let path: PathBuf = (format!("{}{}", "./static/js/", last)).parse().unwrap();
    let contents = fs::read_to_string(path).unwrap();
    HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "application/javascript")
        .body(contents)
}

async fn css(req: HttpRequest) -> HttpResponse {
    let uri = req.uri();
    let path_str = uri.path();
    let v: Vec<&str> = path_str.split("/").collect();
    let last = v[v.len() - 1];
    let path: PathBuf = (format!("{}{}", "./static/css/", last)).parse().unwrap();
    let contents = fs::read(path).unwrap();
    HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "text/css")
        .body(contents)
}

async fn img(req: HttpRequest) -> HttpResponse {
    let uri = req.uri();
    let path_str = uri.path();
    let v: Vec<&str> = path_str.split("/").collect();
    let last = v[v.len() - 1];
    let path: PathBuf = (format!("{}{}", "./static/img/", last)).parse().unwrap();
    let contents = fs::read(path).unwrap();
    HttpResponse::Ok()
        .header(http::header::CONTENT_TYPE, "image/png")
        .body(contents)
}

#[derive(Deserialize)]
struct AuthInfo {
    mailaddress: String,
    password: String,
}

async fn authenticate(info: web::Json<AuthInfo>) -> HttpResponse {
    let mailaddress = info.mailaddress.clone();
    let password = info.password.clone();
    if mailaddress == "test@example.com" && password == "test" {
        let contents = "{ \"name\": \"test name\" }";
        HttpResponse::Ok().body(contents)
    } else {
        HttpResponse::from_error(error::ErrorUnauthorized("err: T"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_web::{web, App, HttpServer};

    HttpServer::new(|| {
        let json_config = web::JsonConfig::default()
            .limit(4096)
            .error_handler(|err, _req| {
                // create custom error response
                error::InternalError::from_response(err, HttpResponse::Conflict().finish()).into()
            });

        App::new()
            .service(actix_files::Files::new("/static", ".").show_files_listing()) // staticディレクトリ以下のファイルのサーブを許可する。
            //.app_data(json_config)
            .route("/js/*", web::get().to(js))
            .route("/css/*", web::get().to(css))
            .route("/img/*", web::get().to(img))
            .route("/index.html", web::get().to(index))
            .route("/", web::get().to(index))
            .route("/auth-info", web::post().to(authenticate))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
