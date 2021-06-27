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

use diesel::prelude::*;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use dotenv::dotenv;
use handlebars::to_json;
use handlebars::Handlebars;

use actix_session::Session;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use futures::TryStreamExt;
use rusoto_s3::S3;
use serde_json::value::Map;
use std::env;
use std::fs;
use std::path;

use serde::Deserialize;

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
            .service(advisor_registration_detail)
            .service(advisor_registration_accept)
            .service(advisor_registration_reject_detail)
            .service(advisor_registration_reject)
            .service(advisor_registration_approval_list)
            .service(advisor_registration_rejection_list)
            .service(advisor_registration_rejection_detail)
            .service(advisor_registration_approval_detail)
            .service(authentication)
            .default_service(web::route().to(index_inner))
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

#[derive(Deserialize)]
struct AccountInfo {
    email: String,
    password: String,
}

#[post("/authentication")]
async fn authentication(
    session: Session,
    params: web::Form<AccountInfo>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> HttpResponse {
    let email = params.email.clone();
    let pwd = params.password.clone();
    let pool_clone = pool.clone();

    let admin = web::block(move || {
        let conn = pool_clone.get().expect("Failed to get conn");
        use db::schema::career_change_supporter_schema::administrator_account::dsl::{
            administrator_account, email_address,
        };
        let target = administrator_account.filter(email_address.eq(email));
        let admins = match target.get_results::<db::model::administrator::AccountQueryResult>(&conn)
        {
            Ok(admins) => admins,
            Err(e) => {
                return Err(e);
            }
        };
        if admins.len() != 1 {
            panic!("Failed to get account");
        }
        let admin = admins[0].clone();
        Ok(admin)
    })
    .await
    .expect("Failed to proccess db access");
    let pwd_str = String::from_utf8(admin.hashed_password).expect("Failed to get pwd str");
    let verified = bcrypt::verify(pwd, &pwd_str).expect("Failed to verify");
    if verified {
        let _ = web::block(move || {
            let conn = pool.get().expect("Failed to get conn");
            use db::schema::career_change_supporter_schema::administrator_account::dsl::{
                administrator_account, email_address, last_login_time,
            };
            let current_date_time = chrono::Utc::now();
            let target = administrator_account.filter(email_address.eq(params.email.clone()));
            let result = diesel::update(target)
                .set(last_login_time.eq(&current_date_time))
                .execute(&conn);
            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(e),
            }
        })
        .await;
        let _ = session
            .set(
                KEY_TO_ADMINISTRATOR_ACCOUNT_ID,
                admin.administrator_account_id,
            )
            .expect("Failed to set value");
        HttpResponse::PermanentRedirect()
            .header(actix_web::http::header::LOCATION, "/")
            .finish()
    } else {
        HttpResponse::Unauthorized().body(LOGIN_ERR_TEMPLATE)
    }
}

#[get("/")]
async fn index(session: Session) -> HttpResponse {
    index_inner(session)
}

fn index_inner(session: Session) -> HttpResponse {
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
    let option_acc_id: Option<i32> =
        session
            .get(KEY_TO_ADMINISTRATOR_ACCOUNT_ID)
            .map_err(|err| {
                log::error!("err: {}", err);
                HttpResponse::InternalServerError().finish()
            })?;
    match option_acc_id {
        Some(_acc_id) => Ok(()),
        None => {
            let res = HttpResponse::Unauthorized().body(LOGIN_ERR_TEMPLATE);
            Err(res)
        }
    }
}

const LOGIN_ERR_TEMPLATE: &str = r#"<!DOCTYPE html>
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
</html>"#;

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

    let mut requests = result.unwrap();
    requests.sort_by(|a, b| a.requested_time.cmp(&b.requested_time));
    let mut data = Map::new();
    data.insert("num".to_string(), to_json(requests.len()));
    let mut items = Vec::new();
    // TODO: for in (Iterator) が順番通りに処理されることを確認
    for request in requests {
        let value = json!({
            "last_name": request.last_name,
            "first_name": request.first_name,
            "requested_time": request.requested_time,
            "id": request.advisor_acc_request_id
        });
        items.push(value);
    }
    data.insert("items".to_string(), to_json(items));
    let body = hb.render("advisor-registration-list", &data).unwrap();
    HttpResponse::Ok().body(body)
}

#[derive(Deserialize)]
struct DetailRequest {
    id: i32,
}

