// Copyright 2021 Ken Miura

mod models;
mod schema;

#[macro_use]
extern crate diesel;

use actix_http::http;
use actix_web::{
    dev::Body, error, get, http::StatusCode, http::Uri, post, web, App, HttpRequest, HttpResponse,
    HttpServer, Result,
};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use dotenv::dotenv;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::PathBuf;

static ASSETS_DIR: &str = "static";

#[get("/index.html")]
async fn index(req: HttpRequest) -> HttpResponse {
    // TODO: Add log accessing index
    serve_index(req)
}

fn serve_index(_req: HttpRequest) -> HttpResponse {
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
async fn js(req: HttpRequest) -> HttpResponse {
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
async fn css(req: HttpRequest) -> HttpResponse {
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
async fn img(req: HttpRequest) -> HttpResponse {
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

#[derive(Deserialize)]
struct AuthInfo {
    mailaddress: String,
    password: String,
}

fn find_user_by_mail_address(
    mailaddress: &String,
    conn: &PgConnection,
) -> Result<Option<models::User>, diesel::result::Error> {
    use self::schema::user_data::user::dsl::*;
    let result = user.filter(mail_addr.eq(mailaddress))
        .first::<models::User>(conn)
    .optional()?;
    Ok(result)
}

#[post("/auth-request")]
async fn auth_request(info: web::Json<AuthInfo>, pool: web::Data<Pool<ConnectionManager<PgConnection>>>) -> HttpResponse {
    let mailaddress = info.mailaddress.clone();
    let password = info.password.clone();

    let conn = pool.get().expect("failed to get connection");

    let user = web::block(move || find_user_by_mail_address(&mailaddress, &conn)).await;

    let info = user.expect("error");
    let mut auth_res = false;
    match info {
        Some(user) => {
            auth_res = password == user.hashed_pass;
        },
        None => {}
    }

    if auth_res {
        let contents = "{ \"result\": \"OK\" }";
        HttpResponse::Ok().body(contents)
    } else {
        HttpResponse::from_error(error::ErrorUnauthorized("err: T"))
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager =
        ConnectionManager::<PgConnection>::new(&database_url);
    let pool: Pool<ConnectionManager<PgConnection>> = Pool::builder().build(manager).expect("failed to create connection pool");

    HttpServer::new(move || {
        App::new()
            .service(actix_files::Files::new(ASSETS_DIR, ".").show_files_listing())
            .service(js)
            .service(css)
            .service(img)
            .service(index)
            .service(auth_request)
            .default_service(web::route().to(serve_index))
            .data(pool.clone())
    })
    .bind("127.0.0.1:8081")?
    .run()
    .await
}
