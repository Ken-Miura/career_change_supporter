// Copyright 2021 Ken Miura
use crate::error_codes;

use crate::models;
use actix_session::Session;
use actix_web::{error, get, http::StatusCode, post, web, HttpResponse};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use regex::Regex;
use ring::hmac;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// TODO: Consider and change KEY
const PASSWORD_HASH_KEY: [u8; 4] = [0, 1, 2, 3];
// TODO: 運用しながら上限を調整する
const LIMIT_OF_REGISTRATION_RECORD: i64 = 7;

enum ValidationError {
    EmailAddressLength { length: usize },
    EmailAddressFormat { email_address: String },
    // NOTE: パスワード系はセキュリティのために入力情報は保持させない
    PasswordLength,
    PasswordFormat,
    PasswordConstraintsViolation,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ValidationError::EmailAddressLength { length } => {
                write!(f, "invalid email address length: {}", length)
            }
            ValidationError::EmailAddressFormat { email_address } => {
                write!(f, "invalid email address format: {}", email_address)
            }
            ValidationError::PasswordLength => {
                write!(f, "invalid password length")
            }
            ValidationError::PasswordFormat => {
                write!(f, "invalid password format")
            }
            ValidationError::PasswordConstraintsViolation => {
                write!(f, "password constraints vaiolation")
            }
        }
    }
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

const EMAIL_ADDRESS_MAX_LENGTH: usize = 254;
const EMAIL_ADDRESS_REGEXP: &str = r"^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$";

const PASSWORD_MIN_LENGTH: usize = 10;
const PASSWORD_MAX_LENGTH: usize = 32;
// TODO: パスワード文字に記号を利用して問題ないか検証する
const PASSWORD_REGEXP: &str = r"^[!-~]{10,32}$";
const UPPER_CASE_REGEXP: &str = r".*[A-Z].*";
const LOWER_CASE_REGEXP: &str = r".*[a-z].*";
const NUMBER_REGEXP: &str = r".*[0-9].*";
const SYMBOL_REGEXP: &str = r".*[!-/:-@\[-`{-~].*";
const CONSTRAINTS_OF_NUM_OF_COMBINATION: u32 = 2;

#[derive(Deserialize)]
pub(crate) struct AuthInfo {
    email_address: String,
    password: String,
}

impl AuthInfo {
    fn validate_format(self: &AuthInfo) -> Result<(), ValidationError> {
        let _ = AuthInfo::validate_email_address(&self.email_address)?;
        let _ = AuthInfo::validate_password(&self.password)?;
        Ok(())
    }

    fn validate_email_address(email_address: &str) -> Result<(), ValidationError> {
        let mail_addr_length = email_address.len();
        if mail_addr_length > EMAIL_ADDRESS_MAX_LENGTH {
            return Err(ValidationError::EmailAddressLength {
                length: mail_addr_length,
            });
        }
        lazy_static! {
            static ref MAIL_ADDR_RE: Regex =
                Regex::new(EMAIL_ADDRESS_REGEXP).expect("never happens panic");
        }
        if !MAIL_ADDR_RE.is_match(email_address) {
            return Err(ValidationError::EmailAddressFormat {
                email_address: email_address.to_string(),
            });
        }
        Ok(())
    }

    /// パスワード要件
    /// 10文字以上32文字以下の文字列
    /// 使える文字列は半角英数字と記号 (ASCIIコードの0x21-0x7e)
    /// 大文字、小文字、数字、記号のいずれか二種類以上を組み合わせる必要がある
    fn validate_password(password: &str) -> Result<(), ValidationError> {
        let pwd_length = password.len();
        if pwd_length < PASSWORD_MIN_LENGTH || pwd_length > PASSWORD_MAX_LENGTH {
            return Err(ValidationError::PasswordLength);
        }
        lazy_static! {
            static ref PWD_RE: Regex = Regex::new(PASSWORD_REGEXP).expect("never happens panic");
        }
        if !PWD_RE.is_match(password) {
            return Err(ValidationError::PasswordFormat);
        }
        if !AuthInfo::check_if_pwd_satisfies_constraints(password) {
            return Err(ValidationError::PasswordConstraintsViolation);
        }
        Ok(())
    }

    fn check_if_pwd_satisfies_constraints(pwd: &str) -> bool {
        lazy_static! {
            static ref UPPER_CASE_RE: Regex =
                Regex::new(UPPER_CASE_REGEXP).expect("never happens panic");
        }
        lazy_static! {
            static ref LOWER_CASE_RE: Regex =
                Regex::new(LOWER_CASE_REGEXP).expect("never happens panic");
        }
        lazy_static! {
            static ref NUMBER_RE: Regex = Regex::new(NUMBER_REGEXP).expect("never happens panic");
        }
        lazy_static! {
            static ref SYMBOL_RE: Regex = Regex::new(SYMBOL_REGEXP).expect("never happens panic");
        }
        let mut count = 0;
        if UPPER_CASE_RE.is_match(pwd) {
            count += 1;
        }
        if LOWER_CASE_RE.is_match(pwd) {
            count += 1;
        }
        if NUMBER_RE.is_match(pwd) {
            count += 1;
        }
        if SYMBOL_RE.is_match(pwd) {
            count += 1;
        }
        count >= CONSTRAINTS_OF_NUM_OF_COMBINATION
    }
}

