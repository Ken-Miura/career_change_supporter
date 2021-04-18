// Copyright 2021 Ken Miura

use crate::common;
use crate::common::credential;
use crate::common::error;
use crate::common::error::handled;
use crate::common::error::unexpected;
use crate::model;
use actix_session::Session;
use actix_web::{get, http::StatusCode, post, web, HttpResponse};
use diesel::prelude::*;

const KEY_TO_EMAIL: &str = "email_address";

#[post("/login-request")]
pub(crate) async fn login_request(
    credential: web::Json<credential::Credential>,
    pool: web::Data<common::ConnectionPool>,
    session: Session,
) -> Result<HttpResponse, error::Error> {
    let _ = credential.validate().map_err(|err| {
        let e = error::Error::Handled(err);
        log::error!("failed to login: {}", e);
        e
    })?;

    let conn = pool.get().map_err(|err| {
        let e = error::Error::Unexpected(unexpected::Error::R2d2Err(err));
        log::error!("failed to login: {}", e);
        e
    })?;

    let mail_addr = credential.email_address.clone();
    let result = web::block(move || find_user_by_email_address(&mail_addr, &conn)).await;
    let user_account = result.map_err(|err| {
        let e = error::Error::from(err);
        log::error!("failed to login: {}", e);
        e
    })?;

    let pwd = credential.password.clone();
    let _ = credential::verify_password(&pwd, &user_account.hashed_password).map_err(|err| {
        let e = error::Error::Handled(err);
        log::error!("failed to login: {}", e);
        e
    })?;

    let primary_key = user_account.id;
    let current_date_time = chrono::Utc::now();
    let conn = pool.get().map_err(|err| {
        let e = error::Error::Unexpected(unexpected::Error::R2d2Err(err));
        log::error!("failed to login: {}", e);
        e
    })?;
    let result =
        web::block(move || update_last_login_time(primary_key, &current_date_time, &conn)).await;
    let _ = result.map_err(|err| {
        let e = error::Error::from(err);
        log::error!("failed to login: {}", e);
        e
    })?;

    let _ = session
        .set(KEY_TO_EMAIL, &credential.email_address)
        .map_err(|err| {
            let e = error::Error::Unexpected(unexpected::Error::ActixWebErr(err.to_string()));
            log::error!("failed to login: {}", e);
            e
        })?;
    Ok(HttpResponse::build(StatusCode::OK).finish())
}

fn find_user_by_email_address(
    mail_addr: &str,
    conn: &PgConnection,
) -> Result<model::AccountQueryResult, error::Error> {
    use crate::schema::my_project_schema::user_account::dsl::*;
    let users = user_account
        .filter(email_address.eq(mail_addr))
        .get_results::<model::AccountQueryResult>(conn)
        .map_err(|e| error::Error::Unexpected(unexpected::Error::DieselResultErr(e)))?;
    if users.len() > 1 {
        let e = unexpected::AccountDuplicate::new(mail_addr.to_string());
        return Err(error::Error::Unexpected(
            unexpected::Error::AccountDuplicate(e),
        ));
    }
    if users.is_empty() {
        let e = handled::NoAccountFound::new(mail_addr.to_string());
        return Err(error::Error::Handled(handled::Error::NoAccountFound(e)));
    }
    let u = users[0].clone();
    Ok(u)
}

fn update_last_login_time(
    primary_key: i32,
    current_date_time: &chrono::DateTime<chrono::Utc>,
    conn: &PgConnection,
) -> Result<(), error::Error> {
    use crate::schema::my_project_schema::user_account::dsl::{last_login_time, user_account};
    let affected_useraccounts = diesel::update(user_account.find(primary_key))
        .set(last_login_time.eq(Some(current_date_time)))
        .get_results::<model::AccountQueryResult>(conn)
        .map_err(|e| error::Error::Unexpected(unexpected::Error::DieselResultErr(e)))?;

    // NOTE: findはプライマリキーを用いた検索を行うため、影響される数は0か1しかない。そのため、affected_useraccounts.len() > 1 のケースはチェックしない
    // NOTE: メールアドレスを見つけ、パスワードの一致を確認後、最終ログイン時間を更新する前にアカウントの削除処理が走った場合に発生する可能性がある
    // TODO: 人の手で起こるようなケースはありえないので、運用の結果、発生が見られなければ削除する
    if affected_useraccounts.is_empty() {
        let e = unexpected::FailedToUpdateAccount::new(primary_key);
        return Err(error::Error::Unexpected(
            unexpected::Error::FailedToUpdateAccount(e),
        ));
    }
    Ok(())
}

// Use POST for logout: https://stackoverflow.com/questions/3521290/logout-get-or-post
#[post("/logout-request")]
pub(crate) async fn logout_request(session: Session) -> Result<HttpResponse, error::Error> {
    let option_email_address: Option<String> = session.get(KEY_TO_EMAIL).map_err(|err| {
        let e = error::Error::Unexpected(unexpected::Error::ActixWebErr(err.to_string()));
        log::error!("failed to logout {}", e);
        e
    })?;
    if let Some(email_address) = option_email_address {
        log::info!("{} requested logout", email_address);
    } else {
        log::info!("somebody requested logout");
    }
    session.purge();
    Ok(HttpResponse::build(StatusCode::OK).finish())
}

#[get("/session-state")]
pub(crate) async fn session_state(session: Session) -> Result<HttpResponse, error::Error> {
    let option_email_address: Option<String> = session.get(KEY_TO_EMAIL).map_err(|err| {
        let e = error::Error::Unexpected(unexpected::Error::ActixWebErr(err.to_string()));
        log::error!("failed to get session state {}", e);
        e
    })?;
    return match option_email_address {
        Some(email_address) => {
            // set value to explicitly enhance ttl
            let _ = session.set(KEY_TO_EMAIL, email_address).map_err(|err| {
                let e = error::Error::Unexpected(unexpected::Error::ActixWebErr(err.to_string()));
                log::error!("failed to get session state {}", e);
                e
            })?;
            Ok(HttpResponse::build(StatusCode::OK).finish())
        }
        None => {
            let e = error::Error::Handled(handled::Error::NoSessionFound(
                handled::NoSessionFound::new(),
            ));
            log::error!("failed to get session state {}", e);
            Err(e)
        }
    };
}
