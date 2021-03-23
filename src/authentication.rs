// Copyright 2021 Ken Miura
use crate::utils;

use actix_session::Session;
use actix_web::{error, get, post, web, HttpResponse};
use diesel::prelude::*;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use serde::Deserialize;

// TODO: Consider and change KEY
pub(crate) const SESSION_SIGN_KEY: [u8; 32] = [1; 32];
// TODO: Consider and change KEY
const PASSWORD_HASH_KEY: [u8; 4] = [0, 1, 2, 3];

enum ValidationError {
    EmailAddressFormatError(String),
    PasswordFormatError(String),
}

const EMAIL_ADDRESS_MAX_LENGTH: usize = 254;
//const EMAIL_REGEXP: &str = r"^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$";
const PASSWORD_MIN_LENGTH: usize = 8;
const PASSWORD_MAX_LENGTH: usize = 32;
// TOOD: Add password regexp
//const PASSWORD_REGEXP: &str = r"";

#[derive(Deserialize)]
pub(crate) struct AuthInfo {
    email_address: String,
    password: String,
}

impl AuthInfo {
    fn validate_format(self: &AuthInfo) -> Result<(), ValidationError> {
        let mail_addr_length = self.email_address.len();
        if mail_addr_length > EMAIL_ADDRESS_MAX_LENGTH {
            let error_message = format!(
                "email address length is {}: email address length must be {} or less",
                mail_addr_length, EMAIL_ADDRESS_MAX_LENGTH
            );
            return Err(ValidationError::EmailAddressFormatError(error_message));
        }
        let pwd_length = self.password.len();
        if pwd_length < PASSWORD_MIN_LENGTH || pwd_length > PASSWORD_MAX_LENGTH {
            // NOTE: don't include password related information (pwd_length) for security
            let error_message = format!(
                "password length must be {} or more, {} or less",
                PASSWORD_MIN_LENGTH, PASSWORD_MAX_LENGTH
            );
            return Err(ValidationError::PasswordFormatError(error_message));
        }
        // TODO: Add regular expression check
        return Ok(());
    }
}

#[post("/auth-request")]
pub(crate) async fn auth_request(
    auth_info: web::Json<AuthInfo>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
    session: Session,
) -> HttpResponse {
    let result = auth_info.validate_format();
    if let Err(_e) = result {
        // TODO: Consider returning JSON format
        return HttpResponse::from_error(error::ErrorBadRequest("failed to authenticate account"));
    }
    let mail_addr = auth_info.email_address.clone();
    let pwd = auth_info.password.clone();

    let conn = pool.get().expect("failed to get connection");

    let user = web::block(move || utils::find_user_by_mail_address(&mail_addr, &conn)).await;

    let user_info = user.expect("error");
    let mut auth_res = false;
    match user_info {
        Some(user) => {
            use ring::hmac;
            let key = hmac::Key::new(hmac::HMAC_SHA512, &PASSWORD_HASH_KEY);
            let result = hmac::verify(&key, pwd.as_bytes(), &user.hashed_password);
            match result {
                Ok(_) => auth_res = true,
                Err(_) => auth_res = false,
            }
        }
        None => {}
    }

    if auth_res {
        let _ = session.set("email_address", &auth_info.email_address);
        let contents = "{ \"result\": \"OK\" }";
        HttpResponse::Ok().body(contents)
    } else {
        HttpResponse::from_error(error::ErrorUnauthorized("failed to authenticate"))
    }
}

#[post("/registration-request")]
pub(crate) async fn registration_request(
    auth_info: web::Json<AuthInfo>,
    pool: web::Data<Pool<ConnectionManager<PgConnection>>>,
) -> HttpResponse {
    let result = auth_info.validate_format();
    if let Err(_e) = result {
        // TODO: Consider returning JSON format
        return HttpResponse::from_error(error::ErrorBadRequest("failed to register account"));
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

    let json_text = format!(
        "{{ \"message\": \"{}宛に登録用URLを送りました。登録用URLにアクセスし、登録を完了させてください（登録用URLの有効期間は24時間です）\"}}",
        auth_info.email_address
    );
    HttpResponse::Ok().body(json_text)
}

use crate::models::Account;

fn register_account(
    mail_addr: &String,
    hashed_pwd: &[u8],
    conn: &PgConnection,
) -> Result<usize, diesel::result::Error> {
    use crate::schema::my_project_schema::user;
    let new_account = Account {
        email_address: mail_addr,
        hashed_password: hashed_pwd,
    };

    let result = diesel::insert_into(user::table)
        .values(&new_account)
        .execute(conn);
    result
}

// Use POST for logout: https://stackoverflow.com/questions/3521290/logout-get-or-post
#[post("/logout-request")]
pub(crate) async fn logout_request(session: Session) -> HttpResponse {
    session.purge();
    let contents = "succeeded in logging out";
    HttpResponse::Ok().body(contents)
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
