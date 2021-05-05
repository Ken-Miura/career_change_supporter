// Copyright 2021 Ken Miura

use crate::common;
use crate::common::credential;
use crate::common::error;
use crate::common::error::handled;
use crate::common::error::unexpected;

use actix_web::{post, web, Error, HttpResponse};
use diesel::prelude::*;
use futures::{StreamExt, TryStreamExt};
use once_cell::sync::Lazy;
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
    // parameterの処理
    Ok(HttpResponse::Ok().into())
}
