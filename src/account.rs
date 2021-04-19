// Copyright 2021 Ken Miura

use crate::common;
use crate::common::credential;
use crate::common::error;
use crate::common::error::handled;
use crate::common::error::unexpected;

use crate::model;
use actix_web::{get, post, web, HttpResponse};
use diesel::prelude::*;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// TODO: 運用しながら上限を調整する
const TEMPORARY_ACCOUNT_LIMIT: i64 = 7;
const UUID_REGEXP: &str = "^[a-zA-Z0-9]{32}$";

static UUID_RE: Lazy<Regex> = Lazy::new(|| Regex::new(UUID_REGEXP).expect("never happens panic"));

#[post("/temporary-account")]
pub(crate) async fn temporary_account(
    credential: web::Json<credential::Credential>,
    pool: web::Data<common::ConnectionPool>,
) -> Result<HttpResponse, error::Error> {
    let _ = credential.validate().map_err(|err| {
        let e = error::Error::Handled(err);
        log::error!("failed to create temporary account: {}", e);
        e
    })?;

    let conn = pool.get().map_err(|err| {
        let e = error::Error::Unexpected(unexpected::Error::R2d2Err(err));
        log::error!("failed to create temporary account: {}", e);
        e
    })?;

    let temp_acc_id = Uuid::new_v4().to_simple().to_string();
    let id_cloned = temp_acc_id.clone();
    let mail_addr = credential.email_address.clone();
    let hashed_pwd = credential::hash_password(&credential.password);
    let current_date_time = chrono::Utc::now();
    let result = web::block(move || {
        insert_temporary_account(id_cloned, mail_addr, hashed_pwd, current_date_time, &conn)
    })
    .await;
    let temporary_account_cnt = result.map_err(|err| {
        let e = error::Error::from(err);
        log::error!("failed to create temporary account: {}", e);
        e
    })?;

    let mut message = format!(
        "{}宛に登録用URLを送りました。登録用URLにアクセスし、登録を完了させてください（登録用URLの有効期間は24時間です）",
        credential.email_address
    );
    if temporary_account_cnt > 1 {
        message = format!(
            "{}。メールが届かない場合、迷惑メールフォルダに届いていないか、このサイトのドメインのメールが受信許可されているかをご確認ください。",
            message
        )
    }
    let _ = send_notification_mail(&credential.email_address, &temp_acc_id).map_err(|e| {
        log::error!("failed to create temporary account: {}", e);
        e
    })?;
    log::info!("created user temporary account successfully (temporary account id: {}, email address: {}) at {}", temp_acc_id, credential.email_address, current_date_time);
    Ok(HttpResponse::Ok().json(TemporaryAccountResult {
        email_address: credential.email_address.clone(),
        message,
    }))
}

fn insert_temporary_account(
    temp_acc_id: String,
    mail_addr: String,
    hashed_pwd: Vec<u8>,
    current_date_time: chrono::DateTime<chrono::Utc>,
    conn: &PgConnection,
) -> Result<i64, error::Error> {
    conn.transaction::<_, error::Error, _>(|| {
        check_if_account_exists(&mail_addr, conn)?;
        let cnt = num_of_temporary_accounts(&mail_addr, conn)?;
        if cnt >= TEMPORARY_ACCOUNT_LIMIT {
            return Err(error::Error::Handled(
                handled::Error::ReachLimitOfTemporaryAccount(
                    handled::ReachLimitOfTemporaryAccount::new(mail_addr, cnt),
                ),
            ));
        }
        use crate::schema::my_project_schema::user_temporary_account;
        let temp_acc = model::TemporaryAccount {
            user_temporary_account_id: &temp_acc_id,
            email_address: &mail_addr,
            hashed_password: &hashed_pwd,
            created_at: &current_date_time,
        };
        // TODO: 戻り値（usize: the number of rows affected）を利用する必要があるか検討する
        let _result = diesel::insert_into(user_temporary_account::table)
            .values(temp_acc)
            .execute(conn)
            .map_err(|e| error::Error::Unexpected(unexpected::Error::DieselResultErr(e)))?;
        Ok(cnt)
    })
}

