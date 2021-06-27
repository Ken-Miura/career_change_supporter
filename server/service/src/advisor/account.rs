// Copyright 2021 Ken Miura

use crate::common;
use crate::common::credential;
use crate::common::error;
use crate::common::error::handled;
use crate::common::error::unexpected;

use crate::common::util;
use actix_multipart::Field;
use actix_web::{post, web, HttpResponse};
use chrono::DateTime;
use diesel::prelude::*;
use futures::{StreamExt, TryStreamExt};
use rusoto_s3::S3;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str;
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

#[post("/registration-request-id-check")]
async fn registration_request_id_check(
    id_check_req: web::Json<IdCheckRequest>,
    pool: web::Data<common::ConnectionPool>,
) -> Result<HttpResponse, error::Error> {
    let email_address = check_if_id_expires_and_get_email_address(id_check_req.id.clone(), &pool)
        .await
        .map_err(|err| {
            log::error!("failed to check registration request id: {}", err);
            err
        })?;
    log::info!(
        "checked registration request (id: {}, email address: {})",
        id_check_req.id,
        email_address
    );
    Ok(HttpResponse::Ok().json(IdCheckResponse { email_address }))
}

async fn check_if_id_expires_and_get_email_address(
    id_to_check: String,
    pool: &web::Data<common::ConnectionPool>,
) -> Result<String, error::Error> {
    let _ = validate_id_format(&id_to_check)?;
    let conn = pool
        .get()
        .map_err(|err| error::Error::Unexpected(unexpected::Error::R2d2Err(err)))?;
    let current_date_time = chrono::Utc::now();
    let req_id = id_to_check.clone();
    let email_address = web::block(move || {
        // 一連の操作をトランザクションで実行はしない
        // advidsor registration requestテーブルに対してUPDATE権限を許可していないため、取得したreg_reqがdeleteされるまでに変化することはない。
        let reg_req = find_registration_req_by_id(&req_id, &conn)?;
        let time_elapsed = current_date_time - reg_req.created_at;
        if time_elapsed.num_days() > 0 {
            let _ = delete_registration_request(&req_id, &conn)?;
            let e = handled::RegistrationRequestExpired::new(
                req_id.to_string(),
                reg_req.created_at,
                current_date_time,
            );
            return Err(error::Error::Handled(
                handled::Error::RegistrationRequestExpired(e),
            ));
        }
        Ok(reg_req.email_address)
    })
    .await?;
    return Ok(email_address);
}

#[derive(Deserialize)]
struct IdCheckRequest {
    id: String,
}

#[derive(Serialize)]
struct IdCheckResponse {
    email_address: String,
}

fn validate_id_format(request_id: &str) -> Result<(), error::Error> {
    let correct_format = util::check_if_uuid_format_is_correct(request_id);
    if !correct_format {
        let e = error::Error::Handled(handled::Error::InvalidRegistrationRequestId(
            handled::InvalidRegistrationRequestId::new(request_id.to_string()),
        ));
        return Err(e);
    }
    Ok(())
}

fn find_registration_req_by_id(
    req_id: &str,
    conn: &PgConnection,
) -> Result<db::model::advisor::RegistrationRequestQueryResult, error::Error> {
    use db::schema::career_change_supporter_schema::advisor_registration_request::dsl::{
        advisor_registration_request, advisor_registration_request_id,
    };
    let registration_requests = advisor_registration_request
        .filter(advisor_registration_request_id.eq(req_id))
        .get_results::<db::model::advisor::RegistrationRequestQueryResult>(conn)?;
    if registration_requests.is_empty() {
        let e = handled::NoRegistrationRequestFound::new(req_id.to_string());
        return Err(error::Error::Handled(
            handled::Error::NoRegistrationRequestFound(e),
        ));
    }
    if registration_requests.len() != 1 {
        let e = unexpected::RegistrationRequestIdDuplicate::new(req_id.to_string());
        return Err(error::Error::Unexpected(
            unexpected::Error::RegistrationRequestIdDuplicate(e),
        ));
    }
    let reg_req = registration_requests[0].clone();
    Ok(reg_req)
}

