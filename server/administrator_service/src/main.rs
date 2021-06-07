// Copyright 2021 Ken Miura

#[macro_use]
extern crate serde_json;

use actix_http::error::BlockingError;
use actix_http::{body::Body, Response};
use actix_web::dev::ServiceResponse;
use actix_web::http::StatusCode;
use actix_web::middleware::errhandlers::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{get, web, App, HttpResponse, HttpServer, Result};

use diesel::QueryDsl;
use diesel::RunQueryDsl;
use dotenv::dotenv;
use handlebars::Handlebars;

use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use rusoto_s3::S3;
use std::env;
use  std::fs::File;
use std::io::Write;
use futures::TryStreamExt;
use bytes;

use std::io;

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
            .app_data(handlebars_ref.clone())
            .data(pool.clone())
            .service(index)
            .service(user)
    })
    .bind("127.0.0.1:8082")?
    .run()
    .await
}

const AWS_S3_ID_IMG_BUCKET_NAME: &str = "identification-images";
const AWS_REGION: &str = "ap-northeast-1";
const AWS_ENDPOINT_URL: &str = "http://localhost:4566";

#[get("/")]
async fn index(
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
    });

    let get_request = rusoto_s3::GetObjectRequest {
        bucket: AWS_S3_ID_IMG_BUCKET_NAME.to_string(),
        key: request.image1.clone(),
        ..Default::default()
    };
    let region = rusoto_core::Region::Custom {
        name: AWS_REGION.to_string(),
        endpoint: AWS_ENDPOINT_URL.to_string(),
    };
    let s3_client = rusoto_s3::S3Client::new(region);
    let result = s3_client.get_object(get_request).await;
    let stream = result.unwrap().body.unwrap();
    let body = stream.map_ok(|b| bytes::BytesMut::from(&b[..])).try_concat().await.unwrap();

    let mut file = File::create(&request.image1).expect("create failed");
    file.write_all(&body).expect("failed to write body");
    
    let body = hb.render("index", &data).unwrap();
    HttpResponse::Ok().body(body)
}

#[get("/{user}/{data}")]
async fn user(
    hb: web::Data<Handlebars<'_>>,
    web::Path(info): web::Path<(String, String)>,
) -> HttpResponse {
    let data = json!({
        "user": info.0,
        "data": info.1
    });
    let body = hb.render("user", &data).unwrap();

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