fn check_if_account_exists(mail_addr: &str, conn: &PgConnection) -> Result<(), error::Error> {
    use crate::schema::my_project_schema::user_account::dsl::*;
    let cnt = user_account
        .filter(email_address.eq(mail_addr))
        .count()
        .get_result::<i64>(conn)
        .map_err(|e| error::Error::Unexpected(unexpected::Error::DieselResultErr(e)))?;
    if cnt > 1 {
        return Err(error::Error::Unexpected(
            unexpected::Error::AccountDuplicate(unexpected::AccountDuplicate::new(
                mail_addr.to_string(),
            )),
        ));
    }
    if cnt == 1 {
        return Err(error::Error::Handled(handled::Error::AccountAlreadyExists(
            handled::AccountAlreadyExists::new(mail_addr.to_string()),
        )));
    }
    // TODO: 念の為、負の数のケースを考える必要があるか調べる
    Ok(())
}

fn num_of_temporary_accounts(mail_addr: &str, conn: &PgConnection) -> Result<i64, error::Error> {
    use crate::schema::my_project_schema::user_temporary_account::dsl::{
        email_address, user_temporary_account,
    };
    let cnt = user_temporary_account
        .filter(email_address.eq(mail_addr))
        .count()
        .get_result::<i64>(conn)
        .map_err(|e| error::Error::Unexpected(unexpected::Error::DieselResultErr(e)))?;
    Ok(cnt)
}

fn send_notification_mail(
    email_address: &str,
    temporary_account_id: &str,
) -> Result<(), error::Error> {
    use lettre::{ClientSecurity, SmtpClient, Transport};
    use lettre_email::EmailBuilder;
    let email = EmailBuilder::new()
        .to(email_address)
        // TODO: 送信元メールを更新する
        .from("from@example.com")
        // TOOD: メールの件名を更新する
        .subject("アカウント登録依頼メール")
        // TOOD: メールの本文を更新する (http -> httpsへの変更も含む)
        .text(format!(
            r"下記のURLにアクセスし、登録を完了させてください（URLの有効期間は24時間です）
            http://{}:{}/temporary-accounts?id={}",
            common::DOMAIN,
            common::PORT,
            temporary_account_id
        ))
        .build()
        .map_err(|e| {
            error::Error::Unexpected(common::error::unexpected::Error::LettreEmailErr(e))
        })?;

    use std::net::SocketAddr;
    let addr = SocketAddr::from(common::SMTP_SERVER_ADDR);
    let client = SmtpClient::new(addr, ClientSecurity::None).map_err(|e| {
        error::Error::Unexpected(common::error::unexpected::Error::LettreSmtpErr(e))
    })?;
    let mut mailer = client.transport();
    // TODO: メール送信後のレスポンスが必要か検討する
    let _ = mailer.send(email.into()).map_err(|e| {
        error::Error::Unexpected(common::error::unexpected::Error::LettreSmtpErr(e))
    })?;
    Ok(())
}

#[derive(Serialize)]
struct TemporaryAccountResult {
    email_address: String,
    message: String,
}

// TODO: SameSite=Strictで問題ないか（アクセスできるか）確認する
#[get("/temporary-accounts")]
pub(crate) async fn temporary_accounts(
    web::Query(account_req): web::Query<AccountRequest>,
    pool: web::Data<common::ConnectionPool>,
) -> Result<HttpResponse, error::Error> {
    let _ = validate_id_format(&account_req.id).map_err(|e| {
        log::error!("failed to create account: {}", e);
        e
    })?;

    let conn = pool.get().map_err(|err| {
        let e = error::Error::Unexpected(unexpected::Error::R2d2Err(err));
        log::error!("failed to create account: {}", e);
        e
    })?;

    let current_date_time = chrono::Utc::now();
    let temp_acc_id = account_req.id.clone();
    let result = web::block(move || {
        check_temp_acc_and_create_account(&temp_acc_id, current_date_time, &conn)
    })
    .await;
    let user_acc = result.map_err(|err| {
        let e = error::Error::from(err);
        log::error!("failed to create account: {}", e);
        e
    })?;

    // NOTE: アカウント作成には成功しているので、ログに記録するだけでエラーを無視している
    // TODO: ログに記録するだけで、成功として返して良いか検討する
    let _ = send_account_creation_success_mail(&user_acc.email_address).map_err(|e| {
        log::warn!("failed to create account: {}", e);
        e
    });
    log::info!(
        "created user account successfully (user account id: {}, email address: {})",
        user_acc.user_account_id,
        user_acc.email_address
    );
    let message =
        r#"登録に成功しました。<a href="/login">こちら</a>よりログインを行ってください。"#
            .to_string();
    Ok(HttpResponse::Ok().json(AccountResult { message }))
}

