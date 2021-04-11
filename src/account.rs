// Copyright 2021 Ken Miura

// 用語の統一
// 仮登録 => temporary account creation
// 仮ユーザ => temporary account
// 登録 => account creation
// ユーザ => account

// URL
// post temporary accounts
// get temporary accounts?id=xxxx

// 上記の用語で統一して書き直す。

use crate::common::credential;
use crate::common::database;
use crate::common::error;
use crate::common::error::Detail;

use crate::model;
use actix_web::{get, http::StatusCode, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// TODO: 運用しながら上限を調整する
const LIMIT_OF_REGISTRATION_RECORD: i64 = 7;
const UUID_REGEXP: &str = "^[a-zA-Z0-9]{32}$";

static UUID_RE: Lazy<Regex> = Lazy::new(|| Regex::new(UUID_REGEXP).expect("never happens panic"));

#[post("/temporary-accounts")]
pub(crate) async fn temporary_accounts(
    credential: web::Json<credential::Credential>,
    pool: web::Data<database::ConnectionPool>,
) -> HttpResponse {
    let result = credential.validate();
    if let Err(e) = result {
        log::error!(
            "failed to register \"{}\" (error code: {}): {}",
            credential.email_address,
            e.code(),
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
        log::error!("failed to get connection (error code: {}): {}", e.code(), e);
        return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type("application/problem+json")
            .json(error::Error {
                code: e.code(),
                message: e.ui_message(),
            });
    }
    let conn = result.expect("never happens panic");
    let mail_addr = credential.email_address.clone();
    let hashed_password = credential::hash_password(&credential.password);
    let query_id = Uuid::new_v4().to_simple().to_string();
    let query_id_to_register = query_id.clone();
    let current_date_time = Utc::now();
    let result = web::block(move || {
        register_tentative_user(
            &mail_addr,
            &hashed_password,
            &query_id_to_register,
            &current_date_time,
            &conn,
        )
    })
    .await;

    if let Err(err) = result {
        match err {
            actix_web::error::BlockingError::Error(e) => {
                log::error!(
                    "failed to register \"{}\" tentatively: {}",
                    credential.email_address,
                    e
                );
                return create_registration_err_response(e);
            }
            actix_web::error::BlockingError::Canceled => {
                log::error!("failed to register tentatively: error::BlockingError::Canceled");
                return create_execution_canceled_response();
            }
        }
    }
    let num_of_tentative_accounts = result.expect("never happens panic");
    let notification =  format!(
        "{}宛に登録用URLを送りました。登録用URLにアクセスし、登録を完了させてください（登録用URLの有効期間は24時間です）",
        credential.email_address.clone()
    );
    let message = if num_of_tentative_accounts > 0 {
        format!(
            "{}。メールが届かない場合、迷惑メールフォルダに届いていないか、このサイトのドメインのメールが受信許可されているかをご確認ください。",
            notification
        )
    } else {
        notification
    };
    let result = send_activation_mail(&credential.email_address, &query_id);
    if let Err(err) = result {
        log::error!("failed to send email: {}", err);
        return create_registration_err_response(err);
    }
    HttpResponse::Ok().json(TentativeRegistrationResult {
        email_address: credential.email_address.clone(),
        message,
    })
}

#[derive(Debug)]
enum RegistrationError {
    AlreadyExist { email_address: String },
    Duplicate { email_address: String, count: i64 },
    ExceedRegistrationLimit { email_address: String, count: i64 },
    DieselError(diesel::result::Error),
    EmailError(lettre_email::error::Error),
    SmtpError(lettre::smtp::error::Error),
}

impl fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RegistrationError::AlreadyExist { email_address } => {
                write!(f, "{} has already existed in user", email_address)
            }
            RegistrationError::Duplicate {
                email_address,
                count,
            } => {
                write!(
                    f,
                    "fatal error (Something is wrong!!): found \"{}\" {} times",
                    email_address, count
                )
            }
            RegistrationError::ExceedRegistrationLimit {
                email_address,
                count,
            } => {
                write!(
                    f,
                    "exceed regstration limit (email: \"{}\", registration count: {})",
                    email_address, count
                )
            }
            RegistrationError::DieselError(e) => {
                write!(f, "diesel error: {}", e)
            }
            RegistrationError::EmailError(e) => {
                write!(f, "lettre email error: {}", e)
            }
            RegistrationError::SmtpError(e) => {
                write!(f, "lettre smtp error: {}", e)
            }
        }
    }
}