#[get("/advisor-registration-detail")]
async fn advisor_registration_detail(
    web::Query(info): web::Query<DetailRequest>,
    hb: web::Data<Handlebars<'_>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> HttpResponse {
    let conn = pool.get().unwrap();
    // TODO: エラー処理の追加
    let result: Result<_, BlockingError<String>> = web::block(move || {
        use db::schema::career_change_supporter_schema::advisor_account_creation_request::dsl::{
            advisor_account_creation_request
        };
        let request = advisor_account_creation_request.find(info.id)
            .first::<db::model::advisor::AccountCreationRequestResult>(&conn)
            .expect("failed to get data");
        Ok(request)
    }).await;

    let request = result.unwrap();
    let address_line2_option: Option<String> = request.address_line2;
    let address_line2_exists = address_line2_option.is_some();
    let data = json!({
        "id": request.advisor_acc_request_id,
        "requested_time": request.requested_time,
        "last_name": request.last_name,
        "first_name": request.first_name,
        "last_name_furigana": request.last_name_furigana,
        "first_name_furigana": request.first_name_furigana,
        "year": request.year_of_birth,
        "month": request.month_of_birth,
        "day": request.day_of_birth,
        "prefecture": request.prefecture,
        "city": request.city,
        "address_line1": request.address_line1,
        "address_line2_exists": address_line2_exists,
        "address_line2": address_line2_option.expect("Failed to get address line 2"),
        "email_address": request.email_address,
        "telephone_num": request.telephone_number,
        "image1": request.image1,
        "image2": request.image2,
    });

    let body = hb.render("advisor-registration-detail", &data).unwrap();
    HttpResponse::Ok().body(body)
}

#[get("/advisor-registration-accept")]
async fn advisor_registration_accept(
    web::Query(info): web::Query<DetailRequest>,
    hb: web::Data<Handlebars<'_>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> HttpResponse {
    let conn = pool.get().unwrap();
    // TODO: エラー処理の追加
    let result: Result<String, BlockingError<diesel::result::Error>> = web::block(move || {
        conn.transaction::<_, diesel::result::Error, _>(|| {
            use db::schema::career_change_supporter_schema::advisor_account_creation_request::dsl::{
                advisor_account_creation_request
            };
            let request = advisor_account_creation_request.find(info.id)
                .first::<db::model::advisor::AccountCreationRequestResult>(&conn)
                .expect("failed to get data");

            let acc = db::model::advisor::Account {
                email_address: &request.email_address,
                hashed_password: &request.hashed_password,
                last_login_time: None
            };
            use db::schema::career_change_supporter_schema::advisor_account;
            let res = diesel::insert_into(advisor_account::table)
                .values(&acc)
                .get_result::<db::model::advisor::AccountQueryResult>(&conn)
                .expect("Failed to insert data");

            let current_date_time = chrono::Utc::now();
            let addr_line2= request.address_line2.unwrap();
            let img2 = request.image2.unwrap();
            let approval_data = db::model::administrator::AdvisorRegReqApproved {
                email_address: &request.email_address,
                last_name: &request.last_name,
                first_name: &request.first_name,
                last_name_furigana: &request.last_name_furigana,
                first_name_furigana: &request.first_name_furigana,
                telephone_number: &request.telephone_number,
                year_of_birth: request.year_of_birth,
                month_of_birth: request.month_of_birth,
                day_of_birth: request.day_of_birth,
                prefecture: &request.prefecture,
                city: &request.city,
                address_line1: &request.address_line1,
                address_line2: Some(&addr_line2),
                image1: &request.image1,
                image2: Some(&img2),
                associated_advisor_account_id: Some(res.advisor_account_id),
                approved_time: &current_date_time,
            };
            use db::schema::career_change_supporter_schema::advisor_reg_req_approved;
            let _res = diesel::insert_into(advisor_reg_req_approved::table)
                .values(&approval_data)
                .get_result::<db::model::administrator::AdvisorRegReqApprovedResult>(&conn)
                .expect("Failed to insert data");

            let _del_res = diesel::delete(advisor_account_creation_request.find(info.id)).execute(&conn).expect("Failed to delete req");

            Ok(request.email_address)
        })
    })
    .await;

    let mail_addr = result.expect("Failed to get data");
    let _result = send_notification_mail_to_advisor(&mail_addr);

    let data = json!({
        "email_address": mail_addr,
    });

    let body = hb.render("advisor-registration-accepted", &data).unwrap();
    HttpResponse::Ok().body(body)
}