#[derive(Deserialize)]
pub(crate) struct AccountRequest {
    id: String,
}

fn validate_id_format(temp_acc_id: &str) -> Result<(), error::Error> {
    if !UUID_RE.is_match(temp_acc_id) {
        let e = error::Error::Handled(handled::Error::InvalidTemporaryAccountId(
            handled::InvalidTemporaryAccountId::new(temp_acc_id.to_string()),
        ));
        return Err(e);
    }
    Ok(())
}

fn check_temp_acc_and_create_account(
    temporary_account_id: &str,
    current_date_time: chrono::DateTime<chrono::Utc>,
    conn: &PgConnection,
) -> Result<model::AccountQueryResult, error::Error> {
    conn.transaction::<_, error::Error, _>(|| {
        let temp_acc = find_temporary_account_by_id(temporary_account_id, conn)?;
        let _ = delete_temporary_account(temporary_account_id, conn)?;
        let time_elapsed = current_date_time - temp_acc.created_at;
        if time_elapsed.num_days() > 0 {
            let e = handled::TemporaryAccountExpired::new(
                temporary_account_id.to_string(),
                temp_acc.created_at,
                current_date_time,
            );
            return Err(error::Error::Handled(
                handled::Error::TemporaryAccountExpired(e),
            ));
        }
        // NOTE: 関数内でtransactionを利用しているため、この点でSAVEPOINTとなる
        // TODO: transacstionの中で、transacstionを利用して問題がないか確認する
        let user = create_account(&temp_acc.email_address, &temp_acc.hashed_password, conn)?;
        Ok(user)
    })
}

fn find_temporary_account_by_id(
    temp_acc_id: &str,
    conn: &PgConnection,
) -> Result<model::TemporaryAccountQueryResult, error::Error> {
    use crate::schema::my_project_schema::user_temporary_account::dsl::*;
    let users = user_temporary_account
        .filter(user_temporary_account_id.eq(temp_acc_id))
        .get_results::<model::TemporaryAccountQueryResult>(conn)?;
    if users.is_empty() {
        let e = handled::NoTemporaryAccountFound::new(temp_acc_id.to_string());
        return Err(error::Error::Handled(
            handled::Error::NoTemporaryAccountFound(e),
        ));
    }
    if users.len() != 1 {
        let e = unexpected::TemporaryAccountIdDuplicate::new(temp_acc_id.to_string());
        return Err(error::Error::Unexpected(
            unexpected::Error::TemporaryAccountIdDuplicate(e),
        ));
    }
    let user = users[0].clone();
    Ok(user)
}

fn send_account_creation_success_mail(email_address: &str) -> Result<(), error::Error> {
    use lettre::{ClientSecurity, SmtpClient, Transport};
    use lettre_email::EmailBuilder;
    let email = EmailBuilder::new()
        .to(email_address)
        // TODO: 送信元メールを更新する
        .from("from@example.com")
        // TOOD: メールの件名を更新する
        .subject("アカウント登録完了")
        // TOOD: メールの本文を更新する
        .text("アカウントの登録が完了しました。")
        .build()
        .map_err(|e| {
            error::Error::Unexpected(common::error::unexpected::Error::LettreEmailErr(e))
        })?;

    use std::net::SocketAddr;
    let addr = SocketAddr::from(common::SMTP_SERVER_ADDR);
    let client = SmtpClient::new(addr, ClientSecurity::None).map_err(|e| {
        error::Error::Unexpected(common::error::unexpected::Error::LettreSmtpErr(e))
    })?;
    let mut mailer = client.transport();
    // TODO: メール送信後のレスポンスが必要か検討する
    let _ = mailer.send(email.into()).map_err(|e| {
        error::Error::Unexpected(common::error::unexpected::Error::LettreSmtpErr(e))
    })?;
    Ok(())
}