fn create_validation_err_response(err: ValidationError) -> HttpResponse {
    let code: u32;
    let message: String;
    match err {
        ValidationError::EmailAddressLength { length } => {
            code = error_codes::EMAIL_FORMAT_INVALID_LENGTH;
            message = format!("メールアドレスの長さが不正です (入力されたメールアドレスの長さ: {})。メールアドレスは{}文字以下である必要があります。", length, EMAIL_ADDRESS_MAX_LENGTH);
        }
        ValidationError::EmailAddressFormat { email_address } => {
            code = error_codes::EMAIL_FORMAT_INVALID_EXPRESSION;
            message = format!("メールアドレスの形式が不正です (入力されたメールアドレス: {})。\"email.address@example.com\"のような形式で入力してください。", email_address);
        }
        ValidationError::PasswordLength => {
            code = error_codes::PASSWORD_FORMAT_INVALID_LENGTH;
            message = format!("パスワードの長さが不正です。パスワードは{}文字以上、{}文字以下である必要があります。", PASSWORD_MIN_LENGTH, PASSWORD_MAX_LENGTH);
        }
        ValidationError::PasswordFormat => {
            code = error_codes::PASSWORD_FORMAT_INVALID_EXPRESSION;
            message = "パスワードに使用できない文字が含まれています。パスワードに使用可能な文字は、半角英数字と記号です。".to_string();
        }
        ValidationError::PasswordConstraintsViolation => {
            code = error_codes::PASSWORD_FORMAT_CONSTRAINTS_VIOLATION;
            message = "不正な形式のパスワードです。パスワードは小文字、大文字、数字または記号の内、2種類以上を組み合わせる必要があります。".to_string();
        }
    }
    return HttpResponse::build(StatusCode::BAD_REQUEST)
        .content_type("application/problem+json")
        .json(error_codes::Error { code, message });
}

#[post("/login-request")]
pub(crate) async fn login_request(
    auth_info: web::Json<AuthInfo>,
    _pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
    _session: Session,
) -> HttpResponse {
    let result = auth_info.validate_format();
    if let Err(e) = result {
        log::error!(
            "failed to authenticate \"{}\": {}",
            auth_info.email_address,
            e
        );
        return create_validation_err_response(e);
    }
    // let mail_addr = auth_info.email_address.clone();
    // let pwd = auth_info.password.clone();

    // let conn = pool.get().expect("failed to get connection");

    // let user = web::block(move || utils::find_user_by_mail_address(&mail_addr, &conn)).await;

    // let user_info = user.expect("error");
    // let mut auth_res = false;
    // if let Some(user) = user_info {
    //     let key = hmac::Key::new(hmac::HMAC_SHA512, &PASSWORD_HASH_KEY);
    //     let result = hmac::verify(&key, pwd.as_bytes(), &user.hashed_password);
    //     match result {
    //         Ok(_) => auth_res = true,
    //         Err(_) => auth_res = false,
    //     }
    // }

    // if auth_res {
    //     let _ = session.set("email_address", &auth_info.email_address);
    //     let contents = "{ \"result\": \"OK\" }";
    //     HttpResponse::Ok().body(contents)
    // } else {
    //     let code = error_codes::AUTHENTICATION_FAILED;
    //     let message = "メールアドレス、もしくはパスワードが間違っています。".to_string();
    //     return HttpResponse::build(StatusCode::UNAUTHORIZED)
    //         .content_type("application/problem+json")
    //         .json(error_codes::Error { code, message });
    // }
    // TODO: 一時的に同じレスポンスを返すようにする
    HttpResponse::build(StatusCode::OK).finish()
}

#[derive(Deserialize)]
pub(crate) struct EntryRequest {
    id: String,
}

