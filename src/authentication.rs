// Copyright 2021 Ken Miura
use crate::error_codes;

use crate::models::TentativeAccountInfo;
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
    DieselError(diesel::result::Error),
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
            RegistrationError::DieselError(e) => {
                write!(f, "diesel error: {}", e)
            }
        }
    }
}

impl From<diesel::result::Error> for RegistrationError {
    fn from(error: diesel::result::Error) -> Self {
        RegistrationError::DieselError(error)
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
    let current_date_time = Utc::now();
    let result = web::block(move || {
        register_tentative_user(
            &mail_addr,
            hashed_password.as_ref(),
            &query_id.clone(),
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
    // TODO: 仮ユーザテーブルにアクセスするためのクエリパラメータを含んだメールを送信
    test_mail();

    let message = format!(
        "{}宛に登録用URLを送りました。登録用URLにアクセスし、登録を完了させてください（登録用URLの有効期間は24時間です）",
        auth_info.email_address.clone()
    );
    HttpResponse::Ok().json(TentativeRegistrationResult {
        email_address: auth_info.email_address.clone(),
        message,
    })
}

fn test_mail() {
    use lettre::{ClientSecurity, SmtpClient, Transport};
    use lettre_email::EmailBuilder;

    let email = EmailBuilder::new()
        .to(("to@example.com", "Firstname Lastname"))
        .from("from@example.com")
        .subject("日本語のタイトル")
        .text("日本語の本文")
        .build()
        .unwrap();

    use std::net::SocketAddr;
    let addr = SocketAddr::from(([127, 0, 0, 1], 1025));
    let mut mailer = SmtpClient::new(addr, ClientSecurity::None)
        .unwrap()
        .transport();

    // Send the email
    let _ = mailer.send(email.into());
}

#[derive(Serialize)]
struct TentativeRegistrationResult {
    email_address: String,
    message: String,
}

fn create_registration_err_response(err: RegistrationError) -> HttpResponse {
    let code: u32;
    let message: String;
    match err {
        RegistrationError::AlreadyExist { email_address } => {
            code = error_codes::USER_ALREADY_EXISTS;
            message = format!("{}は既に登録されています。", email_address);
        }
        RegistrationError::Duplicate {
            email_address: _,
            count: _,
        } => {
            code = error_codes::USER_DUPLICATE;
            message = error_codes::INTERNAL_SERVER_ERROR_MESSAGE.to_string();
        }
        RegistrationError::DieselError(_e) => {
            code = error_codes::DB_ACCESS_ERROR;
            message = error_codes::INTERNAL_SERVER_ERROR_MESSAGE.to_string();
        }
    };
    return HttpResponse::build(StatusCode::BAD_REQUEST)
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
) -> Result<(), RegistrationError> {
    use crate::schema::my_project_schema::tentative_user;
    use crate::schema::my_project_schema::user::dsl::*;
    conn.transaction::<_, RegistrationError, _>(|| {
        // DBに既にメールアドレスが登録されているかチェック
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

        // 仮ユーザとして登録
        let tentative_user = TentativeAccountInfo {
            query_id,
            email_address: mail_addr,
            hashed_password: hashed_pwd,
            registration_time: current_date_time,
        };
        // TODO: 戻り値（usize: the number of rows affected）を利用する必要があるか検討する
        let _result = diesel::insert_into(tentative_user::table)
            .values(&tentative_user)
            .execute(conn)?;
        Ok(())
    })
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
