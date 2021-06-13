// Copyright 2021 Ken Miura

#[macro_use]
extern crate serde_json;

use actix_http::error::BlockingError;
use actix_http::{body::Body, Response};
use actix_web::dev::ServiceResponse;
use actix_web::http::StatusCode;
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Result};

use actix_web::cookie;
use time::Duration;

use actix_redis::RedisSession;

use diesel::QueryDsl;
use diesel::RunQueryDsl;
use dotenv::dotenv;
use handlebars::Handlebars;

use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use futures::TryStreamExt;
use rusoto_s3::S3;
use std::env;
use std::fs;
use std::path;
use actix_session::Session;

use std::io;

// TODO: Consider and change KEY
const ADMINISTRATOR_SESSION_SIGN_KEY: [u8; 32] = [1; 32];
const CACHE_SERVER_ADDR: &str = "127.0.0.1:6379";

#[actix_web::main]
async fn main() -> io::Result<()> {
    dotenv().ok();

    std::env::set_var("RUST_LOG", "info");
    std::env::set_var("RUST_BACKTRACE", "1");
    env_logger::init();

    let mut handlebars = Handlebars::new();
    handlebars
        .register_templates_directory(".html", "./administrator_service/static/templates")
        .unwrap();
    let handlebars_ref = web::Data::new(handlebars);

    let database_url = env::var("ADMINISTRATOR_APP_DATABASE_URL")
        .expect("ADMINISTRATOR_APP_DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool: Pool<ConnectionManager<PgConnection>> = Pool::builder()
        /*
         * NOTE: actixのarchitectureより、1 Actor（1スレッド）ごとにconnection poolを作成して割り当てる。
         * 1スレッドあたり1コネクションで十分と思われるため、max_sizeを1に設定する。
         */
        .max_size(1)
        .build(manager)
        .expect("never fails to create connection pool");

    HttpServer::new(move || {
        App::new()
            .wrap(error_handlers())
            .wrap(Logger::default())
            .wrap(
                RedisSession::new(CACHE_SERVER_ADDR, &ADMINISTRATOR_SESSION_SIGN_KEY)
                    // TODO: 適切なTTLを設定する
                    .ttl(180)
                    .cookie_max_age(Duration::days(7))
                    // TODO: Add producion environment
                    //.cookie_secure(true)
                    .cookie_name("administrator-session")
                    .cookie_http_only(true)
                    // TODO: Consider LAX policy
                    .cookie_same_site(cookie::SameSite::Strict),
            )
            .app_data(handlebars_ref.clone())
            .data(pool.clone())
            .service(index)
            .service(login)
            .service(images)
            .service(advisor_registration_list)
            .service(authentication)
    })
    .bind("127.0.0.1:8082")?
    .run()
    .await
}

const AWS_S3_ID_IMG_BUCKET_NAME: &str = "identification-images";
const AWS_REGION: &str = "ap-northeast-1";
const AWS_ENDPOINT_URL: &str = "http://localhost:4566";

#[get("/login")]
async fn login() -> HttpResponse {
    let file_path_str = "administrator_service/static/login.html";
    let parse_result: Result<path::PathBuf, _> = file_path_str.parse();
    if let Err(e) = parse_result {
        log::error!("failed to parse path ({}): {}", file_path_str, e);
        return HttpResponse::InternalServerError().body("500 Internal Server Error");
    }
    let path = parse_result.expect("never happens panic");
    let read_result = fs::read_to_string(path);
    match read_result {
        Ok(contents) => HttpResponse::Ok()
            .header(actix_web::http::header::CONTENT_TYPE, "text/html")
            .body(contents),
        Err(e) => {
            log::error!("failed to read file ({}): {}", file_path_str, e);
            HttpResponse::NotFound().body("404 Page Not Found")
        }
    }
}

#[post("/authentication")]
async fn authentication() -> HttpResponse {
    return HttpResponse::InternalServerError().body("500 Internal Server Error");
}

#[get("/")]
async fn index(session: Session) -> HttpResponse {
    let res = check_session(&session);
    if res.is_err() {
        return res.expect_err("OK value detected");
    }
    let file_path_str = "administrator_service/static/index.html";
    let parse_result: Result<path::PathBuf, _> = file_path_str.parse();
    if let Err(e) = parse_result {
        log::error!("failed to parse path ({}): {}", file_path_str, e);
        return HttpResponse::InternalServerError().body("500 Internal Server Error");
    }
    let path = parse_result.expect("never happens panic");
    let read_result = fs::read_to_string(path);
    match read_result {
        Ok(contents) => HttpResponse::Ok()
            .header(actix_web::http::header::CONTENT_TYPE, "text/html")
            .body(contents),
        Err(e) => {
            log::error!("failed to read file ({}): {}", file_path_str, e);
            HttpResponse::NotFound().body("404 Page Not Found")
        }
    }
}

const KEY_TO_ADMINISTRATOR_ACCOUNT_ID: &str = "administrator_account_id";

fn check_session(session: &Session) -> Result<(), HttpResponse> {
    let option_acc_id: Option<i32> = session.get(KEY_TO_ADMINISTRATOR_ACCOUNT_ID).map_err(|err| {
        log::error!("err: {}", err);
        HttpResponse::InternalServerError().finish()
    })?;
    match option_acc_id {
        Some(_acc_id) => {
            Ok(())
        }
        None => {
            let res = HttpResponse::Unauthorized().body(r#"<!DOCTYPE html>
            <html>
                <head>
                    <meta charset="utf-8">
                    <style type="text/css">
                      .container{
                        display: flex;
                        justify-content: center;
                        align-items: center;
                        flex-direction: column;
                      }
                    </style> 
                    <title>管理者メニュー</title>
                </head>
                <body>
                  <div class="container">
                    <p>セッションが切れています。</p>
                    <p><a href="login">ログインページへ</a></p>
                  </div>
                </body>
            </html>"#);
            Err(res)
        }
    }
}

#[get("/images/{data}")]
async fn images(web::Path(image_path): web::Path<String>) -> HttpResponse {
    let get_request = rusoto_s3::GetObjectRequest {
        bucket: AWS_S3_ID_IMG_BUCKET_NAME.to_string(),
        key: image_path.clone(),
        ..Default::default()
    };
    let region = rusoto_core::Region::Custom {
        name: AWS_REGION.to_string(),
        endpoint: AWS_ENDPOINT_URL.to_string(),
    };
    let s3_client = rusoto_s3::S3Client::new(region);
    let result = s3_client.get_object(get_request).await;
    let stream = result.unwrap().body.unwrap();
    let body = stream
        .map_ok(|b| bytes::BytesMut::from(&b[..]))
        .try_concat()
        .await
        .unwrap();
    let contents = body.to_vec();
    HttpResponse::Ok()
        .header(actix_web::http::header::CONTENT_TYPE, "img/png")
        .body(contents)
}

#[get("/advisor-registration-list")]
async fn advisor_registration_list(
    hb: web::Data<Handlebars<'_>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> HttpResponse {
    let conn = pool.get().unwrap();
    // TODO: エラー処理の追加
    let result: Result<_, BlockingError<String>> = web::block(move || {
        use db::schema::career_change_supporter_schema::advisor_account_creation_request::dsl::{
            advisor_account_creation_request
        };
        let requests = advisor_account_creation_request
            .limit(100)
            .load::<db::model::advisor::AccountCreationRequestResult>(&conn)
            .expect("failed to get data");
        Ok(requests)
    }).await;

    let requests = result.unwrap();
    let request = requests[0].clone();
    let data = json!({
        "last_name": request.last_name,
        "requested_time": request.requested_time,
        "image1": request.image1,
        "image2": request.image2,
    });

    let body = hb.render("advisor-registration-list", &data).unwrap();
    HttpResponse::Ok().body(body)
}

// Custom error handlers, to return HTML responses when an error occurs.
fn error_handlers() -> ErrorHandlers<Body> {
    ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found)
}

// Error handler for a 404 Page not found error.
fn not_found<B>(res: ServiceResponse<B>) -> Result<ErrorHandlerResponse<B>> {
    let response = get_error_response(&res, "Page not found");
    Ok(ErrorHandlerResponse::Response(
        res.into_response(response.into_body()),
    ))
}

// Generic error handler.
fn get_error_response<B>(res: &ServiceResponse<B>, error: &str) -> Response<Body> {
    let request = res.request();

    // Provide a fallback to a simple plain text response in case an error occurs during the
    // rendering of the error page.
    let fallback = |e: &str| {
        Response::build(res.status())
            .content_type("text/plain")
            .body(e.to_string())
    };

    let hb = request
        .app_data::<web::Data<Handlebars>>()
        .map(|t| t.get_ref());
    match hb {
        Some(hb) => {
            let data = json!({
                "error": error,
                "status_code": res.status().as_str()
            });
            let body = hb.render("error", &data);

            match body {
                Ok(body) => Response::build(res.status())
                    .content_type("text/html")
                    .body(body),
                Err(_) => fallback(error),
            }
        }
        None => fallback(error),
    }
}
