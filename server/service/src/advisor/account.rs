// Copyright 2021 Ken Miura

use crate::advisor::authentication::check_advisor_session_state;
use crate::common;
use crate::common::credential;
use crate::common::error;
use crate::common::error::handled;
use crate::common::error::unexpected;
use actix_session::Session;
use openssl::ssl::{SslConnector, SslMethod};
use std::collections::HashMap;

use crate::common::util;
use actix_multipart::Field;
use actix_web::{client, http::StatusCode, post, web, HttpResponse};
use chrono::DateTime;
use db::model::administrator::AdvisorCareerCreateReq;
use db::model::administrator::AdvisorRegReqApprovedResultForCareerReq;
use diesel::prelude::*;
use futures::{StreamExt, TryStreamExt};
use rusoto_s3::S3;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str;
use uuid::Uuid;

use diesel::QueryDsl;
use diesel::RunQueryDsl;

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
    Ok(email_address)
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
            let filename = upload_to_s3_bucket(field, AWS_S3_ID_IMG_BUCKET_NAME).await;
            image1_filename = Some(filename)
        } else if name == "image2" {
            let filename = upload_to_s3_bucket(field, AWS_S3_ID_IMG_BUCKET_NAME).await;
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
    let result = web::block(move || {
        create_account_registration(
            email_address,
            hashed_password,
            submitted_data.unwrap(),
            image1_filename,
            image2_filename,
            &pool,
        )
    })
    .await;
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
            message: "失敗".to_string(),
        }));
    };
    Ok(HttpResponse::Ok().json(AccountCreationRequestResult {
        message: "成功".to_string(),
    }))
}

