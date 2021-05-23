// Copyright 2021 Ken Miura

use crate::common;
use crate::common::error;
use crate::common::error::handled;
use crate::common::error::unexpected;

use crate::common::util;
use actix_web::{post, web, HttpResponse};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// TODO: 運用しながら上限を調整する
const REGISTRATION_REQUEST_LIMIT: i64 = 7;

#[derive(Deserialize)]
pub(crate) struct RegistrationRequest {
    pub(crate) email_address: String,
}

// TODO: 有効期限切れのregistration requestを自動で削除する仕組みを検討、導入する
#[post("/registration-request")]
async fn registration_request(
    registration_req: web::Json<RegistrationRequest>,
    pool: web::Data<common::ConnectionPool>,
) -> Result<HttpResponse, error::Error> {
    let _ = util::validate_email_address(&registration_req.email_address).map_err(|err| {
        let e = error::Error::Handled(err);
        log::error!("failed to process advisor registration request: {}", e);
        e
    })?;

    let conn = pool.get().map_err(|err| {
        let e = error::Error::Unexpected(unexpected::Error::R2d2Err(err));
        log::error!("failed to process advisor registration request: {}", e);
        e
    })?;
    let registration_req_id = Uuid::new_v4().to_simple().to_string();
    let id_cloned = registration_req_id.clone();
    let mail_addr = registration_req.email_address.clone();
    let current_date_time = chrono::Utc::now();
    let result = web::block(move || {
        insert_registration_request_id(id_cloned, mail_addr, current_date_time, &conn)
    })
    .await;
    let registration_req_cnt = result.map_err(|err| {
        let e = error::Error::from(err);
        log::error!("failed to process advisor registration request: {}", e);
        e
    })?;
    let mut message = format!(
        "{}宛に登録用URLを送りました。登録用URLにアクセスし、必要な内容を記入して登録を完了させてください（登録用URLの有効期間は24時間です）",
        registration_req.email_address
    );
    if registration_req_cnt > 1 {
        message = format!(
            "{}。メールが届かない場合、迷惑メールフォルダに届いていないか、このサイトのドメインのメールが受信許可されているかをご確認ください。",
            message
        )
    }
    let _ = send_notification_mail(&registration_req.email_address, &registration_req_id).map_err(
        |e| {
            log::error!("failed to process advisor registration request: {}", e);
            e
        },
    )?;
    log::info!("processed advisor registration request (registration request id: {}, email address: {}) at {}", registration_req_id, registration_req.email_address, current_date_time);
    Ok(HttpResponse::Ok().json(RegistrationRequestResult {
        email_address: registration_req.email_address.clone(),
        message,
    }))
}

fn insert_registration_request_id(
    request_id: String,
    mail_addr: String,
    current_date_time: chrono::DateTime<chrono::Utc>,
    conn: &PgConnection,
) -> Result<i64, error::Error> {
    conn.transaction::<_, error::Error, _>(|| {
        check_if_account_exists(&mail_addr, conn)?;
        let cnt = num_of_registration_requests(&mail_addr, conn)?;
        if cnt >= REGISTRATION_REQUEST_LIMIT {
            return Err(error::Error::Handled(
                handled::Error::ReachLimitOfRegistrationRequest(
                    handled::ReachLimitOfRegistrationRequest::new(mail_addr, cnt),
                ),
            ));
        }
        use db::schema::career_change_supporter_schema::advisor_registration_request;
        let registration_req = db::model::advisor::RegistrationRequest {
            advisor_registration_request_id: &request_id,
            email_address: &mail_addr,
            created_at: &current_date_time,
        };
        // TODO: 戻り値（usize: the number of rows affected）を利用する必要があるか検討する
        let _result = diesel::insert_into(advisor_registration_request::table)
            .values(registration_req)
            .execute(conn)
            .map_err(|e| error::Error::Unexpected(unexpected::Error::DieselResultErr(e)))?;
        Ok(cnt)
    })
}

fn check_if_account_exists(mail_addr: &str, conn: &PgConnection) -> Result<(), error::Error> {
    use db::schema::career_change_supporter_schema::advisor_account::dsl::{
        advisor_account, email_address,
    };
    let cnt = advisor_account
        .filter(email_address.eq(mail_addr))
        .count()
        .get_result::<i64>(conn)
        .map_err(|e| error::Error::Unexpected(unexpected::Error::DieselResultErr(e)))?;
    if cnt > 1 {
        return Err(error::Error::Unexpected(
            unexpected::Error::AdvisorAccountDuplicate(unexpected::AdvisorAccountDuplicate::new(
                mail_addr.to_string(),
            )),
        ));
    }
    if cnt == 1 {
        return Err(error::Error::Handled(
            handled::Error::AdvisorAccountAlreadyExists(handled::AdvisorAccountAlreadyExists::new(
                mail_addr.to_string(),
            )),
        ));
    }
    // TODO: 念の為、負の数のケースを考える必要があるか調べる
    Ok(())
}

fn num_of_registration_requests(mail_addr: &str, conn: &PgConnection) -> Result<i64, error::Error> {
    use db::schema::career_change_supporter_schema::advisor_registration_request::dsl::{
        advisor_registration_request, email_address,
    };
    let cnt = advisor_registration_request
        .filter(email_address.eq(mail_addr))
        .count()
        .get_result::<i64>(conn)
        .map_err(|e| error::Error::Unexpected(unexpected::Error::DieselResultErr(e)))?;
    Ok(cnt)
}

fn send_notification_mail(
    email_address: &str,
    registration_req_id: &str,
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
            r"下記のURLにアクセスし、必要な内容を記入して登録を完了させてください（URLの有効期間は24時間です）
            http://{}:{}/advisor/registration-requests?id={}",
            common::DOMAIN,
            common::PORT,
            registration_req_id
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
struct RegistrationRequestResult {
    email_address: String,
    message: String,
}
