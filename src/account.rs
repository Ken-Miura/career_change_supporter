// Copyright 2021 Ken Miura

use crate::common;
use crate::common::credential;
use crate::common::error;
use crate::common::error::ToCode;
use crate::common::error::ToMessage;
use crate::common::error::ToStatusCode;

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
const TEMPORARY_ACCOUNT_LIMIT: i64 = 7;
const UUID_REGEXP: &str = "^[a-zA-Z0-9]{32}$";
// TODO: 環境変数から読み込むように変更する
const SMTP_SERVER_ADDR: ([u8; 4], u16) = ([127, 0, 0, 1], 1025);
// TODO: サーバのドメイン名を変更し、共通で利用するmoduleへ移動する
const DOMAIN: &str = "localhost";
const PORT: &str = "8080";

static UUID_RE: Lazy<Regex> = Lazy::new(|| Regex::new(UUID_REGEXP).expect("never happens panic"));

// TODO: ValidationError以降の残りの処理を書き直す
#[post("/temporary-account")]
pub(crate) async fn temporary_account(
    credential: web::Json<credential::Credential>,
    pool: web::Data<common::ConnectionPool>,
) -> Result<HttpResponse, common::credential::ValidationError> {
    let _ = credential.validate().map_err(|e| {
        log::info!(
            "failed to create temporary account for  \"{}\" (code: {}): {}",
            credential.email_address,
            e.to_code(),
            e
        );
        e
    })?;

    let result = pool.get();
    if let Err(e) = result {
        log::error!("failed to get connection: {}", e);
        return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type("application/problem+json")
            .json(error::Error {
                code: e.to_code(),
                message: e.to_message(),
            }));
    }

    let conn = result.expect("never happens panic");
    let mail_addr = credential.email_address.clone();
    let hashed_password = credential::hash_password(&credential.password);
    let id = Uuid::new_v4().to_simple().to_string();
    let id_to_register = id.clone();
    let current_date_time = Utc::now();
    let result = web::block(move || {
        create_temporary_account(
            &mail_addr,
            &hashed_password,
            &id_to_register,
            &current_date_time,
            &conn,
        )
    })
    .await;

    if let Err(actix_web::error::BlockingError::Error(e)) = result {
        if e.to_status_code() == StatusCode::INTERNAL_SERVER_ERROR {
            log::error!(
                "failed to create temporary account for  \"{}\" (code: {}): {}",
                credential.email_address,
                e.to_code(),
                e
            );
        } else {
            log::info!(
                "failed to create temporary account for  \"{}\" (code: {}): {}",
                credential.email_address,
                e.to_code(),
                e
            );
        }
        return Ok(HttpResponse::build(e.to_status_code())
            .content_type("application/problem+json")
            .json(error::Error {
                code: e.to_code(),
                message: e.to_message(),
            }));
    }
    if let Err(actix_web::error::BlockingError::Canceled) = result {
        log::error!("failed to create temporary account for  \"{}\": actix_web::error::BlockingError::Canceled", credential.email_address);
        return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type("application/problem+json")
            .json(error::Error {
                code: error::code::INTERNAL_SERVER_ERROR,
                message: String::from(common::error::INTERNAL_SERVER_ERROR_MESSAGE),
            }));
    }

    let temporary_account_cnt = result.expect("never happens panic");
    let notification =  format!(
        "{}宛に登録用URLを送りました。登録用URLにアクセスし、登録を完了させてください（登録用URLの有効期間は24時間です）",
        credential.email_address
    );
    let mut message = notification.clone();
    if temporary_account_cnt > 1 {
        message = format!(
            "{}。メールが届かない場合、迷惑メールフォルダに届いていないか、このサイトのドメインのメールが受信許可されているかをご確認ください。",
            notification
        )
    }
    let result = send_notification_mail(&credential.email_address, &id);
    if let Err(err) = result {
        log::error!("failed to send email: {}", err);
        return Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
            .content_type("application/problem+json")
            .json(error::Error {
                code: error::code::INTERNAL_SERVER_ERROR,
                message: String::from(common::error::INTERNAL_SERVER_ERROR_MESSAGE),
            }));
    }
    Ok(HttpResponse::Ok().json(TemporaryAccountResult {
        email_address: credential.email_address.clone(),
        message,
    }))
}

