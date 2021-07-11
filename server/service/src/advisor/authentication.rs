// Copyright 2021 Ken Miura

use crate::common;
use crate::common::credential;
use crate::common::error;
use crate::common::error::handled;
use crate::common::error::unexpected;
use actix_session::Session;
use actix_web::{get, http::StatusCode, post, web, HttpResponse};
use diesel::prelude::*;

const KEY_TO_ADVISOR_ACCOUNT_ID: &str = "advisor_account_id";

#[post("/login-request")]
async fn login_request(
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
    let advisor_account = result.map_err(|err| {
        let e = error::Error::from(err);
        log::error!("failed to login: {}", e);
        e
    })?;

    let pwd = credential.password.clone();
    let _ = credential::verify_password(&pwd, &advisor_account.hashed_password).map_err(|e| {
        log::error!("failed to login: {}", e);
        e
    })?;

    let advisor_acc_id = advisor_account.advisor_account_id;
    let current_date_time = chrono::Utc::now();
    let conn = pool.get().map_err(|err| {
        let e = error::Error::Unexpected(unexpected::Error::R2d2Err(err));
        log::error!("failed to login: {}", e);
        e
    })?;
    let result = web::block(move || {
        update_last_login_time(
            advisor_account.advisor_account_id,
            &current_date_time,
            &conn,
        )
    })
    .await;
    let _ = result.map_err(|err| {
        let e = error::Error::from(err);
        log::error!("failed to login: {}", e);
        e
    })?;

    let _ = session
        .set(KEY_TO_ADVISOR_ACCOUNT_ID, advisor_acc_id)
        .map_err(|err| {
            let e = error::Error::Unexpected(unexpected::Error::ActixWebErr(err.to_string()));
            log::error!("failed to login: {}", e);
            e
        })?;
    // TODO: session.set()の後にsession.renew() が必要かどうか確認する
    log::info!(
        "advisor (advisor account id: {}) logged in successfully",
        advisor_acc_id
    );
    Ok(HttpResponse::build(StatusCode::OK).finish())
}

fn find_user_by_email_address(
    mail_addr: &str,
    conn: &PgConnection,
) -> Result<db::model::advisor::AccountQueryResult, error::Error> {
    use db::schema::career_change_supporter_schema::advisor_account::dsl::*;
    let advisors = advisor_account
        .filter(email_address.eq(mail_addr))
        .get_results::<db::model::advisor::AccountQueryResult>(conn)
        .map_err(|e| error::Error::Unexpected(unexpected::Error::DieselResultErr(e)))?;
    if advisors.len() > 1 {
        let e = unexpected::UserAccountDuplicate::new(mail_addr.to_string());
        return Err(error::Error::Unexpected(
            unexpected::Error::UserAccountDuplicate(e),
        ));
    }
    if advisors.is_empty() {
        let e = handled::NoAccountFound::new(mail_addr.to_string());
        return Err(error::Error::Handled(handled::Error::NoAccountFound(e)));
    }
    let u = advisors[0].clone();
    Ok(u)
}

fn update_last_login_time(
    advisor_acc_id: i32,
    current_date_time: &chrono::DateTime<chrono::Utc>,
    conn: &PgConnection,
) -> Result<(), error::Error> {
    use db::schema::career_change_supporter_schema::advisor_account::dsl::{
        advisor_account, last_login_time,
    };
    let affected_useraccounts = diesel::update(advisor_account.find(advisor_acc_id))
        .set(last_login_time.eq(Some(current_date_time)))
        .get_results::<db::model::advisor::AccountQueryResult>(conn)
        .map_err(|e| error::Error::Unexpected(unexpected::Error::DieselResultErr(e)))?;

    // NOTE: findはプライマリキーを用いた検索を行うため、影響される数は0か1しかない。そのため、affected_useraccounts.len() > 1 のケースはチェックしない
    // NOTE: メールアドレスを見つけ、パスワードの一致を確認後、最終ログイン時間を更新する前にアカウントの削除処理が走った場合に発生する可能性がある
    // TODO: 人の手で起こるようなケースはありえないので、運用の結果、発生が見られなければ削除する
    if affected_useraccounts.is_empty() {
        let e = unexpected::FailedToUpdateAccount::new(advisor_acc_id);
        return Err(error::Error::Unexpected(
            unexpected::Error::FailedToUpdateAccount(e),
        ));
    }
    Ok(())
}

// Use POST for logout: https://stackoverflow.com/questions/3521290/logout-get-or-post
#[post("/logout-request")]
async fn logout_request(session: Session) -> Result<HttpResponse, error::Error> {
    let option_advisor_acc_id: Option<i32> =
        session.get(KEY_TO_ADVISOR_ACCOUNT_ID).map_err(|err| {
            let e = error::Error::Unexpected(unexpected::Error::ActixWebErr(err.to_string()));
            log::error!("failed to logout {}", e);
            e
        })?;
    session.purge();
    if let Some(advisor_acc_id) = option_advisor_acc_id {
        log::info!(
            "advisor (advisor account id ({}) logged out successfully",
            advisor_acc_id
        );
    } else {
        log::info!("somebody logged out successfully");
    }
    Ok(HttpResponse::build(StatusCode::OK).finish())
}

#[get("/session-state")]
async fn session_state(session: Session) -> Result<HttpResponse, error::Error> {
    let option_advisor_acc_id = session_state_inner(&session)?;
    return match option_advisor_acc_id {
        Some(advisor_acc_id) => {
            // set value to explicitly enhance ttl
            // TODO: session.set() でなく、session.renew() を利用すべきか確認する
            // 参考: https://github.com/actix/actix-extras/blob/master/actix-redis/examples/authentication.rs
            let _ = session
                .set(KEY_TO_ADVISOR_ACCOUNT_ID, advisor_acc_id)
                .map_err(|err| {
                    let e =
                        error::Error::Unexpected(unexpected::Error::ActixWebErr(err.to_string()));
                    log::error!("failed to get session state: {}", e);
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

pub(in crate::advisor) fn session_state_inner(
    session: &Session,
) -> Result<Option<i32>, common::error::Error> {
    let option_advisor_acc_id: Option<i32> =
        session.get(KEY_TO_ADVISOR_ACCOUNT_ID).map_err(|err| {
            let e = error::Error::Unexpected(unexpected::Error::ActixWebErr(err.to_string()));
            log::error!("failed to get session state: {}", e);
            e
        })?;
    Ok(option_advisor_acc_id)
}