#[derive(Serialize)]
struct AccountCreationRequestResult {
    message: String,
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

async fn upload_to_s3_bucket(mut field: Field, bucket_name: &str) -> String {
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
        bucket: bucket_name.to_string(),
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
    image_filename
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
    year_of_birth: i32,
    month_of_birth: u32,
    day_of_birth: u32,
    prefecture: String,
    city: String,
    address_line1: String,
    address_line2: Option<String>,
    sex: String,
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
    let date_of_birth = chrono::NaiveDate::from_ymd(
        submitted_data.year_of_birth,
        submitted_data.month_of_birth,
        submitted_data.day_of_birth,
    );
    db::model::advisor::AccountCreationRequest {
        email_address: mail_addr,
        hashed_password: hashed_passwoed,
        last_name: &submitted_data.last_name,
        first_name: &submitted_data.first_name,
        last_name_furigana: &submitted_data.last_name_furigana,
        first_name_furigana: &submitted_data.first_name_furigana,
        telephone_number: &submitted_data.telephone_number,
        date_of_birth,
        prefecture: &submitted_data.prefecture,
        city: &submitted_data.city,
        address_line1: &submitted_data.address_line1,
        address_line2: Some(address_line2),
        sex: &submitted_data.sex,
        image1,
        image2: Some(image2),
        requested_time: current_date_time,
    }
}

#[post("/bank-info")]
async fn bank_info(
    tenant_req: web::Json<TenantRequest>,
    session: Session,
    pool: web::Data<common::ConnectionPool>,
) -> Result<HttpResponse, common::error::Error> {
    let option_id = check_advisor_session_state(&session)?;
    let id = option_id.expect("Failed to get id");

    let tenant_change_request = TenantChangeRequest {
        bank_account_holder_name: tenant_req.bank_account_holder_name.clone(),
        bank_code: tenant_req.bank_code.clone(),
        bank_branch_code: tenant_req.bank_branch_code.clone(),
        bank_account_number: tenant_req.bank_account_number.clone(),
    };

    // TODO: どのような設定に気をつけなければならないか確認する
    // https://github.com/actix/examples/tree/master/security/awc_https
    let ssl_builder = SslConnector::builder(SslMethod::tls()).unwrap();
    let client_builder = client::ClientBuilder::new();
    let secret_key = common::PAYJP_TEST_SECRET_KEY.to_string();
    let password = common::PAYJP_TEST_PASSWORD.to_string();
    let client = client_builder
        .connector(client::Connector::new().ssl(ssl_builder.build()).finish())
        .basic_auth(&secret_key, Some(&password))
        .finish();

    let c = pool.get().expect("Failed to get connection");
    use crate::common::util;
    let _ = util::transaction(&c, async {
        use db::schema::career_change_supporter_schema::advisor_account::dsl::{
            advisor_account, tenant_id,
        };
        let result: Result<db::model::advisor::AccountQueryResult, diesel::result::Error> =
            advisor_account
                .find(id)
                .first::<db::model::advisor::AccountQueryResult>(&c);
        let acc = result.expect("Failed to get account");
        match acc.clone().tenant_id {
            Some(t_id) => {
                // Update teanant
                // Create request builder and send request
                let result = client
                    .post(format!("https://api.pay.jp/v1/tenants/{}", t_id))
                    .send_form(&tenant_change_request)
                    .await; // <- Wait for response

                // https://github.com/actix/actix-web/issues/536#issuecomment-579380701
                let mut response = result.expect("test");
                if response.status() != StatusCode::OK {
                    let result = response.json::<ErrorSt>().await;
                    // エラーならロールバック
                    let tenant = result.expect("test");
                    log::info!("{:?}", tenant);
                    return Err(diesel::result::Error::RollbackTransaction);
                }
                let result = response.json::<Tenant>().await;
                // エラーならロールバック
                let _tenant = result.expect("test");
            }
            None => {
                // Createa teanant
                let t_id = Uuid::new_v4().to_simple().to_string();
                let _res = diesel::update(advisor_account.find(id))
                    .set(tenant_id.eq(t_id.clone()))
                    .get_results::<db::model::advisor::AccountQueryResult>(&c)?;

                let tenant_create_request = TenantCreateRequest {
                    id: t_id,
                    name: tenant_req.bank_account_holder_name.clone(),
                    platform_fee_rate: "10.15".to_string(),
                    minimum_transfer_amount: "1000".to_string(),
                    bank_account_holder_name: tenant_req.bank_account_holder_name.clone(),
                    bank_code: tenant_req.bank_code.clone(),
                    bank_branch_code: tenant_req.bank_branch_code.clone(),
                    bank_account_type: "普通".to_string(),
                    bank_account_number: tenant_req.bank_account_number.clone(),
                };
                // Create request builder and send request
                let result = client
                    .post("https://api.pay.jp/v1/tenants")
                    .send_form(&tenant_create_request)
                    .await; // <- Wait for response

                // https://github.com/actix/actix-web/issues/536#issuecomment-579380701
                let mut response = result.expect("test");
                if response.status() != StatusCode::OK {
                    let result = response.json::<ErrorSt>().await;
                    // エラーならロールバック
                    let tenant = result.expect("test");
                    log::info!("{:?}", tenant);
                    return Err(diesel::result::Error::RollbackTransaction);
                }
                let result = response.json::<Tenant>().await;
                // エラーならロールバック
                let _tenant = result.expect("test");
            }
        }
        Ok::<_, diesel::result::Error>(())
    })
    .await;

    // parameterの処理
    Ok(HttpResponse::Ok().into())
}

#[post("/advice-fee")]
async fn advice_fee(
    advice_fee_req: web::Json<AdviceFeeRequest>,
    session: Session,
    pool: web::Data<common::ConnectionPool>,
) -> Result<HttpResponse, common::error::Error> {
    let option_id = check_advisor_session_state(&session)?;
    let id = option_id.expect("Failed to get id");

    let advice_fee = advice_fee_req.advice_fee;
    log::info!("advice fee: {}", advice_fee);
    let conn = pool.get().expect("Failed to get connection");
    let result = web::block(move || {
        conn.transaction::<_, diesel::result::Error, _>(|| {
            use db::schema::career_change_supporter_schema::advisor_account::dsl::{
                advice_fee_in_yen, advisor_account,
            };
            let _result = diesel::update(advisor_account.find(id))
                .set(advice_fee_in_yen.eq(advice_fee))
                .get_results::<db::model::advisor::AccountQueryResult>(&conn)?;
            Ok(())
        })
    })
    .await;
    let _a = result.expect("Failed to get result");
    Ok(HttpResponse::Ok().finish())
}

#[derive(Deserialize)]
pub(in crate::advisor) struct AdviceFeeRequest {
    advice_fee: i32,
}

#[derive(Deserialize)]
pub(in crate::advisor) struct TenantRequest {
    bank_code: String,
    bank_branch_code: String,
    bank_account_number: String,
    bank_account_holder_name: String,
}

#[derive(Serialize)]
struct TenantCreateRequest {
    id: String,
    name: String,
    platform_fee_rate: String,
    minimum_transfer_amount: String,
    bank_account_holder_name: String,
    bank_code: String,
    bank_branch_code: String,
    bank_account_type: String,
    bank_account_number: String,
}

#[derive(Serialize)]
struct TenantChangeRequest {
    bank_account_holder_name: String,
    bank_code: String,
    bank_branch_code: String,
    bank_account_number: String,
}

#[derive(Debug, Deserialize)]
pub(in crate::advisor) struct Tenant {
    id: String,
    object: String,
    livemode: bool,
    created: i64,
    platform_fee_rate: String,
    payjp_fee_included: bool,
    minimum_transfer_amount: i32,
    pub(in crate::advisor) bank_code: String,
    pub(in crate::advisor) bank_branch_code: String,
    bank_account_type: String,
    pub(in crate::advisor) bank_account_number: String,
    pub(in crate::advisor) bank_account_holder_name: String,
    bank_account_status: String,
    currencies_supported: Vec<String>,
    default_currency: String,
    reviewed_brands: Vec<ReviewedBrands>,
    // nullのときNoneになる。Optionで囲んでなければnullのときpanic
    // TODO: metadataの方がHashMap<String, String>でよいか確認する
    // https://payjp.hatenablog.com/entry/2016/02/22/100000
    metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct ReviewedBrands {
    brand: String,
    status: String,
    // nullのときNoneになる。Optionで囲んでなければnullのときpanic
    available_date: Option<i64>,
}

// {
//     "error": {
//       "message": "There is no tenant with ID: dummy",
//       "param": "id",
//       "status": 404,
//       "type": "client_error"
//     }
//   }
#[derive(Debug, Deserialize)]
pub(in crate::advisor) struct ErrorSt {
    pub(in crate::advisor) error: Error,
}

#[derive(Debug, Deserialize)]
pub(in crate::advisor) struct Error {
    pub(in crate::advisor) message: String,
    pub(in crate::advisor) param: String,
    pub(in crate::advisor) status: i32,
    pub(in crate::advisor) r#type: String,
}

// 経歴の作成依頼を行う
#[post("/career-registeration")]
async fn career_registeration(
    session: Session,
    mut payload: actix_multipart::Multipart,
    pool: web::Data<common::ConnectionPool>,
) -> Result<HttpResponse, common::error::Error> {
    // セッションチェック
    // idから最新のアカウント承認記録を取得
    // ペイロードからデータを取得
    // ペイロードと承認記録を承認依頼DBに記録＋S3へ画像のアップロード（transaction内で実施）

    let option_id = check_advisor_session_state(&session)?;
    let id = option_id.expect("Failed to get id");
    let conn = pool.get().expect("failed to get connection");
    use db::schema::career_change_supporter_schema::advisor_reg_req_approved::dsl::{
        advisor_reg_req_approved, advisor_reg_req_approved_id, approved_time,
        associated_advisor_account_id,
    };
    let selected = advisor_reg_req_approved.select((
        advisor_reg_req_approved_id,
        associated_advisor_account_id,
        approved_time,
    ));
    let result = selected
        .filter(associated_advisor_account_id.eq(id))
        .order(approved_time.desc())
        .limit(1)
        .load::<AdvisorRegReqApprovedResultForCareerReq>(&conn);
    let v = result.expect("failed to get vector");
    if v.is_empty() {
        panic!("empty");
    }
    let req = v[0].clone();

    let mut submitted_career: Option<SubmittedCareer> = None;
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
                submitted_career = Some(serde_json::from_str(&parameter).unwrap());
                log::info!("data: {:?}", data);
            }
        } else if name == "image1" {
            let filename =
                upload_to_s3_bucket(field, AWS_S3_CARER_CONFIRMATION_IMG_BUCKET_NAME).await;
            image1_filename = Some(filename)
        } else if name == "image2" {
            let filename =
                upload_to_s3_bucket(field, AWS_S3_CARER_CONFIRMATION_IMG_BUCKET_NAME).await;
            image2_filename = Some(filename)
        }
    }

    let result = web::block(move || {
        create_career_registration(
            req,
            submitted_career.expect("failed to get career"),
            image1_filename,
            image2_filename,
            &conn,
        )
    })
    .await;
    let resp = match result {
        Ok(()) => {
            log::info!("ok");
            let _ = send_career_notification_mail_to_admin();
            Ok(HttpResponse::Ok().finish())
        }
        Err(e) => {
            log::error!("{}", e);
            Err(HttpResponse::BadRequest().finish())
        }
    };
    if resp.is_err() {
        return Ok(HttpResponse::Ok().json(AccountCreationRequestResult {
            message: "失敗".to_string(),
        }));
    };
    Ok(HttpResponse::Ok().json(AccountCreationRequestResult {
        message: "成功".to_string(),
    }))
}