const SMTP_SERVER_ADDR: ([u8; 4], u16) = ([127, 0, 0, 1], 1025);

fn send_notification_mail_to_advisor(email_address: &str) -> Result<(), lettre::error::Error> {
    use lettre::{ClientSecurity, SmtpClient, Transport};
    use lettre_email::EmailBuilder;
    let email = EmailBuilder::new()
        // TODO: ドメイン取得後書き直し
        .to(email_address)
        // TODO: 送信元メールを更新する
        .from("from@example.com")
        // TOOD: メールの件名を更新する
        .subject("アカウント作成完了")
        // TOOD: メールの本文を更新する (http -> httpsへの変更も含む)
        .text(format!(
            "{}からアドバイザーアカウントの作成が完了しました",
            email_address
        ))
        .build()
        .expect("Failed to build");

    use std::net::SocketAddr;
    let addr = SocketAddr::from(SMTP_SERVER_ADDR);
    let client = SmtpClient::new(addr, ClientSecurity::None).expect("Failed to create clietn");
    let mut mailer = client.transport();
    // TODO: メール送信後のレスポンスが必要か検討する
    let _ = mailer.send(email.into()).expect("Failed to send email");
    Ok(())
}

#[get("/advisor-registration-reject-detail")]
async fn advisor_registration_reject_detail(
    web::Query(info): web::Query<DetailRequest>,
    hb: web::Data<Handlebars<'_>>,
) -> HttpResponse {
    let data = json!({
        "id": info.id,
    });
    let body = hb
        .render("advisor-registration-reject-detail", &data)
        .unwrap();
    HttpResponse::Ok().body(body)
}

#[derive(Deserialize)]
struct Reason {
    reason: String,
}