fn delete_temporary_account(temp_acc_id: &str, conn: &PgConnection) -> Result<(), error::Error> {
    use crate::schema::my_project_schema::user_temporary_account::dsl::{
        user_temporary_account, user_temporary_account_id,
    };
    // TODO: 戻り値 cnt（usize: the number of rows affected）を利用する必要があるか検討する
    let cnt =
        diesel::delete(user_temporary_account.filter(user_temporary_account_id.eq(temp_acc_id)))
            .execute(conn)?;
    if cnt != 1 {
        log::warn!(
            "diesel::delete::execute result (id: {}): {}",
            temp_acc_id,
            cnt
        );
    }
    Ok(())
}

fn create_account(
    mail_addr: &str,
    hashed_pwd: &[u8],
    conn: &PgConnection,
) -> Result<model::AccountQueryResult, error::Error> {
    conn.transaction::<_, error::Error, _>(|| {
        use crate::schema::my_project_schema::user_account::dsl::{email_address, user_account};
        let cnt = user_account
            .filter(email_address.eq(mail_addr))
            .count()
            .get_result::<i64>(conn)?;
        if cnt > 0 {
            let e = unexpected::AccountDuplicate::new(mail_addr.to_string());
            return Err(error::Error::Unexpected(
                unexpected::Error::AccountDuplicate(e),
            ));
        }
        use crate::schema::my_project_schema::user_account as user_acc;
        let user = model::Account {
            email_address: mail_addr,
            hashed_password: hashed_pwd,
            last_login_time: None,
        };
        let users = diesel::insert_into(user_acc::table)
            .values(&user)
            .get_results::<model::AccountQueryResult>(conn)?;
        if users.len() > 1 {
            return Err(error::Error::Unexpected(
                unexpected::Error::AccountDuplicate(unexpected::AccountDuplicate::new(
                    mail_addr.to_string(),
                )),
            ));
        }
        let user = users[0].clone();
        Ok(user)
    })
}

#[derive(Serialize)]
struct AccountResult {
    message: String,
}

// fn create_invalid_id_format_view() -> HttpResponse {
//     let body = r#"<!DOCTYPE html>
//     <html>
//       <head>
//         <meta charset="utf-8">
//         <title>不正なリクエスト</title>
//       </head>
//       <body>
//         不正なURLです。ブラウザに入力されているURLと、メール本文に記載されているURLが間違っていないかご確認ください。
//       </body>
//     </html>"#
//         .to_string();
//     HttpResponse::build(StatusCode::BAD_REQUEST)
//         .content_type("text/html; charset=UTF-8")
//         .body(body)
// }

// fn create_internal_error_view() -> HttpResponse {
//     let body = format!(
//         r#"<!DOCTYPE html>
//     <html>
//       <head>
//         <meta charset="utf-8">
//         <title><サーバエラー/title>
//       </head>
//       <body>
//       {} (エラーコード: {})
//       </body>
//     </html>"#,
//         common::error::INTERNAL_SERVER_ERROR_MESSAGE,
//         error::code::INTERNAL_SERVER_ERROR
//     );
//     return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
//         .content_type("text/html; charset=UTF-8")
//         .body(body);
// }

// fn create_error_view(err: AccountCreationError) -> HttpResponse {
//     let body = format!(
//         r#"<!DOCTYPE html>
//     <html>
//       <head>
//         <meta charset="utf-8">
//         <title>登録失敗</title>
//       </head>
//       <body>
//       {}
//       </body>
//     </html>"#,
//         err.to_message()
//     );
//     HttpResponse::build(StatusCode::BAD_REQUEST)
//         .content_type("text/html; charset=UTF-8")
//         .body(body)
// }

// fn create_success_view() -> HttpResponse {
//     let body = r#"<!DOCTYPE html>
//     <html>
//       <head>
//         <meta charset="utf-8">
//         <title>登録失敗</title>
//       </head>
//       <body>
//       登録に成功しました。<a href="/login">こちら</a>よりログインを行ってください。
//       </body>
//     </html>"#;
//     HttpResponse::build(StatusCode::OK)
//         .content_type("text/html; charset=UTF-8")
//         .body(body)
// }