const UUID_REGEXP: &str = "^[a-zA-Z0-9]{32}$";

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
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
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
    let body =
        r#"<!DOCTYPE html>
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
        if time_passed.num_days() > 1 {
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
    let user = models::AccountInfo {
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
) -> Result<Vec<models::TentativeUser>, EntryError> {
    use crate::schema::my_project_schema::tentative_user::dsl::*;
    let users = tentative_user
        .filter(query_id.eq(entry_id))
        .get_results::<models::TentativeUser>(conn)?;
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
        error_codes::INTERNAL_SERVER_ERROR_MESSAGE,
        error_codes::DB_CONNECTION_UNAVAILABLE
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
    lazy_static! {
        static ref UUID_RE: Regex = Regex::new(UUID_REGEXP).expect("never happens panic");
    }
    if !UUID_RE.is_match(id) {
        return Err(IdValidationError::InvalidId(id.to_string()));
    }
    Ok(())
}

#[post("/registration-request")]
pub(crate) async fn registration_request(
    auth_info: web::Json<AuthInfo>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> HttpResponse {
    let result = auth_info.validate_format();
    if let Err(e) = result {
        log::error!("failed to register \"{}\": {}", auth_info.email_address, e);
        return create_validation_err_response(e);
    }

    let result = pool.get();
    if let Err(e) = result {
        log::error!("failed to get connection: {}", e);
        return create_db_connection_error_response();
    }
    let conn = result.expect("never happens panic");
    let mail_addr = auth_info.email_address.clone();
    let key = hmac::Key::new(hmac::HMAC_SHA512, &PASSWORD_HASH_KEY);
    let hashed_password = hmac::sign(&key, auth_info.password.as_bytes());
    let query_id = Uuid::new_v4().to_simple().to_string();
    let query_id_to_register = query_id.clone();
    let current_date_time = Utc::now();
    let result = web::block(move || {
        register_tentative_user(
            &mail_addr,
            hashed_password.as_ref(),
            &query_id_to_register,
            &current_date_time,
            &conn,
        )
    })
    .await;

    if let Err(err) = result {
        match err {
            error::BlockingError::Error(e) => {
                log::error!(
                    "failed to register \"{}\" tentatively: {}",
                    auth_info.email_address,
                    e
                );
                return create_registration_err_response(e);
            }
            error::BlockingError::Canceled => {
                log::error!("failed to register tentatively: error::BlockingError::Canceled");
                return create_execution_canceled_response();
            }
        }
    }
    let num_of_tentative_accounts = result.expect("never happens panic");
    let notification =  format!(
        "{}宛に登録用URLを送りました。登録用URLにアクセスし、登録を完了させてください（登録用URLの有効期間は24時間です）",
        auth_info.email_address.clone()
    );
    let message = if num_of_tentative_accounts > 0 {
        format!(
            "{}。メールが届かない場合、迷惑メールフォルダに届いていないか、このサイトのドメインのメールが受信許可されているかをご確認ください。",
            notification
        )
    } else {
        notification
    };
    let result = send_activation_mail(&auth_info.email_address, &query_id);
    if let Err(err) = result {
        log::error!("failed to send email: {}", err);
        return create_registration_err_response(err);
    }
    HttpResponse::Ok().json(TentativeRegistrationResult {
        email_address: auth_info.email_address.clone(),
        message,
    })
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
            code = error_codes::USER_ALREADY_EXISTS;
            message = format!("{}は既に登録されています。", email_address);
            status_code = StatusCode::CONFLICT;
        }
        RegistrationError::Duplicate {
            email_address: _,
            count: _,
        } => {
            code = error_codes::USER_DUPLICATE;
            message = error_codes::INTERNAL_SERVER_ERROR_MESSAGE.to_string();
            status_code = StatusCode::INTERNAL_SERVER_ERROR;
            // TODO: 管理者にメールを送信するか検討する（通り得ない処理なので、管理者への通知があったほうがよいかもしれない）
        }
        RegistrationError::ExceedRegistrationLimit {
            email_address: _,
            count: _,
        } => {
            code = error_codes::EXCEED_REGISTRATION_LIMIT;
            message = "アカウント作成を依頼できる回数の上限を超えました。一定の期間が過ぎた後、再度お試しください。".to_string();
            status_code = StatusCode::BAD_REQUEST;
        }
        RegistrationError::DieselError(_e) => {
            code = error_codes::DB_ACCESS_ERROR;
            message = error_codes::INTERNAL_SERVER_ERROR_MESSAGE.to_string();
            status_code = StatusCode::INTERNAL_SERVER_ERROR;
        }
        RegistrationError::EmailError(_e) => {
            code = error_codes::EMAIL_ERROR;
            message = error_codes::INTERNAL_SERVER_ERROR_MESSAGE.to_string();
            status_code = StatusCode::INTERNAL_SERVER_ERROR;
        }
        RegistrationError::SmtpError(_e) => {
            code = error_codes::SMTP_ERROR;
            message = error_codes::INTERNAL_SERVER_ERROR_MESSAGE.to_string();
            status_code = StatusCode::INTERNAL_SERVER_ERROR;
        }
    };
    return HttpResponse::build(status_code)
        .content_type("application/problem+json")
        .json(error_codes::Error { code, message });
}

fn create_db_connection_error_response() -> HttpResponse {
    let code = error_codes::DB_CONNECTION_UNAVAILABLE;
    return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
        .content_type("application/problem+json")
        .json(error_codes::Error {
            code,
            message: error_codes::INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
        });
}

fn create_execution_canceled_response() -> HttpResponse {
    let code = error_codes::EXECUTION_CANCELED;
    return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
        .content_type("application/problem+json")
        .json(error_codes::Error {
            code,
            message: error_codes::INTERNAL_SERVER_ERROR_MESSAGE.to_string(),
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
        let tentative_user = models::TentativeAccountInfo {
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
        return HttpResponse::from_error(error::ErrorUnauthorized("failed to authenticate"));
    }
    // set value to explicitly enhance ttl
    let _ = session.set("email_address", session_info.expect("msg: &str"));
    let contents = "contents";
    HttpResponse::Ok().body(contents)
}