impl From<diesel::result::Error> for RegistrationError {
    fn from(error: diesel::result::Error) -> Self {
        RegistrationError::DieselError(error)
    }
}

impl From<lettre_email::error::Error> for RegistrationError {
    fn from(error: lettre_email::error::Error) -> Self {
        RegistrationError::EmailError(error)
    }
}

impl From<lettre::smtp::error::Error> for RegistrationError {
    fn from(error: lettre::smtp::error::Error) -> Self {
        RegistrationError::SmtpError(error)
    }
}

#[derive(Deserialize)]
pub(crate) struct EntryRequest {
    id: String,
}

enum IdValidationError {
    InvalidId(String),
}

impl fmt::Display for IdValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IdValidationError::InvalidId(id) => {
                write!(f, "invalid id: {}", id)
            }
        }
    }
}

// TODO: SameSite=Strictで問題ないか（アクセスできるか）確認する
#[get("/entry")]
pub(crate) async fn entry(
    web::Query(entry): web::Query<EntryRequest>,
    pool: web::Data<database::ConnectionPool>,
) -> HttpResponse {
    let result = validate_id(&entry.id);
    if let Err(e) = result {
        log::error!("failed to get entry: {}", e);
        return create_invalid_id_view();
    }
    let result = pool.get();
    if let Err(e) = result {
        log::error!("failed to get connection: {}", e);
        return create_db_connection_error_view();
    }
    let conn = result.expect("never happens panic");
    let current_date_time = Utc::now();
    let result = web::block(move || create_user(&entry.id, current_date_time, &conn)).await;
    if let Err(_e) = result {
        return create_error_view();
    }
    // TODO: アカウント作成成功メールを送る
    create_success_view()
}

fn create_error_view() -> HttpResponse {
    let body = r#"<!DOCTYPE html>
    <html>
      <head>
        <meta charset="utf-8">
        <title>登録失敗</title>
      </head>
      <body>
      登録に失敗しました。
      </body>
    </html>"#
        .to_string();
    HttpResponse::build(StatusCode::BAD_REQUEST)
        .content_type("text/html; charset=UTF-8")
        .body(body)
}

fn create_success_view() -> HttpResponse {
    let body = r#"<!DOCTYPE html>
    <html>
      <head>
        <meta charset="utf-8">
        <title>登録失敗</title>
      </head>
      <body>
      登録に成功しました。<a href="/login">こちら</a>よりログインを行ってください。
      </body>
    </html>"#;
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=UTF-8")
        .body(body)
}

fn create_user(
    query_id: &str,
    current_date_time: DateTime<Utc>,
    conn: &PgConnection,
) -> Result<(), EntryError> {
    conn.transaction::<_, EntryError, _>(|| {
        let result = find_tentative_user_by_id(query_id, conn)?;
        if result.is_empty() {
            return Err(EntryError::NoEntry(query_id.to_string()));
        }
        if result.len() != 1 {
            return Err(EntryError::EntryDuplicate(query_id.to_string()));
        }
        let deletion_result = delete_entry(query_id, conn);
        if let Err(e) = deletion_result {
            log::error!("failed to delete entry: {}", e);
        };
        let user = &result[0];
        let time_passed = current_date_time - user.registration_time;
        if time_passed.num_days() > 0 {
            return Err(EntryError::EntryExpire {
                registration_time: user.registration_time,
                activation_time: current_date_time,
            });
        }
        let user = &result[0];
        create_account(&user.email_address, user.hashed_password.as_ref(), conn)?;
        Ok(())
    })
}

fn delete_entry(entry_id: &str, conn: &PgConnection) -> Result<(), EntryError> {
    use crate::schema::my_project_schema::tentative_user::dsl::*;
    // TODO: 戻り値（usize: the number of rows affected）を利用する必要があるか検討する
    let _result = diesel::delete(tentative_user.filter(query_id.eq(entry_id))).execute(conn)?;
    Ok(())
}