fn delete_registration_request(req_id: &str, conn: &PgConnection) -> Result<(), error::Error> {
    use db::schema::career_change_supporter_schema::advisor_registration_request::dsl::{
        advisor_registration_request, advisor_registration_request_id,
    };
    // TODO: 戻り値 cnt（usize: the number of rows affected）を利用する必要があるか検討する
    let cnt = diesel::delete(
        advisor_registration_request.filter(advisor_registration_request_id.eq(req_id)),
    )
    .execute(conn)?;
    if cnt != 1 {
        log::warn!("diesel::delete::execute result (id: {}): {}", req_id, cnt);
    }
    Ok(())
}

// TODO: 綺麗に書き直す＋エラーハンドリングの追加
#[post("/account-creation-request")]
async fn account_creation_request(
    mut payload: actix_multipart::Multipart,
    pool: web::Data<common::ConnectionPool>,
) -> Result<HttpResponse, common::error::Error> {
    let mut submitted_data: Option<SubmittedData> = None;
    let mut image1_filename: Option<String> = None;
    let mut image2_filename: Option<String> = None;
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let name = content_type.get_name().unwrap();
        if name == "parameter" {
            // バイナリ->Stringへ変換して変数に格納
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                let parameter: String = str::from_utf8(&data).unwrap().parse().unwrap();
                submitted_data = Some(serde_json::from_str(&parameter).unwrap());
                log::info!("data: {:?}", data);
            }
        } else if name == "image1" {
            let filename = upload_to_s3_bucket(field).await;
            image1_filename = Some(filename)
        } else if name == "image2" {
            let filename = upload_to_s3_bucket(field).await;
            image2_filename = Some(filename)
        }
    }
    let id = submitted_data.clone().unwrap().id;
    let email_address = check_if_id_expires_and_get_email_address(id.clone(), &pool)
        .await
        .map_err(|err| {
            log::error!("failed to check registration request id: {}", err);
            err
        })?;
    let password = submitted_data.clone().unwrap().password;
    let hashed_password = credential::hash_password(&password).unwrap();

    let mail_addr = email_address.clone();
    let result = create_account_registration(
        email_address,
        hashed_password,
        submitted_data.unwrap(),
        image1_filename,
        image2_filename,
        &pool,
    );
    let resp = match result {
        Ok(()) => {
            log::info!("ok");
            let _ = send_notification_mail_to_admin(&mail_addr);
            Ok(HttpResponse::Ok().finish())
        }
        Err(e) => {
            log::error!("{}", e);
            Err(HttpResponse::BadRequest().finish())
        }
    };
    if resp.is_err() {
        return Ok(HttpResponse::Ok().json(AccountCreationRequestResult {
            message: "失敗".to_string()
        }));
    };
    Ok(HttpResponse::Ok().json(AccountCreationRequestResult {
        message: "成功".to_string()
    }))
}

#[derive(Serialize)]
struct AccountCreationRequestResult {
    message: String
}

