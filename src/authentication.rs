// Copyright 2021 Ken Miura
use crate::error_codes;
use crate::utils;

use actix_session::Session;
use actix_web::{error, get, http::StatusCode, post, web, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use regex::Regex;
use serde::{Deserialize, Serialize};

// TODO: Consider and change KEY
const PASSWORD_HASH_KEY: [u8; 4] = [0, 1, 2, 3];

enum ValidationError {
    EmailAddressLength,
    EmailAddressExpresson,
    PasswordLength,
    PasswordExpression,
    PasswordConstraintsViolation,
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
        let _ = AuthInfo::validate_email_address_format(&self.email_address)?;
        let _ = AuthInfo::validate_password_format(&self.password)?;
        Ok(())
    }

    fn validate_email_address_format(email_address: &str) -> Result<(), ValidationError> {
        let mail_addr_length = email_address.len();
        if mail_addr_length > EMAIL_ADDRESS_MAX_LENGTH {
            return Err(ValidationError::EmailAddressLength);
        }
        lazy_static! {
            static ref MAIL_ADDR_RE: Regex =
                Regex::new(EMAIL_ADDRESS_REGEXP).expect("never happens panic");
        }
        if !MAIL_ADDR_RE.is_match(email_address) {
            return Err(ValidationError::EmailAddressExpresson);
        }
        Ok(())
    }

    /// パスワード要件
    /// 10文字以上32文字以下の文字列
    /// 使える文字列は半角英数字と記号 (ASCIIコードの0x21-0x7e)
    /// 大文字、小文字、数字、記号のいずれか二種類以上を組み合わせる必要がある
    fn validate_password_format(password: &str) -> Result<(), ValidationError> {
        let pwd_length = password.len();
        if pwd_length < PASSWORD_MIN_LENGTH || pwd_length > PASSWORD_MAX_LENGTH {
            return Err(ValidationError::PasswordLength);
        }
        lazy_static! {
            static ref PWD_RE: Regex = Regex::new(PASSWORD_REGEXP).expect("never happens panic");
        }
        if !PWD_RE.is_match(password) {
            return Err(ValidationError::PasswordExpression);
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

fn create_validation_err_response(auth_info: &AuthInfo, err: ValidationError) -> HttpResponse {
    let code: u32;
    let message: String;
    match err {
        ValidationError::EmailAddressLength => {
            // TODO: Log
            code = error_codes::EMAIL_FORMAT_INVALID_LENGTH;
            message = format!("メールアドレスの長さが不正です (入力されたメールアドレスの長さ: {})。メールアドレスは{}文字以下である必要があります。", auth_info.email_address.len(), EMAIL_ADDRESS_MAX_LENGTH);
        }
        ValidationError::EmailAddressExpresson => {
            // TODO: Log
            code = error_codes::EMAIL_FORMAT_INVALID_EXPRESSION;
            message = format!("メールアドレスの形式が不正です (入力されたメールアドレス: {})。\"email.address@example.com\"のような形式で入力してください。", auth_info.email_address);
        }
        ValidationError::PasswordLength => {
            // NOTE: Never log security sensitive information
            // TODO: Log
            code = error_codes::PASSWORD_FORMAT_INVALID_LENGTH;
            message = format!("パスワードの長さが不正です。パスワードは{}文字以上、{}文字以下である必要があります。", PASSWORD_MIN_LENGTH, PASSWORD_MAX_LENGTH);
        }
        ValidationError::PasswordExpression => {
            // NOTE: Never log security sensitive information
            // TODO: Log
            code = error_codes::PASSWORD_FORMAT_INVALID_EXPRESSION;
            message = "パスワードに使用できない文字が含まれています。パスワードに使用可能な文字は、半角英数字と記号です。".to_string();
        }
        ValidationError::PasswordConstraintsViolation => {
            // NOTE: Never log security sensitive information
            // TODO: Log
            code = error_codes::PASSWORD_FORMAT_CONSTRAINTS_VIOLATION;
            message = "不正な形式のパスワードです。パスワードは小文字、大文字、数字または記号の内、2種類以上を組み合わせる必要があります。".to_string();
        }
    }
    return HttpResponse::build(StatusCode::BAD_REQUEST)
        .content_type("application/problem+json")
        .json(error_codes::Error { code, message });
}

#[post("/auth-request")]
pub(crate) async fn auth_request(
    auth_info: web::Json<AuthInfo>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
    session: Session,
) -> HttpResponse {
    let result = auth_info.validate_format();
    if let Err(e) = result {
        // TODO: Log authentication fail
        return create_validation_err_response(&auth_info, e);
    }
    let mail_addr = auth_info.email_address.clone();
    let pwd = auth_info.password.clone();

    let conn = pool.get().expect("failed to get connection");

    let user = web::block(move || utils::find_user_by_mail_address(&mail_addr, &conn)).await;

    let user_info = user.expect("error");
    let mut auth_res = false;
    if let Some(user) = user_info {
        use ring::hmac;
        let key = hmac::Key::new(hmac::HMAC_SHA512, &PASSWORD_HASH_KEY);
        let result = hmac::verify(&key, pwd.as_bytes(), &user.hashed_password);
        match result {
            Ok(_) => auth_res = true,
            Err(_) => auth_res = false,
        }
    }

    if auth_res {
        let _ = session.set("email_address", &auth_info.email_address);
        let contents = "{ \"result\": \"OK\" }";
        HttpResponse::Ok().body(contents)
    } else {
        let code = error_codes::AUTHENTICATION_FAILED;
        let message = "メールアドレス、もしくはパスワードが間違っています。".to_string();
        return HttpResponse::build(StatusCode::UNAUTHORIZED)
            .content_type("application/problem+json")
            .json(error_codes::Error { code, message });
    }
}

#[post("/registration-request")]
pub(crate) async fn registration_request(
    auth_info: web::Json<AuthInfo>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> HttpResponse {
    log::info!("registration request call"); // TODO: Remove this line because this is just test for logging crate
    let result = auth_info.validate_format();
    if let Err(e) = result {
        // TODO: Log registration fail
        return create_validation_err_response(&auth_info, e);
    }

    use ring::hmac;
    let key = hmac::Key::new(hmac::HMAC_SHA512, &PASSWORD_HASH_KEY);
    let hashed_password = hmac::sign(&key, auth_info.password.as_bytes());

    // トランザクションで、既存のDBにメールアドレスがあるかチェック＋登録
    // TODO: メールアドレスにUnique制約を追加するのか、トランザクションを利用するのか確認する
    let mail_addr = auth_info.email_address.clone();
    let conn = pool.get().expect("failed to get connection");
    let result =
        web::block(move || register_account(&mail_addr, hashed_password.as_ref(), &conn)).await;

    match result {
        Ok(num) => print!("{}", num),
        Err(err) => {
            // reach here if unique violation
            // TOOD: Consider other error handling
            return HttpResponse::from_error(error::ErrorConflict(format!(
                "failed to register account: {}",
                err
            )));
        }
    }

    // 登録用URLのクエリパラメータの生成
    // TODO: Add func to enable account
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    let _entry_id: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16) // TODO: Consider enough length
        .map(char::from)
        .collect();

    // 登録用URLを含んだメールを送信

    let text = format!(
        "{}宛に登録用URLを送りました。登録用URLにアクセスし、登録を完了させてください（登録用URLの有効期間は24時間です）",
        auth_info.email_address
    );
    HttpResponse::Ok().json(Message { message: text })
}

#[derive(Serialize)]
struct Message {
    message: String,
}

use crate::models::Account;

fn register_account(
    mail_addr: &str,
    hashed_pwd: &[u8],
    conn: &PgConnection,
) -> Result<usize, diesel::result::Error> {
    use crate::schema::my_project_schema::user;
    let new_account = Account {
        email_address: mail_addr,
        hashed_password: hashed_pwd,
    };
    diesel::insert_into(user::table)
        .values(&new_account)
        .execute(conn)
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