fn create_account(
    mail_addr: &str,
    hashed_pwd: &[u8],
    conn: &PgConnection,
) -> Result<(), EntryError> {
    use crate::schema::my_project_schema::user;
    let user = model::AccountInfo {
        email_address: mail_addr,
        hashed_password: hashed_pwd,
        last_login_time: None,
    };
    // TODO: 既に登録されているケースをエラーハンドリングする
    // TODO: 戻り値（usize: the number of rows affected）を利用する必要があるか検討する
    let _result = diesel::insert_into(user::table)
        .values(&user)
        .execute(conn)?;
    Ok(())
}

#[derive(Debug)]
enum EntryError {
    DieselError(diesel::result::Error),
    NoEntry(String),
    EntryDuplicate(String),
    EntryExpire {
        registration_time: DateTime<Utc>,
        activation_time: DateTime<Utc>,
    },
}

impl fmt::Display for EntryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EntryError::DieselError(e) => {
                write!(f, "diesel error: {}", e)
            }
            EntryError::NoEntry(entry_id) => {
                write!(f, "not found entry: {}", entry_id)
            }
            EntryError::EntryDuplicate(entry_id) => {
                write!(f, "duplicate entry: {}", entry_id)
            }
            EntryError::EntryExpire {
                registration_time,
                activation_time,
            } => {
                write!(
                    f,
                    "entry already expired (registration time: {}, activation time: {})",
                    registration_time, activation_time
                )
            }
        }
    }
}

impl From<diesel::result::Error> for EntryError {
    fn from(error: diesel::result::Error) -> Self {
        EntryError::DieselError(error)
    }
}

fn find_tentative_user_by_id(
    entry_id: &str,
    conn: &PgConnection,
) -> Result<Vec<model::TentativeUser>, EntryError> {
    use crate::schema::my_project_schema::tentative_user::dsl::*;
    let users = tentative_user
        .filter(query_id.eq(entry_id))
        .get_results::<model::TentativeUser>(conn)?;
    Ok(users)
}

fn create_db_connection_error_view() -> HttpResponse {
    let body = format!(
        r#"<!DOCTYPE html>
    <html>
      <head>
        <meta charset="utf-8">
        <title><サーバエラー/title>
      </head>
      <body>
      {} (エラーコード: {})
      </body>
    </html>"#,
        "test", // TODO: メッセージを変更
        error::code::DB_CONNECTION_UNAVAILABLE
    );
    return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
        .content_type("text/html; charset=UTF-8")
        .body(body);
}

fn create_invalid_id_view() -> HttpResponse {
    let body = r#"<!DOCTYPE html>
    <html>
      <head>
        <meta charset="utf-8">
        <title>不正なリクエスト</title>
      </head>
      <body>
        不正なURLです。ブラウザに入力されているURLと、メール本文に記載されているURLが間違っていないかご確認ください。
      </body>
    </html>"#
        .to_string();
    HttpResponse::build(StatusCode::BAD_REQUEST)
        .content_type("text/html; charset=UTF-8")
        .body(body)
}

fn validate_id(id: &str) -> Result<(), IdValidationError> {
    if !UUID_RE.is_match(id) {
        return Err(IdValidationError::InvalidId(id.to_string()));
    }
    Ok(())
}

// TODO: 環境変数から読み込むように変更する
const SMTP_SERVER_ADDR: ([u8; 4], u16) = ([127, 0, 0, 1], 1025);
// TODO: サーバのドメイン名を変更し、共通で利用するmoduleへ移動する
const DOMAIN: &str = "localhost";
const PORT: &str = "8080";

fn send_activation_mail(email_address: &str, query_id: &str) -> Result<(), RegistrationError> {
    use lettre::{ClientSecurity, SmtpClient, Transport};
    use lettre_email::EmailBuilder;
    let email = EmailBuilder::new()
        .to(email_address)
        // TODO: 送信元メールを更新する
        .from("from@example.com")
        // TOOD: メールの件名を更新する
        .subject("日本語のタイトル")
        // TOOD: メールの本文を更新する
        .text(format!("http://{}:{}/entry?id={}", DOMAIN, PORT, query_id))
        .build()?;

    use std::net::SocketAddr;
    let addr = SocketAddr::from(SMTP_SERVER_ADDR);
    let client = SmtpClient::new(addr, ClientSecurity::None)?;
    let mut mailer = client.transport();
    // TODO: メール送信後のレスポンスが必要か検討する
    let _ = mailer.send(email.into())?;
    Ok(())
}

#[derive(Serialize)]
struct TentativeRegistrationResult {
    email_address: String,
    message: String,
}

