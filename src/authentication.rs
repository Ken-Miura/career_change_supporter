// Copyright 2021 Ken Miura

use crate::common::credential;
use crate::common::database;
use crate::common::error;
use crate::common::error::Detail;
use crate::model;
use actix_session::Session;
use actix_web::{dev::Body, get, http::StatusCode, post, web, HttpResponse};
use diesel::prelude::*;

#[post("/login-request")]
pub(crate) async fn login_request(
    credential: web::Json<credential::Credential>,
    pool: web::Data<database::ConnectionPool>,
    session: Session,
) -> HttpResponse {
    let result = credential.validate();
    if let Err(e) = result {
        log::error!(
            "failed to authenticate \"{}\": {}",
            credential.email_address,
            e
        );
        return HttpResponse::build(StatusCode::BAD_REQUEST)
            .content_type("application/problem+json")
            .json(error::Error {
                code: e.code(),
                message: e.ui_message(),
            });
    }
    let result = pool.get();
    if let Err(e) = result {
        log::error!("failed to get connection: {}", e);
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type("application/problem+json")
            .json(error::Error {
                code: e.code(),
                message: e.ui_message(),
            });
    }
    let conn = result.expect("never happens panic");
    let mail_addr = credential.email_address.clone();
    let result = web::block(move || find_user_by_email_address(&mail_addr, &conn)).await;
    if let Err(_e) = result {
        // TODO: エラーハンドリング
        let code = error::code::AUTHENTICATION_FAILED;
        let message = String::from("メールアドレス、もしくはパスワードが間違っています。");
        return HttpResponse::build(StatusCode::UNAUTHORIZED)
            .content_type("application/problem+json")
            .json(error::Error { code, message });
    }
    let user = result.expect("never happens panic");
    let pwd = credential.password.clone();
    let result = credential::verify_password(&pwd, &user.hashed_password);
    if let Err(e) = result {
        return HttpResponse::build(StatusCode::UNAUTHORIZED)
            .content_type("application/problem+json")
            .json(error::Error {
                code: e.code(),
                message: e.ui_message(),
            });
    }
    let _ = session.set("email_address", &credential.email_address);
    // TODO: 最終ログイン日時の更新
    HttpResponse::with_body(StatusCode::OK, Body::Empty)
}

fn find_user_by_email_address(
    mail_addr: &str,
    conn: &PgConnection,
) -> Result<model::User, diesel::result::Error> {
    use crate::schema::my_project_schema::user::dsl::*;
    let users = user
        .filter(email_address.eq(mail_addr))
        .get_results::<model::User>(conn)?;
    // TODO: ユーザの数が0もしくは1であることのチェック
    let u = users[0].clone();
    Ok(u)
}

// Use POST for logout: https://stackoverflow.com/questions/3521290/logout-get-or-post
#[post("/logout-request")]
pub(crate) async fn logout_request(session: Session) -> HttpResponse {
    session.purge();
    HttpResponse::build(StatusCode::OK).finish()
}

#[get("/session-state")]
pub(crate) async fn session_state(session: Session) -> HttpResponse {
    // TODO: Handle Result
    let session_info: Option<String> = session.get("email_address").unwrap_or(None);
    if session_info == None {
        return HttpResponse::from_error(actix_web::error::ErrorUnauthorized(
            "failed to authenticate",
        ));
    }
    // set value to explicitly enhance ttl
    let _ = session.set("email_address", session_info.expect("msg: &str"));
    let contents = "contents";
    HttpResponse::Ok().body(contents)
}