fn create_temporary_account(
    mail_addr: &str,
    hashed_pwd: &[u8],
    temporary_account_id: &str,
    current_date_time: &DateTime<Utc>,
    conn: &PgConnection,
) -> Result<i64, TemporaryAccountCreationError> {
    conn.transaction::<_, TemporaryAccountCreationError, _>(|| {
        check_if_account_exists(mail_addr, conn)?;
        let cnt = num_of_temporary_accounts(mail_addr, conn)?;
        if cnt >= TEMPORARY_ACCOUNT_LIMIT {
            return Err(TemporaryAccountCreationError::ReachLimit {
                code: error::code::REACH_TEMPORARY_ACCOUNT_LIMIT,
                email_address: mail_addr.to_string(),
                count: cnt,
                status_code: StatusCode::BAD_REQUEST,
            });
        }
        use crate::schema::my_project_schema::user_temporary_account;
        let temp_account = model::TemporaryAccount {
            temporary_account_id,
            email_address: mail_addr,
            hashed_password: hashed_pwd,
            created_at: current_date_time,
        };
        // TODO: 戻り値（usize: the number of rows affected）を利用する必要があるか検討する
        let _result = diesel::insert_into(user_temporary_account::table)
            .values(&temp_account)
            .execute(conn)?;
        Ok(cnt)
    })
}

fn check_if_account_exists(
    mail_addr: &str,
    conn: &PgConnection,
) -> Result<(), TemporaryAccountCreationError> {
    use crate::schema::my_project_schema::user_account::dsl::*;
    let cnt = user_account
        .filter(email_address.eq(mail_addr))
        .count()
        .get_result::<i64>(conn)?;
    if cnt > 1 {
        return Err(TemporaryAccountCreationError::AccountDuplicate {
            code: error::code::INTERNAL_SERVER_ERROR,
            email_address: mail_addr.to_string(),
            count: cnt,
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        });
    }
    if cnt == 1 {
        return Err(TemporaryAccountCreationError::AccountAlreadyExist {
            code: error::code::ACCOUNT_ALREADY_EXISTS,
            email_address: mail_addr.to_string(),
            status_code: StatusCode::CONFLICT,
        });
    }
    // TODO: 念の為、負の数のケースを考える必要があるか調べる
    Ok(())
}

fn num_of_temporary_accounts(
    mail_addr: &str,
    conn: &PgConnection,
) -> Result<i64, diesel::result::Error> {
    use crate::schema::my_project_schema::user_temporary_account::dsl::*;
    let cnt = user_temporary_account
        .filter(email_address.eq(mail_addr))
        .count()
        .get_result::<i64>(conn)?;
    Ok(cnt)
}

#[derive(Debug)]
enum TemporaryAccountCreationError {
    AccountAlreadyExist {
        code: u32,
        email_address: String,
        status_code: StatusCode,
    },
    AccountDuplicate {
        code: u32,
        email_address: String,
        count: i64,
        status_code: StatusCode,
    },
    ReachLimit {
        code: u32,
        email_address: String,
        count: i64,
        status_code: StatusCode,
    },
    DieselError {
        code: u32,
        error: diesel::result::Error,
        status_code: StatusCode,
    },
    EmailError {
        code: u32,
        error: lettre_email::error::Error,
        status_code: StatusCode,
    },
    SmtpError {
        code: u32,
        error: lettre::smtp::error::Error,
        status_code: StatusCode,
    },
}

