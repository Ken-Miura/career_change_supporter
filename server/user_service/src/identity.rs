// Copyright 2021 Ken Miura

use std::io::Cursor;

use async_session::serde_json;
use axum::{
    extract::{ContentLengthLimit, Multipart},
    http::StatusCode,
    Json,
};
use chrono::NaiveDate;
use common::RespResult;
use image::ImageFormat;
use serde::Serialize;

use crate::util::{session::User, validator::identity_validator::validate_identity, Identity};

pub(crate) async fn post_identity(
    User { account_id }: User,
    ContentLengthLimit(mut multipart): ContentLengthLimit<
        Multipart,
        {
            9 * 1024 * 1024 /* 9mb */
        },
    >,
) -> RespResult<IdentityResult> {
    println!("account id: {}", account_id);
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap().to_string();
        let file_name_option = field.file_name();
        if let Some(file_name) = file_name_option {
            println!("file name:  `{}`", file_name);
        }
        let data = field.bytes().await.unwrap();
        println!("Length of `{}` is {} bytes", name, data.len());
        if name == "identity" {
            let identity_str = std::str::from_utf8(&data)
                .unwrap()
                .parse::<String>()
                .unwrap();
            let identity = serde_json::from_str::<Identity>(&identity_str).unwrap();
            // validate, trim
            let current_date = NaiveDate::from_ymd(2022, 1, 30);
            let _ = validate_identity(&identity, &current_date).map_err(|e| println!("{:?}", e));
            println!("identity:  `{:?}`", identity);
        } else if name == "identity-image1" {
            println!("identity-image1");
            let img = image::io::Reader::with_format(Cursor::new(data), ImageFormat::Jpeg)
                .decode()
                .expect("failed to decode");
            let mut bytes: Vec<u8> = Vec::new();
            img.write_to(&mut bytes, image::ImageOutputFormat::Png)
                .expect("failed to write_to");
            let magic_number_option = bytes.get(0..8);
            if let Some(magic_number) = magic_number_option {
                println!("magic_number: ");
                for n in magic_number {
                    print!("{:02X} ", n);
                }
                println!();
            }
            //img.save("test.png").expect("failed to save");
        } else if name == "identity-image2" {
            println!("identity-image2");
        } else {
            println!("else");
        }
    }
    Ok((StatusCode::OK, Json(IdentityResult {})))
}

#[derive(Serialize, Debug)]
pub(crate) struct IdentityResult {}