const AWS_S3_CARER_CONFIRMATION_IMG_BUCKET_NAME: &str = "career-confirmation-images";

fn create_career_registration(
    req: AdvisorRegReqApprovedResultForCareerReq,
    submitted_career: SubmittedCareer,
    image1: Option<String>,
    image2: Option<String>,
    conn: &PgConnection,
) -> Result<(), common::error::Error> {
    conn.transaction::<_, error::Error, _>(|| {
        // TODO: 経歴の最大件数を考える

        use db::schema::career_change_supporter_schema::advisor_career_create_req;
        let current_date_time = chrono::Utc::now();
        let i1 = image1.expect("failed to get image1");
        let i2 = image2.expect("failed to get image2");
        let career_req = create_career_req(&req, &submitted_career, &i1, &i2, &current_date_time);
        // TODO: 戻り値（usize: the number of rows affected）を利用する必要があるか検討する
        let _result = diesel::insert_into(advisor_career_create_req::table)
            .values(career_req)
            .execute(conn)
            .map_err(|e| error::Error::Unexpected(unexpected::Error::DieselResultErr(e)))?;

        Ok(())
    })?;
    Ok(())
}

#[derive(Deserialize, Debug, Clone)]
struct SubmittedCareer {
    company_name: String,
    department_name: String,
    office: String,
    contract_type: String,
    profession: String,
    is_manager: bool,
    position_name: String,
    start_year: i32,
    start_month: u32,
    start_day: u32,
    end_year: i32,
    end_month: u32,
    end_day: u32,
    annual_income_in_man_yen: i32,
    is_new_graduate: bool,
    note: String,
}