impl fmt::Display for TemporaryAccountCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            TemporaryAccountCreationError::AccountAlreadyExist {
                code,
                email_address,
                status_code: _,
            } => {
                write!(
                    f,
                    "\"{}\" has already existed in user (code: {})",
                    email_address, code
                )
            }
            TemporaryAccountCreationError::AccountDuplicate {
                code: _,
                email_address,
                count,
                status_code: _,
            } => {
                write!(
                    f,
                    "fatal error: there are {} records of \"{}\" in account table",
                    count, email_address
                )
            }
            TemporaryAccountCreationError::ReachLimit {
                code,
                email_address,
                count,
                status_code: _,
            } => {
                write!(
                    f,
                    "reach limit of temporary accounts (code: {}, email {} found {} times)",
                    code, email_address, count,
                )
            }
            TemporaryAccountCreationError::DieselError {
                code,
                error,
                status_code: _,
            } => {
                write!(f, "diesel error (code: {}): {}", code, error)
            }
            TemporaryAccountCreationError::EmailError {
                code,
                error,
                status_code: _,
            } => {
                write!(f, "lettre email error (code: {}): {}", code, error)
            }
            TemporaryAccountCreationError::SmtpError {
                code,
                error,
                status_code: _,
            } => {
                write!(f, "lettre smtp error (code: {}): {}", code, error)
            }
        }
    }
}

