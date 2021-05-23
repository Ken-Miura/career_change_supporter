// Copyright 2021 Ken Miura

use crate::common;
use actix_http::http;
use actix_web::{get, HttpRequest, HttpResponse, Result};
use once_cell::sync::Lazy;
use std::fs;
use std::path::PathBuf;

pub(super) static ADVISOR_ASSETS_DIR: Lazy<String> = Lazy::new(|| {
    format!(
        "{}{}static{}advisor",
        common::PACKAGE_NAME,
        std::path::MAIN_SEPARATOR,
        std::path::MAIN_SEPARATOR
    )
});

// https://host_name/user/registration-requests?id=registration_request_idでアクセスしたときのために利用する
// 該当しないURLにアクセスした際は、serve_indexにルーティングされる設定だが、今後registration-requestsの別ルートが間違って追加されないように明示的に関数を作っておく
#[get("/registration-requests")]
async fn registration_requests(req: HttpRequest) -> HttpResponse {
    serve_advisor_app(req)
}

pub(super) fn serve_advisor_app(req: HttpRequest) -> HttpResponse {
    log::info!("fn serve_advisor_app: requested path: {}", req.uri());
    let file_path_str = format!("{}/advisor_app.html", ADVISOR_ASSETS_DIR.to_string());
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