fn create_career_req<'a>(
    req: &'a AdvisorRegReqApprovedResultForCareerReq,
    submitted_career: &'a SubmittedCareer,
    image1: &'a str,
    image2: &'a str,
    current_date_time: &'a DateTime<chrono::Utc>,
) -> AdvisorCareerCreateReq<'a> {
    let start_date = chrono::NaiveDate::from_ymd(
        submitted_career.start_year,
        submitted_career.start_month,
        submitted_career.start_day,
    );
    let end_date = Some(chrono::NaiveDate::from_ymd(
        submitted_career.end_year,
        submitted_career.end_month,
        submitted_career.end_day,
    ));
    AdvisorCareerCreateReq {
        cre_req_adv_acc_id: req.advisor_reg_req_approved_id,
        company_name: &submitted_career.company_name,
        department_name: Some(&submitted_career.department_name),
        office: Some(&submitted_career.office),
        contract_type: &submitted_career.contract_type,
        profession: Some(&submitted_career.profession),
        is_manager: submitted_career.is_manager,
        position_name: Some(&submitted_career.position_name),
        start_date,
        end_date,
        annual_income_in_man_yen: Some(submitted_career.annual_income_in_man_yen),
        is_new_graduate: submitted_career.is_new_graduate,
        note: Some(&submitted_career.note),
        image1,
        image2: Some(image2),
        requested_time: current_date_time,
    }
}

/// 経歴を編集する
/// https://actix.rs/docs/url-dispatch/
#[post("/career-registeration/{id}")]
async fn career_registeration_id(
    web::Path(_path): web::Path<String>,
    _tenant_req: web::Json<TenantRequest>,
    session: Session,
    _pool: web::Data<common::ConnectionPool>,
) -> Result<HttpResponse, common::error::Error> {
    let option_id = check_advisor_session_state(&session)?;
    let _id = option_id.expect("Failed to get id");
    Ok(HttpResponse::Ok().finish())
}

fn send_career_notification_mail_to_admin() -> Result<(), error::Error> {
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
        .text("経歴作成依頼が来ました")
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