impl From<diesel::result::Error> for TemporaryAccountCreationError {
    fn from(error: diesel::result::Error) -> Self {
        TemporaryAccountCreationError::DieselError {
            code: error::code::INTERNAL_SERVER_ERROR,
            error,
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<lettre_email::error::Error> for TemporaryAccountCreationError {
    fn from(error: lettre_email::error::Error) -> Self {
        TemporaryAccountCreationError::EmailError {
            code: error::code::INTERNAL_SERVER_ERROR,
            error,
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<lettre::smtp::error::Error> for TemporaryAccountCreationError {
    fn from(error: lettre::smtp::error::Error) -> Self {
        TemporaryAccountCreationError::SmtpError {
            code: error::code::INTERNAL_SERVER_ERROR,
            error,
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl error::ToCode for TemporaryAccountCreationError {
    fn to_code(&self) -> u32 {
        match self {
            TemporaryAccountCreationError::AccountAlreadyExist {
                code,
                email_address: _,
                status_code: _,
            } => *code,
            TemporaryAccountCreationError::AccountDuplicate {
                code,
                email_address: _,
                count: _,
                status_code: _,
            } => *code,
            TemporaryAccountCreationError::ReachLimit {
                code,
                email_address: _,
                count: _,
                status_code: _,
            } => *code,
            TemporaryAccountCreationError::DieselError {
                code,
                error: _,
                status_code: _,
            } => *code,
            TemporaryAccountCreationError::EmailError {
                code,
                error: _,
                status_code: _,
            } => *code,
            TemporaryAccountCreationError::SmtpError {
                code,
                error: _,
                status_code: _,
            } => *code,
        }
    }
}

impl error::ToMessage for TemporaryAccountCreationError {
    fn to_message(&self) -> String {
        match self {
            TemporaryAccountCreationError::AccountAlreadyExist {
                code: _,
                email_address,
                status_code: _
            } => format!("{}は既に登録されています。", email_address),
            TemporaryAccountCreationError::AccountDuplicate {
                code: _,
                email_address: _,
                count: _,
                status_code: _
            } => String::from(common::error::INTERNAL_SERVER_ERROR_MESSAGE),
            TemporaryAccountCreationError::ReachLimit {
                code: _,
                email_address: _,
                count: _,
                status_code: _
            } => String::from("アカウント作成を依頼できる回数の上限を超えました。一定の期間が過ぎた後、再度お試しください。"),
            TemporaryAccountCreationError::DieselError { code: _, error: _, status_code: _ } => String::from(common::error::INTERNAL_SERVER_ERROR_MESSAGE),
            TemporaryAccountCreationError::EmailError { code: _, error: _, status_code: _ } => String::from(common::error::INTERNAL_SERVER_ERROR_MESSAGE),
            TemporaryAccountCreationError::SmtpError { code: _, error: _, status_code: _ } => String::from(common::error::INTERNAL_SERVER_ERROR_MESSAGE),
        }
    }
}

impl error::ToStatusCode for TemporaryAccountCreationError {
    fn to_status_code(&self) -> StatusCode {
        match self {
            TemporaryAccountCreationError::AccountAlreadyExist {
                code: _,
                email_address: _,
                status_code,
            } => *status_code,
            TemporaryAccountCreationError::AccountDuplicate {
                code: _,
                email_address: _,
                count: _,
                status_code,
            } => *status_code,
            TemporaryAccountCreationError::ReachLimit {
                code: _,
                email_address: _,
                count: _,
                status_code,
            } => *status_code,
            TemporaryAccountCreationError::DieselError {
                code: _,
                error: _,
                status_code,
            } => *status_code,
            TemporaryAccountCreationError::EmailError {
                code: _,
                error: _,
                status_code,
            } => *status_code,
            TemporaryAccountCreationError::SmtpError {
                code: _,
                error: _,
                status_code,
            } => *status_code,
        }
    }
}

fn send_notification_mail(
    email_address: &str,
    temporary_account_id: &str,
) -> Result<(), TemporaryAccountCreationError> {
    use lettre::{ClientSecurity, SmtpClient, Transport};
    use lettre_email::EmailBuilder;
    let email = EmailBuilder::new()
        .to(email_address)
        // TODO: 送信元メールを更新する
        .from("from@example.com")
        // TOOD: メールの件名を更新する
        .subject("日本語のタイトル")
        // TOOD: メールの本文を更新する
        .text(format!(
            "http://{}:{}/temporary-accounts?id={}",
            DOMAIN, PORT, temporary_account_id
        ))
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
struct TemporaryAccountResult {
    email_address: String,
    message: String,
}

// TODO: SameSite=Strictで問題ないか（アクセスできるか）確認する
#[get("/temporary-accounts")]
pub(crate) async fn temporary_accounts(
    web::Query(account_req): web::Query<AccountRequest>,
    pool: web::Data<common::ConnectionPool>,
) -> HttpResponse {
    let result = validate_id_format(&account_req.id);
    if let Err(e) = result {
        log::info!("failed to create account: {}", e);
        return create_invalid_id_format_view();
    }
    let result = pool.get();
    if let Err(e) = result {
        log::error!("failed to get connection: {}", e);
        return create_internal_error_view();
    }
    let conn = result.expect("never happens panic");
    let current_date_time = Utc::now();
    let temp_acc_id = account_req.id.clone();
    let result = web::block(move || create_account(&temp_acc_id, current_date_time, &conn)).await;
    if let Err(err) = result {
        match err {
            actix_web::error::BlockingError::Error(e) => {
                if e.to_status_code() == StatusCode::INTERNAL_SERVER_ERROR {
                    log::error!(
                        "failed to create account (temporary account id: {}): {}",
                        &account_req.id,
                        e
                    );
                } else {
                    log::info!(
                        "failed to create account (temporary account id: {}): {}",
                        &account_req.id,
                        e
                    );
                }
                return create_error_view(e);
            }
            actix_web::error::BlockingError::Canceled => {
                log::error!("failed to create account (temporary account id: {}): actix_web::error::BlockingError::Canceled", &account_req.id);
                return create_internal_error_view();
            }
        }
    }
    let email_address = result.expect("never happens panic");
    let result = send_account_creation_success_mail(&email_address);
    if let Err(e) = result {
        log::error!(
            "failed to complete creating account (\"{}\"): {}",
            email_address,
            e
        );
        // TODO: ログに残すだけで、成功として返して良いか検討する
    }
    create_success_view()
}

#[derive(Deserialize)]
pub(crate) struct AccountRequest {
    id: String,
}

fn validate_id_format(id: &str) -> Result<(), IdValidationError> {
    if !UUID_RE.is_match(id) {
        return Err(IdValidationError::InvalidIdFormat(id.to_string()));
    }
    Ok(())
}

enum IdValidationError {
    InvalidIdFormat(String),
}

impl fmt::Display for IdValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IdValidationError::InvalidIdFormat(id) => {
                write!(f, "invalid id: {}", id)
            }
        }
    }
}

fn create_invalid_id_format_view() -> HttpResponse {
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

fn create_internal_error_view() -> HttpResponse {
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
        common::error::INTERNAL_SERVER_ERROR_MESSAGE,
        error::code::INTERNAL_SERVER_ERROR
    );
    return HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
        .content_type("text/html; charset=UTF-8")
        .body(body);
}

fn create_account(
    temporary_account_id: &str,
    current_date_time: DateTime<Utc>,
    conn: &PgConnection,
) -> Result<String, AccountCreationError> {
    conn.transaction::<_, AccountCreationError, _>(|| {
        let temp_acc = find_temporary_account_by_id(temporary_account_id, conn)?;
        let result = delete_temporary_account(temporary_account_id, conn);
        if let Err(e) = result {
            log::warn!("failed to delete temporary account: {}", e);
        };
        let time_elapsed = current_date_time - temp_acc.created_at;
        if time_elapsed.num_days() > 0 {
            return Err(AccountCreationError::TemporaryAccountExpire {
                code: error::code::TEMPORARY_ACCOUNT_EXPIRED,
                id: temporary_account_id.to_string(),
                created_at: temp_acc.created_at,
                activated_at: current_date_time,
                status_code: StatusCode::BAD_REQUEST,
            });
        }
        // NOTE: 関数内でtransactionを利用しているため、この点でSAVEPOINTとなる
        // TODO: transacstionの中で、transacstionを利用して問題がないか確認する
        create_account_inner(
            &temp_acc.email_address,
            temp_acc.hashed_password.as_ref(),
            conn,
        )?;
        Ok(temp_acc.email_address)
    })
}

fn find_temporary_account_by_id(
    temp_acc_id: &str,
    conn: &PgConnection,
) -> Result<model::TemporaryAccountQueryResult, AccountCreationError> {
    use crate::schema::my_project_schema::user_temporary_account::dsl::*;
    let users = user_temporary_account
        .filter(temporary_account_id.eq(temp_acc_id))
        .get_results::<model::TemporaryAccountQueryResult>(conn)?;
    if users.is_empty() {
        return Err(AccountCreationError::NoTemporaryAccount {
            code: error::code::NO_TEMPORARY_ACCOUNT,
            id: temp_acc_id.to_string(),
            status_code: StatusCode::NOT_FOUND,
        });
    }
    if users.len() != 1 {
        return Err(AccountCreationError::TemporaryAccountDuplicate {
            code: error::code::INTERNAL_SERVER_ERROR,
            id: temp_acc_id.to_string(),
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        });
    }
    let user = users[0].clone();
    Ok(user)
}

#[derive(Debug)]
enum AccountCreationError {
    DieselError {
        code: u32,
        error: diesel::result::Error,
        status_code: StatusCode,
    },
    NoTemporaryAccount {
        code: u32,
        id: String,
        status_code: StatusCode,
    },
    TemporaryAccountDuplicate {
        code: u32,
        id: String,
        status_code: StatusCode,
    },
    TemporaryAccountExpire {
        code: u32,
        id: String,
        created_at: DateTime<Utc>,
        activated_at: DateTime<Utc>,
        status_code: StatusCode,
    },
    AccountDuplicate {
        code: u32,
        email_address: String,
        status_code: StatusCode,
    },
    EmailError {
        code: u32,
        error: lettre_email::error::Error,
        status_code: StatusCode,
    },
    SmtpError {
        code: u32,
        error: lettre::smtp::error::Error,
        status_code: StatusCode,
    },
}

impl fmt::Display for AccountCreationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AccountCreationError::DieselError {
                code: _,
                error,
                status_code: _,
            } => {
                write!(f, "diesel error: {}", error)
            }
            AccountCreationError::NoTemporaryAccount {
                code,
                id,
                status_code: _,
            } => {
                write!(
                    f,
                    "no temporary account found (code: {}, temporary account id: {})",
                    code, id
                )
            }
            AccountCreationError::TemporaryAccountDuplicate {
                code: _,
                id,
                status_code: _,
            } => {
                write!(
                    f,
                    "temporary account duplicate (temporary account id: {})",
                    id
                )
            }
            AccountCreationError::TemporaryAccountExpire {
                code,
                id,
                created_at,
                activated_at,
                status_code: _,
            } => {
                write!(
                    f,
                    "temporary account already expired (code: {}, id: {}, created at {}, activated at {})",
                    code, id, created_at, activated_at
                )
            }
            AccountCreationError::AccountDuplicate {
                code,
                email_address,
                status_code: _,
            } => {
                write!(
                    f,
                    "account (\"{}\") has already existed (code: {})",
                    email_address, code
                )
            }
            AccountCreationError::EmailError {
                code: _,
                error,
                status_code: _,
            } => {
                write!(f, "failed to build email: {}", error)
            }
            AccountCreationError::SmtpError {
                code: _,
                error,
                status_code: _,
            } => {
                write!(f, "failed to send email: {}", error)
            }
        }
    }
}

impl From<diesel::result::Error> for AccountCreationError {
    fn from(error: diesel::result::Error) -> Self {
        AccountCreationError::DieselError {
            code: error::code::INTERNAL_SERVER_ERROR,
            error,
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<lettre_email::error::Error> for AccountCreationError {
    fn from(error: lettre_email::error::Error) -> Self {
        AccountCreationError::EmailError {
            code: error::code::INTERNAL_SERVER_ERROR,
            error,
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl From<lettre::smtp::error::Error> for AccountCreationError {
    fn from(error: lettre::smtp::error::Error) -> Self {
        AccountCreationError::SmtpError {
            code: error::code::INTERNAL_SERVER_ERROR,
            error,
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl error::ToCode for AccountCreationError {
    fn to_code(&self) -> u32 {
        match self {
            AccountCreationError::DieselError {
                code,
                error: _,
                status_code: _,
            } => *code,
            AccountCreationError::NoTemporaryAccount {
                code,
                id: _,
                status_code: _,
            } => *code,
            AccountCreationError::TemporaryAccountDuplicate {
                code,
                id: _,
                status_code: _,
            } => *code,
            AccountCreationError::TemporaryAccountExpire {
                code,
                id: _,
                created_at: _,
                activated_at: _,
                status_code: _,
            } => *code,
            AccountCreationError::AccountDuplicate {
                code,
                email_address: _,
                status_code: _,
            } => *code,
            AccountCreationError::EmailError {
                code,
                error: _,
                status_code: _,
            } => *code,
            AccountCreationError::SmtpError {
                code,
                error: _,
                status_code: _,
            } => *code,
        }
    }
}

impl error::ToMessage for AccountCreationError {
    fn to_message(&self) -> String {
        match self {
            AccountCreationError::DieselError {
                code: _,
                error: _,
                status_code: _,
            } => String::from(common::error::INTERNAL_SERVER_ERROR_MESSAGE),
            AccountCreationError::NoTemporaryAccount {
                code: _,
                id,
                status_code: _,
            } => {
                // TODO: httpsに更新する
                let url = format!("http://{}:{}/temporary-accounts?id={}", DOMAIN, PORT, id);
                format!("指定されたURL ({}) は存在しません。メールで届いているURLと比較し、同一であるかご確認ください。", url)
            }
            AccountCreationError::TemporaryAccountDuplicate {
                code: _,
                id: _,
                status_code: _,
            } => String::from(common::error::INTERNAL_SERVER_ERROR_MESSAGE),
            AccountCreationError::TemporaryAccountExpire {
                code: _,
                id,
                created_at: _,
                activated_at: _,
                status_code: _,
            } => {
                // TODO: httpsに更新する
                let url = format!("http://{}:{}/temporary-accounts?id={}", DOMAIN, PORT, id);
                format!("指定されたURL ({}) は有効期限が過ぎています。お手数ですが、ユーザアカウント作成より、もう一度作成手続きをお願いします。", url)
            }
            AccountCreationError::AccountDuplicate {
                code: _,
                email_address,
                status_code: _,
            } => {
                format!("{}は既に登録済です", email_address)
            }
            AccountCreationError::EmailError {
                code: _,
                error: _,
                status_code: _,
            } => String::from(common::error::INTERNAL_SERVER_ERROR_MESSAGE),
            AccountCreationError::SmtpError {
                code: _,
                error: _,
                status_code: _,
            } => String::from(common::error::INTERNAL_SERVER_ERROR_MESSAGE),
        }
    }
}

impl error::ToStatusCode for AccountCreationError {
    fn to_status_code(&self) -> StatusCode {
        match self {
            AccountCreationError::DieselError {
                code: _,
                error: _,
                status_code,
            } => *status_code,
            AccountCreationError::NoTemporaryAccount {
                code: _,
                id: _,
                status_code,
            } => *status_code,
            AccountCreationError::TemporaryAccountDuplicate {
                code: _,
                id: _,
                status_code,
            } => *status_code,
            AccountCreationError::TemporaryAccountExpire {
                code: _,
                id: _,
                created_at: _,
                activated_at: _,
                status_code,
            } => *status_code,
            AccountCreationError::AccountDuplicate {
                code: _,
                email_address: _,
                status_code,
            } => *status_code,
            AccountCreationError::EmailError {
                code: _,
                error: _,
                status_code,
            } => *status_code,
            AccountCreationError::SmtpError {
                code: _,
                error: _,
                status_code,
            } => *status_code,
        }
    }
}

fn create_error_view(err: AccountCreationError) -> HttpResponse {
    let body = format!(
        r#"<!DOCTYPE html>
    <html>
      <head>
        <meta charset="utf-8">
        <title>登録失敗</title>
      </head>
      <body>
      {}
      </body>
    </html>"#,
        err.to_message()
    );
    HttpResponse::build(StatusCode::BAD_REQUEST)
        .content_type("text/html; charset=UTF-8")
        .body(body)
}

fn send_account_creation_success_mail(email_address: &str) -> Result<(), AccountCreationError> {
    use lettre::{ClientSecurity, SmtpClient, Transport};
    use lettre_email::EmailBuilder;
    let email = EmailBuilder::new()
        .to(email_address)
        // TODO: 送信元メールを更新する
        .from("from@example.com")
        // TOOD: メールの件名を更新する
        .subject("日本語のタイトル")
        // TOOD: メールの本文を更新する
        .text("アカウントの作成に成功しました。")
        .build()?;

    use std::net::SocketAddr;
    let addr = SocketAddr::from(SMTP_SERVER_ADDR);
    let client = SmtpClient::new(addr, ClientSecurity::None)?;
    let mut mailer = client.transport();
    // TODO: メール送信後のレスポンスが必要か検討する
    let _ = mailer.send(email.into())?;
    Ok(())
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

fn delete_temporary_account(
    temp_acc_id: &str,
    conn: &PgConnection,
) -> Result<(), AccountCreationError> {
    use crate::schema::my_project_schema::user_temporary_account::dsl::*;
    // TODO: 戻り値（usize: the number of rows affected）を利用する必要があるか検討する
    let _result =
        diesel::delete(user_temporary_account.filter(temporary_account_id.eq(temp_acc_id)))
            .execute(conn)?;
    Ok(())
}

fn create_account_inner(
    mail_addr: &str,
    hashed_pwd: &[u8],
    conn: &PgConnection,
) -> Result<(), AccountCreationError> {
    conn.transaction::<_, AccountCreationError, _>(|| {
        use crate::schema::my_project_schema::user_account::dsl::*;
        let cnt = user_account
            .filter(email_address.eq(mail_addr))
            .count()
            .get_result::<i64>(conn)?;
        if cnt > 0 {
            return Err(AccountCreationError::AccountDuplicate {
                code: error::code::ACCOUNT_ALREADY_EXISTS,
                email_address: String::from(mail_addr),
                status_code: StatusCode::CONFLICT,
            });
        }
        use crate::schema::my_project_schema::user_account;
        let user = model::Account {
            email_address: mail_addr,
            hashed_password: hashed_pwd,
            last_login_time: None,
        };
        // TODO: 戻り値（usize: the number of rows affected）を利用する必要があるか検討する
        let _result = diesel::insert_into(user_account::table)
            .values(&user)
            .execute(conn)?;
        Ok(())
    })
}
