// Copyright 2021 Ken Miura

use crate::common;
use crate::common::credential;
use crate::common::error;
use crate::common::error::ToCode;
use crate::common::error::ToMessage;
use crate::model;
use actix_session::Session;
use actix_web::{dev::Body, get, http::StatusCode, post, web, HttpResponse};
use diesel::prelude::*;

const KEY_TO_EMAIL: &str = "email_address";

#[post("/login-request")]
pub(crate) async fn login_request(
    credential: web::Json<credential::Credential>,
    pool: web::Data<common::ConnectionPool>,
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
                code: e.to_code(),
                message: e.to_message(),
            });
    }
    let result = pool.get();
    if let Err(e) = result {
        log::error!("failed to get connection: {}", e);
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type("application/problem+json")
            .json(error::Error {
                code: e.to_code(),
                message: e.to_message(),
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
                code: e.to_code(),
                message: e.to_message(),
            });
    }
    let _ = session.set(KEY_TO_EMAIL, &credential.email_address);
    // TODO: 最終ログイン日時の更新
    HttpResponse::with_body(StatusCode::OK, Body::Empty)
}

fn find_user_by_email_address(
    mail_addr: &str,
    conn: &PgConnection,
) -> Result<model::AccountQueryResult, diesel::result::Error> {
    use crate::schema::my_project_schema::user_account::dsl::*;
    let users = user_account
        .filter(email_address.eq(mail_addr))
        .get_results::<model::AccountQueryResult>(conn)?;
    // TODO: ユーザの数が0もしくは1であることのチェック
    let u = users[0].clone();
    Ok(u)
}

// Use POST for logout: https://stackoverflow.com/questions/3521290/logout-get-or-post
#[post("/logout-request")]
pub(crate) async fn logout_request(session: Session) -> HttpResponse {
    let result: Result<Option<String>, _> = session.get(KEY_TO_EMAIL);
    if let Err(e) = result {
        log::error!("failed to get session: {}", e);
        // TODO: そのままレスポンスとして返却してよいのか確認する
        return e.into();
    }
    let session_info = result.expect("never happens panic");
    if let Some(email) = session_info {
        log::info!("\"{}\" requested logout", email);
    } else {
        log::info!("somebody requested logout");
    }
    session.purge();
    HttpResponse::build(StatusCode::OK).finish()
}

#[get("/session-state")]
pub(crate) async fn session_state(session: Session) -> HttpResponse {
    // TODO: Handle Result
    let session_info: Option<String> = session.get(KEY_TO_EMAIL).unwrap_or(None);
    if session_info == None {
        return HttpResponse::from_error(actix_web::error::ErrorUnauthorized(
            "failed to authenticate",
        ));
    }
    let value = session_info.expect("never happens panic");
    // set value to explicitly enhance ttl
    let _ = session.set(KEY_TO_EMAIL, value);
    HttpResponse::build(StatusCode::OK).finish()
}