fn create_registration_err_response(err: RegistrationError) -> HttpResponse {
    let code: u32;
    let message: String;
    let status_code: StatusCode;
    match err {
        RegistrationError::AlreadyExist { email_address } => {
            code = error::code::USER_ALREADY_EXISTS;
            message = format!("{}は既に登録されています。", email_address);
            status_code = StatusCode::CONFLICT;
        }
        RegistrationError::Duplicate {
            email_address: _,
            count: _,
        } => {
            code = error::code::USER_DUPLICATE;
            message = String::from("test"); // TODO: 変更をする
            status_code = StatusCode::INTERNAL_SERVER_ERROR;
            // TODO: 管理者にメールを送信するか検討する（通り得ない処理なので、管理者への通知があったほうがよいかもしれない）
        }
        RegistrationError::ExceedRegistrationLimit {
            email_address: _,
            count: _,
        } => {
            code = error::code::EXCEED_REGISTRATION_LIMIT;
            message = "アカウント作成を依頼できる回数の上限を超えました。一定の期間が過ぎた後、再度お試しください。".to_string();
            status_code = StatusCode::BAD_REQUEST;
        }
        RegistrationError::DieselError(_e) => {
            code = error::code::DB_ACCESS_ERROR;
            message = String::from("test"); // TODO: 変更をする
            status_code = StatusCode::INTERNAL_SERVER_ERROR;
        }
        RegistrationError::EmailError(_e) => {
            code = error::code::EMAIL_ERROR;
            message = String::from("test"); // TODO: 変更をする
            status_code = StatusCode::INTERNAL_SERVER_ERROR;
        }
        RegistrationError::SmtpError(_e) => {
            code = error::code::SMTP_ERROR;
            message = String::from("test"); // TODO: 変更をする
            status_code = StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    return HttpResponse::build(status_code)
        .content_type("application/problem+json")
        .json(error::Error { code, message });
}

fn create_execution_canceled_response() -> HttpResponse {
    let code = error::code::EXECUTION_CANCELED;
    return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
        .content_type("application/problem+json")
        .json(error::Error {
            code,
            message: String::from("test"), // TODO: 変更をする
        });
}

fn register_tentative_user(
    mail_addr: &str,
    hashed_pwd: &[u8],
    query_id: &str,
    current_date_time: &DateTime<Utc>,
    conn: &PgConnection,
) -> Result<i64, RegistrationError> {
    conn.transaction::<_, RegistrationError, _>(|| {
        check_if_user_exists(mail_addr, conn)?;
        let cnt = count_num_of_registration(mail_addr, conn)?;
        if cnt >= LIMIT_OF_REGISTRATION_RECORD {
            return Err(RegistrationError::ExceedRegistrationLimit {
                email_address: mail_addr.to_string(),
                count: cnt,
            });
        }
        use crate::schema::my_project_schema::tentative_user;
        let tentative_user = model::TentativeAccountInfo {
            query_id,
            email_address: mail_addr,
            hashed_password: hashed_pwd,
            registration_time: current_date_time,
        };
        // TODO: 戻り値（usize: the number of rows affected）を利用する必要があるか検討する
        let _result = diesel::insert_into(tentative_user::table)
            .values(&tentative_user)
            .execute(conn)?;
        Ok(cnt)
    })
}

fn check_if_user_exists(mail_addr: &str, conn: &PgConnection) -> Result<(), RegistrationError> {
    use crate::schema::my_project_schema::user::dsl::*;
    let cnt = user
        .filter(email_address.eq(mail_addr))
        .count()
        .get_result::<i64>(conn)?;
    if cnt > 1 {
        return Err(RegistrationError::Duplicate {
            email_address: mail_addr.to_string(),
            count: cnt,
        });
    }
    if cnt == 1 {
        return Err(RegistrationError::AlreadyExist {
            email_address: mail_addr.to_string(),
        });
    }
    // TODO: 念の為、負の数のケースを考える必要があるか調べる
    Ok(())
}

fn count_num_of_registration(
    mail_addr: &str,
    conn: &PgConnection,
) -> Result<i64, diesel::result::Error> {
    use crate::schema::my_project_schema::tentative_user::dsl::*;
    let cnt = tentative_user
        .filter(email_address.eq(mail_addr))
        .count()
        .get_result::<i64>(conn)?;
    Ok(cnt)
}