#[post("/advisor-registration-reject")]
async fn advisor_registration_reject(
    web::Query(info): web::Query<DetailRequest>,
    hb: web::Data<Handlebars<'_>>,
    params: web::Form<Reason>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> HttpResponse {
    // 1から3はトランザクション
    // 1. advisorリクエスト削除
    // 2. advisor拒絶履歴登録
    // 3. advisor身分証ファイル削除（オプショナルにしておく？）
    // advisorに拒絶メール通知
    // 拒絶画面表示
    let reson_clone = params.reason.clone();
    let conn = pool.get().unwrap();
    // TODO: エラー処理の追加
    let result: Result<db::model::advisor::AccountCreationRequestResult, BlockingError<diesel::result::Error>> = web::block(move || {
        conn.transaction::<_, diesel::result::Error, _>(|| {
            use db::schema::career_change_supporter_schema::advisor_account_creation_request::dsl::{
                advisor_account_creation_request
            };
            let request = advisor_account_creation_request.find(info.id)
                .first::<db::model::advisor::AccountCreationRequestResult>(&conn)
                .expect("failed to get data");
            let request_clone = request.clone();

            let current_date_time = chrono::Utc::now();
            let addr_line2= request.address_line2.unwrap();
            let reason = params.reason.clone();
            let reject_data = db::model::administrator::AdvisorRegReqRejected {
                email_address: &request.email_address,
                last_name: &request.last_name,
                first_name: &request.first_name,
                last_name_furigana: &request.last_name_furigana,
                first_name_furigana: &request.first_name_furigana,
                telephone_number: &request.telephone_number,
                year_of_birth: request.year_of_birth,
                month_of_birth: request.month_of_birth,
                day_of_birth: request.day_of_birth,
                prefecture: &request.prefecture,
                city: &request.city,
                address_line1: &request.address_line1,
                address_line2: Some(&addr_line2),
                reject_reason: &reason,
                rejected_time: &current_date_time,
            };
            use db::schema::career_change_supporter_schema::advisor_reg_req_rejected;
            let _res = diesel::insert_into(advisor_reg_req_rejected::table)
                .values(&reject_data)
                .get_result::<db::model::administrator::AdvisorRegReqRejectedResult>(&conn)
                .expect("Failed to insert data");
            let _del_res = diesel::delete(advisor_account_creation_request.find(info.id)).execute(&conn).expect("Failed to delete req");
            Ok(request_clone)
        })
    })
    .await;

    let req = result.expect("Failed to get data");
    // TODO: async、awaitのせいでトランザクション中に画像削除ができない。トランザクション外での処理で問題ないか検討する
    let img1 = req.image1;
    let _result = delete_image(img1).await.expect("Failed to delete data");
    let img2 = req.image2.unwrap();
    let _result = delete_image(img2).await.expect("Failed to delete data");
    let _result = send_notification_mail_to_advisor(&req.email_address);

    let _res = send_rejection_mail_to_advisor(&req.email_address, &reson_clone);

    let data = json!({
        "email_address": req.email_address,
    });
    let body = hb.render("advisor-registration-rejected", &data).unwrap();
    HttpResponse::Ok().body(body)
}

fn send_rejection_mail_to_advisor(
    email_address: &str,
    reason: &str,
) -> Result<(), lettre::error::Error> {
    use lettre::{ClientSecurity, SmtpClient, Transport};
    use lettre_email::EmailBuilder;
    let email = EmailBuilder::new()
        // TODO: ドメイン取得後書き直し
        .to(email_address)
        // TODO: 送信元メールを更新する
        .from("from@example.com")
        // TOOD: メールの件名を更新する
        .subject("アカウント作成拒絶")
        // TOOD: メールの本文を更新する (http -> httpsへの変更も含む)
        .text(format!(
            r"{}のアカウント作成が拒否されました。理由は下記のとおりです。
            {}",
            email_address, reason
        ))
        .build()
        .expect("Failed to build");

    use std::net::SocketAddr;
    let addr = SocketAddr::from(SMTP_SERVER_ADDR);
    let client = SmtpClient::new(addr, ClientSecurity::None).expect("Failed to create clietn");
    let mut mailer = client.transport();
    // TODO: メール送信後のレスポンスが必要か検討する
    let _ = mailer.send(email.into()).expect("Failed to send email");
    Ok(())
}

async fn delete_image(
    image_path: String,
) -> Result<rusoto_s3::DeleteObjectOutput, rusoto_core::RusotoError<rusoto_s3::DeleteObjectError>> {
    let delete_request = rusoto_s3::DeleteObjectRequest {
        bucket: AWS_S3_ID_IMG_BUCKET_NAME.to_string(),
        key: image_path.clone(),
        ..Default::default()
    };
    let region = rusoto_core::Region::Custom {
        name: AWS_REGION.to_string(),
        endpoint: AWS_ENDPOINT_URL.to_string(),
    };
    let s3_client = rusoto_s3::S3Client::new(region);
    s3_client.delete_object(delete_request).await
}

#[get("/advisor-registration-approval-list")]
async fn advisor_registration_approval_list(
    hb: web::Data<Handlebars<'_>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> HttpResponse {
    let conn = pool.get().expect("Failed to get connection");
    let result: Result<_, BlockingError<String>> = web::block(move || {
        use db::schema::career_change_supporter_schema::advisor_reg_req_approved::dsl::{
            advisor_reg_req_approved
        };
        let requests = advisor_reg_req_approved
            .limit(100)
            .load::<db::model::administrator::AdvisorRegReqApprovedResult>(&conn)
            .expect("failed to get data");
        Ok(requests)
    }).await;
    let mut requests = result.expect("Failed to get data");
    requests.sort_by(|a, b| b.approved_time.cmp(&a.approved_time));
    let mut data = Map::new();
    data.insert("num".to_string(), to_json(requests.len()));
    let mut items = Vec::new();
    // TODO: for in (Iterator) が順番通りに処理されることを確認
    for request in requests {
        let value = json!({
            "last_name": request.last_name,
            "first_name": request.first_name,
            "approved_time": request.approved_time,
            "id": request.advisor_reg_req_approved_id
        });
        items.push(value);
    }
    data.insert("items".to_string(), to_json(items));
    let body = hb.render("advisor-registration-approval-list", &data).unwrap();
    HttpResponse::Ok().body(body)
}

#[get("/advisor-registration-rejection-list")]
async fn advisor_registration_rejection_list(
    hb: web::Data<Handlebars<'_>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> HttpResponse {
    let conn = pool.get().expect("Failed to get connection");
    let result: Result<_, BlockingError<String>> = web::block(move || {
        use db::schema::career_change_supporter_schema::advisor_reg_req_rejected::dsl::{
            advisor_reg_req_rejected
        };
        let requests = advisor_reg_req_rejected
            .limit(100)
            .load::<db::model::administrator::AdvisorRegReqRejectedResult>(&conn)
            .expect("failed to get data");
        Ok(requests)
    }).await;
    let mut requests = result.expect("Failed to get data");
    requests.sort_by(|a, b| b.rejected_time.cmp(&a.rejected_time));
    let mut data = Map::new();
    data.insert("num".to_string(), to_json(requests.len()));
    let mut items = Vec::new();
    // TODO: for in (Iterator) が順番通りに処理されることを確認
    for request in requests {
        let value = json!({
            "last_name": request.last_name,
            "first_name": request.first_name,
            "rejected_time": request.rejected_time,
            "id": request.advisor_reg_req_rejected_id
        });
        items.push(value);
    }
    data.insert("items".to_string(), to_json(items));
    let body = hb.render("advisor-registration-rejection-list", &data).unwrap();
    HttpResponse::Ok().body(body)
}

#[get("/advisor-registration-rejection-detail")]
async fn advisor_registration_rejection_detail(
    web::Query(info): web::Query<DetailRequest>,
    hb: web::Data<Handlebars<'_>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> HttpResponse {
    let conn = pool.get().unwrap();
    // TODO: エラー処理の追加
    let result: Result<_, BlockingError<String>> = web::block(move || {
        use db::schema::career_change_supporter_schema::advisor_reg_req_rejected::dsl::{
            advisor_reg_req_rejected
        };
        let request = advisor_reg_req_rejected.find(info.id)
            .first::<db::model::administrator::AdvisorRegReqRejectedResult>(&conn)
            .expect("failed to get data");
        Ok(request)
    }).await;

    let request = result.unwrap();
    let address_line2_option: Option<String> = request.address_line2;
    let address_line2_exists = address_line2_option.is_some();
    let data = json!({
        //"id": request.advisor_reg_req_rejected_id,
        "rejected_time": request.rejected_time,
        "last_name": request.last_name,
        "first_name": request.first_name,
        "last_name_furigana": request.last_name_furigana,
        "first_name_furigana": request.first_name_furigana,
        "year": request.year_of_birth,
        "month": request.month_of_birth,
        "day": request.day_of_birth,
        "prefecture": request.prefecture,
        "city": request.city,
        "address_line1": request.address_line1,
        "address_line2_exists": address_line2_exists,
        "address_line2": address_line2_option.expect("Failed to get address line 2"),
        "email_address": request.email_address,
        "telephone_num": request.telephone_number,
        "reject_reason": request.reject_reason
    });

    let body = hb.render("advisor-registration-rejection-detail", &data).unwrap();
    HttpResponse::Ok().body(body)
}

#[get("/advisor-registration-approval-detail")]
async fn advisor_registration_approval_detail(
    web::Query(info): web::Query<DetailRequest>,
    hb: web::Data<Handlebars<'_>>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> HttpResponse {
    let conn = pool.get().unwrap();
    // TODO: エラー処理の追加
    let result: Result<_, BlockingError<String>> = web::block(move || {
        use db::schema::career_change_supporter_schema::advisor_reg_req_approved::dsl::{
            advisor_reg_req_approved
        };
        let request = advisor_reg_req_approved.find(info.id)
            .first::<db::model::administrator::AdvisorRegReqApprovedResult>(&conn)
            .expect("failed to get data");
        Ok(request)
    }).await;

    let request = result.unwrap();
    let address_line2_option: Option<String> = request.address_line2;
    let address_line2_exists = address_line2_option.is_some();
    let image2 = request.image2;
    let data = json!({
        "approved_time": request.approved_time,
        "last_name": request.last_name,
        "first_name": request.first_name,
        "last_name_furigana": request.last_name_furigana,
        "first_name_furigana": request.first_name_furigana,
        "year": request.year_of_birth,
        "month": request.month_of_birth,
        "day": request.day_of_birth,
        "prefecture": request.prefecture,
        "city": request.city,
        "address_line1": request.address_line1,
        "address_line2_exists": address_line2_exists,
        "address_line2": address_line2_option.expect("Failed to get address line 2"),
        "email_address": request.email_address,
        "telephone_num": request.telephone_number,
        "image1": request.image1,
        "image2": image2.expect("Failed to get data")
    });

    let body = hb.render("advisor-registration-approval-detail", &data).unwrap();
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
