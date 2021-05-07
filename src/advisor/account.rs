// Copyright 2021 Ken Miura

use crate::common;
use crate::common::credential;
use crate::common::error;
use crate::common::error::handled;
use crate::common::error::unexpected;
use std::collections::HashMap;

use actix_web::{client, post, web, Error, HttpResponse};
use diesel::prelude::*;
use futures::{StreamExt, TryStreamExt};
use once_cell::sync::Lazy;
use openssl::ssl::{SslConnector, SslMethod};
use regex::Regex;
use rusoto_core;
use rusoto_s3;
use rusoto_s3::S3;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::str;
use uuid::Uuid;

#[post("/file-upload-request")]
async fn file_upload_request(
    mut payload: actix_multipart::Multipart,
) -> Result<HttpResponse, Error> {
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let name = content_type.get_name().unwrap();
        // テキストパラメータ
        if name == "parameter" {
            // バイナリ->Stringへ変換して変数に格納
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                let parameter: String = str::from_utf8(&data).unwrap().parse().unwrap();
                log::info!("param: {}", parameter);
            }
        // ファイルデータ
        } else if name == "file" {
            let filename = content_type.get_filename().unwrap();
            //let filepath = format!("./tmp/{}", filename); //sanitize_filename::sanitize(&filename));
            //
            //            // ファイル作成
            //            let mut f = web::block(|| std::fs::File::create(filepath))
            //                .await
            //                .unwrap();
            //
            //            // バイナリをチャンクに分けてwhileループ
            //            while let Some(chunk) = field.next().await {
            //                let data = chunk.unwrap();
            //                // ファイルへの書き込み
            //                f = web::block(move || f.write_all(&data).map(|_| f)).await?;
            //            }

            let mut contents: Vec<u8> = Vec::new();
            // バイナリをチャンクに分けてwhileループ
            while let Some(chunk) = field.next().await {
                let data = chunk.unwrap();
                // ファイルへの書き込み
                contents = web::block(move || contents.write_all(&data).map(|_| contents)).await?;
            }
            let put_request = rusoto_s3::PutObjectRequest {
                bucket: std::env::var("AWS_S3_BUCKET_NAME").unwrap(),
                key: format!("directory_from_program/{}", filename),
                body: Some(contents.into()),
                ..Default::default()
            };

            let region = rusoto_core::Region::Custom {
                name: "ap-northeast-1".to_string(),
                endpoint: "http://localhost:4566".to_string(),
            };
            let s3_client = rusoto_s3::S3Client::new(region);
            let result = s3_client.put_object(put_request).await;
            let output = result.expect("test");
            log::info!("output: {:?}", output);
        }
    }

    let tenant_create_request = TenantCreateRequest {
        id: "test".to_string(),
        name: "test".to_string(),
        platform_fee_rate: "10.15".to_string(),
        minimum_transfer_amount: "1000".to_string(),
        bank_account_holder_name: "ヤマダ タロウ".to_string(),
        bank_code: "0001".to_string(),
        bank_branch_code: "001".to_string(),
        bank_account_type: "普通".to_string(),
        bank_account_number: "0001000".to_string(),
    };

    // TODO: どのような設定に気をつけなければならないか確認する
    // https://github.com/actix/examples/tree/master/security/awc_https
    let ssl_builder = SslConnector::builder(SslMethod::tls()).unwrap();
    let client_builder = client::ClientBuilder::new();
    let client = client_builder
        .connector(client::Connector::new().ssl(ssl_builder.build()).finish())
        .basic_auth("テスト環境の秘密鍵", Some("パスワード"))
        .finish();

    // Create request builder and send request
    let result = client
        .post("https://api.pay.jp/v1/tenants")
        .send_form(&tenant_create_request)
        .await; // <- Wait for response

    // https://github.com/actix/actix-web/issues/536#issuecomment-579380701
    let mut response = result.expect("test");
    let result = response.json::<Tenant>().await;
    let tenant = result.expect("test");

    log::info!("Response body: {:?}", tenant);
    
    // parameterの処理
    Ok(HttpResponse::Ok().into())
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

#[derive(Debug, Deserialize)]
struct Tenant {
	id: String,
	object: String,
	livemode: bool,
    created: i64,
    platform_fee_rate: String,
    payjp_fee_included: bool,
    minimum_transfer_amount: i32,
    bank_code: String,
    bank_branch_code: String,    
    bank_account_type: String,
    bank_account_number: String,
    bank_account_holder_name: String,
    bank_account_status: String,
    currencies_supported: Vec<String>,
    default_currency: String,
    reviewed_brands: Vec<ReviewedBrands>,
    // TODO: nullのケースが、Some(`{})になるのか、Noneになるのか確認
    metadata: Option<HashMap<String, String>>
}

#[derive(Debug, Deserialize)]
struct ReviewedBrands {
	brand: String,
	status: String,
	// TODO: nullのとき、Noneになるのか確認
	available_date: Option<i64>
}