fn send_notification_mail_to_admin(email_address: &str) -> Result<(), error::Error> {
    use lettre::{ClientSecurity, SmtpClient, Transport};
    use lettre_email::EmailBuilder;
    let email = EmailBuilder::new()
        // TODO: ドメイン取得後書き直し
        .to("administrator@example.com")
        // TODO: 送信元メールを更新する
        .from("from@example.com")
        // TOOD: メールの件名を更新する
        .subject("アカウント登録依頼発行")
        // TOOD: メールの本文を更新する (http -> httpsへの変更も含む)
        .text(format!(
            "{}からアドバイザーアカウント登録依頼が来ました。",
            email_address
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

const AWS_S3_ID_IMG_BUCKET_NAME: &str = "identification-images";
const AWS_REGION: &str = "ap-northeast-1";
const AWS_ENDPOINT_URL: &str = "http://localhost:4566";

async fn upload_to_s3_bucket(mut field: Field) -> String {
    let id = Uuid::new_v4().to_simple().to_string();
    // TODO: 画像の種類によって拡張子を変える
    let image_filename = format!("{}.png", &id);
    // TODO: メモリでなく、一時的に/tmp/などに保存する？（直接指定せずにenv等を通して）
    let mut contents: Vec<u8> = Vec::new();
    // バイナリをチャンクに分けてwhileループ
    while let Some(chunk) = field.next().await {
        let data = chunk.unwrap();
        contents = web::block(move || contents.write_all(&data).map(|_| contents))
            .await
            .unwrap();
    }
    let put_request = rusoto_s3::PutObjectRequest {
        bucket: AWS_S3_ID_IMG_BUCKET_NAME.to_string(),
        key: image_filename.clone(),
        body: Some(contents.into()),
        ..Default::default()
    };
    let region = rusoto_core::Region::Custom {
        name: AWS_REGION.to_string(),
        endpoint: AWS_ENDPOINT_URL.to_string(),
    };
    let s3_client = rusoto_s3::S3Client::new(region);
    let result = s3_client.put_object(put_request).await;
    let output = result.unwrap();
    log::info!("{:?}", output);
    return image_filename;
}

#[derive(Deserialize, Debug, Clone)]
struct SubmittedData {
    id: String,
    password: String,
    last_name: String,
    first_name: String,
    last_name_furigana: String,
    first_name_furigana: String,
    telephone_number: String,
    year_of_birth: i16,
    month_of_birth: i16,
    day_of_birth: i16,
    prefecture: String,
    city: String,
    address_line1: String,
    address_line2: Option<String>,
}

fn create_account_registration(
    mail_addr: String,
    hashed_passwoed: Vec<u8>,
    submitted_data: SubmittedData,
    image1: Option<String>,
    image2: Option<String>,
    pool: &web::Data<common::ConnectionPool>,
) -> Result<(), common::error::Error> {
    let conn = pool
        .get()
        .map_err(|err| error::Error::Unexpected(unexpected::Error::R2d2Err(err)))?;
    conn.transaction::<_, error::Error, _>(|| {
            let _r = check_if_account_exists(&mail_addr, &conn)?;
            use db::schema::career_change_supporter_schema::advisor_account_creation_request::dsl::{
                advisor_account_creation_request, email_address
            };
            let email_addrs = advisor_account_creation_request
            .select(email_address).filter(email_address.eq(mail_addr.clone())).load::<String>(&conn)?;
            if !email_addrs.is_empty() {
                panic!("email ({}) already exist", mail_addr.clone());
            }

            use db::schema::career_change_supporter_schema::advisor_account_creation_request as aac_request;
            let address_line2 = submitted_data.address_line2.clone().unwrap();
            let i1 = image1.unwrap();
            let i2 = image2.unwrap();
            let current_date_time = chrono::Utc::now();
            let aac_req = create_acc_request(&mail_addr, &hashed_passwoed, &submitted_data, &address_line2, &i1, &i2, &current_date_time);
            // TODO: 戻り値（usize: the number of rows affected）を利用する必要があるか検討する
            let _result = diesel::insert_into(aac_request::table)
                .values(aac_req)
                .execute(&conn)
                .map_err(|e| error::Error::Unexpected(unexpected::Error::DieselResultErr(e)))?;

            Ok(())
        })?;
    Ok(())
}

fn create_acc_request<'a>(
    mail_addr: &'a str,
    hashed_passwoed: &'a [u8],
    submitted_data: &'a SubmittedData,
    address_line2: &'a str,
    image1: &'a str,
    image2: &'a str,
    current_date_time: &'a DateTime<chrono::Utc>,
) -> db::model::advisor::AccountCreationRequest<'a> {
    db::model::advisor::AccountCreationRequest {
        email_address: mail_addr,
        hashed_password: hashed_passwoed,
        last_name: &submitted_data.last_name,
        first_name: &submitted_data.first_name,
        last_name_furigana: &submitted_data.last_name_furigana,
        first_name_furigana: &submitted_data.first_name_furigana,
        telephone_number: &submitted_data.telephone_number,
        year_of_birth: submitted_data.year_of_birth,
        month_of_birth: submitted_data.month_of_birth,
        day_of_birth: submitted_data.day_of_birth,
        prefecture: &submitted_data.prefecture,
        city: &submitted_data.city,
        address_line1: &submitted_data.address_line1,
        address_line2: Some(address_line2),
        image1,
        image2: Some(image2),
        requested_time: &current_date_time,
    }
}
