// Copyright 2021 Ken Miura

use crate::common;
use crate::common::credential;
use crate::common::error;
use crate::common::error::handled;
use crate::common::error::unexpected;

use actix_web::{post, web, HttpResponse};
use diesel::prelude::*;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// TODO: 運用しながら上限を調整する
const TEMPORARY_ACCOUNT_LIMIT: i64 = 7;
const UUID_REGEXP: &str = "^[a-zA-Z0-9]{32}$";

static UUID_RE: Lazy<Regex> = Lazy::new(|| Regex::new(UUID_REGEXP).expect("never happens panic"));

// TODO: 有効期限切れのtemporary accountを自動で削除する仕組みを検討、導入する
#[post("/temporary-account-creation")]
async fn temporary_account_creation(
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
    let hashed_pwd = credential::hash_password(&credential.password).map_err(|err| {
        let e = error::Error::Unexpected(err);
        log::error!("failed to create temporary account: {}", e);
        e
    })?;
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
    log::info!("created user temporary account (temporary account id: {}, email address: {}) at {}", temp_acc_id, credential.email_address, current_date_time);
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
        use db::schema::career_change_supporter_schema::user_temporary_account;
        let temp_acc = db::model::user::TemporaryAccount {
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
    use db::schema::career_change_supporter_schema::user_account::dsl::{
        email_address, user_account,
    };
    let cnt = user_account
        .filter(email_address.eq(mail_addr))
        .count()
        .get_result::<i64>(conn)
        .map_err(|e| error::Error::Unexpected(unexpected::Error::DieselResultErr(e)))?;
    if cnt > 1 {
        return Err(error::Error::Unexpected(
            unexpected::Error::UserAccountDuplicate(unexpected::UserAccountDuplicate::new(
                mail_addr.to_string(),
            )),
        ));
    }
    if cnt == 1 {
        return Err(error::Error::Handled(
            handled::Error::UserAccountAlreadyExists(handled::UserAccountAlreadyExists::new(
                mail_addr.to_string(),
            )),
        ));
    }
    // TODO: 念の為、負の数のケースを考える必要があるか調べる
    Ok(())
}

fn num_of_temporary_accounts(mail_addr: &str, conn: &PgConnection) -> Result<i64, error::Error> {
    use db::schema::career_change_supporter_schema::user_temporary_account::dsl::{
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
            http://{}:{}/user/temporary-accounts?id={}",
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
#[post("/account-creation")]
async fn account_creation(
    account_req: web::Json<AccountRequest>,
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
        conn.transaction::<_, error::Error, _>(|| {
            let temp_acc =
                check_and_delete_temporary_account(&temp_acc_id, current_date_time, &conn)?;
            let user = create_account(&temp_acc.email_address, &temp_acc.hashed_password, &conn)?;
            Ok(user)
        })
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
        "created user account (user account id: {}, email address: {})",
        user_acc.user_account_id,
        user_acc.email_address
    );
    let message =
        "アカウント作成が完了しました。ログイン画面よりログインをしてください。".to_string();
    Ok(HttpResponse::Ok().json(AccountResult { message }))
}

#[derive(Deserialize)]
struct AccountRequest {
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

fn check_and_delete_temporary_account(
    temporary_account_id: &str,
    current_date_time: chrono::DateTime<chrono::Utc>,
    conn: &PgConnection,
) -> Result<db::model::user::TemporaryAccountQueryResult, error::Error> {
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
    Ok(temp_acc)
}

fn find_temporary_account_by_id(
    temp_acc_id: &str,
    conn: &PgConnection,
) -> Result<db::model::user::TemporaryAccountQueryResult, error::Error> {
    use db::schema::career_change_supporter_schema::user_temporary_account::dsl::*;
    let users = user_temporary_account
        .filter(user_temporary_account_id.eq(temp_acc_id))
        .get_results::<db::model::user::TemporaryAccountQueryResult>(conn)?;
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

fn delete_temporary_account(temp_acc_id: &str, conn: &PgConnection) -> Result<(), error::Error> {
    use db::schema::career_change_supporter_schema::user_temporary_account::dsl::{
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
) -> Result<db::model::user::AccountQueryResult, error::Error> {
    use db::schema::career_change_supporter_schema::user_account::dsl::{
        email_address, user_account,
    };
    let cnt = user_account
        .filter(email_address.eq(mail_addr))
        .count()
        .get_result::<i64>(conn)?;
    if cnt > 0 {
        let e = unexpected::UserAccountDuplicate::new(mail_addr.to_string());
        return Err(error::Error::Unexpected(
            unexpected::Error::UserAccountDuplicate(e),
        ));
    }
    use db::schema::career_change_supporter_schema::user_account as user_acc;
    let user = db::model::user::Account {
        email_address: mail_addr,
        hashed_password: hashed_pwd,
        last_login_time: None,
    };
    let users = diesel::insert_into(user_acc::table)
        .values(&user)
        .get_results::<db::model::user::AccountQueryResult>(conn)?;
    if users.len() > 1 {
        return Err(error::Error::Unexpected(
            unexpected::Error::UserAccountDuplicate(unexpected::UserAccountDuplicate::new(
                mail_addr.to_string(),
            )),
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

#[derive(Serialize)]
struct AccountResult {
    message: String,
}
