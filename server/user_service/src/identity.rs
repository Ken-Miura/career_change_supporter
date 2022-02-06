// Copyright 2021 Ken Miura

use std::io::Cursor;

use async_session::serde_json;
use axum::{
    extract::{ContentLengthLimit, Multipart},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use common::{ApiError, ErrResp, RespResult};
use image::ImageFormat;
use serde::Serialize;

use crate::{
    err,
    util::{
        session::User,
        unexpected_err_resp,
        validator::identity_validator::{validate_identity, IdentityValidationError},
        Identity, JAPANESE_TIME_ZONE,
    },
};

pub(crate) async fn post_identity(
    User { account_id }: User,
    ContentLengthLimit(mut multipart): ContentLengthLimit<
        Multipart,
        {
            9 * 1024 * 1024 /* 9mb */
        },
    >,
) -> RespResult<IdentityResult> {
    let mut identity_option = None;
    // let mut identity_image1_option = None;
    // let mut identity_image2_option = None;
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        tracing::error!("failed to get next_field: {}", e);
        unexpected_err_resp()
    })? {
        let name = match field.name() {
            Some(n) => n.to_string(),
            None => todo!(),
        };
        let file_name_option = field.file_name();
        let data = field.bytes().await.map_err(|e| {
            tracing::error!("failed to get data in field: {}", e);
            // BAD REQにする
            unexpected_err_resp()
        })?;
        println!("Length of `{}` is {} bytes", name, data.len());
        if name == "identity" {
            let identity_str = std::str::from_utf8(&data)
                .unwrap()
                .parse::<String>()
                .unwrap();
            let identity = serde_json::from_str::<Identity>(&identity_str).unwrap();

            let current_date = Utc::now()
                .with_timezone(&JAPANESE_TIME_ZONE.to_owned())
                .naive_local()
                .date();
            let _ = validate_identity(&identity, &current_date).map_err(|e| {
                tracing::error!("invalid identity: {}", e);
                create_invalid_identity_err(&e)
            })?;
            identity_option = Some(trim_space_from_identity(identity));
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
    println!("{:?}", identity_option);
    Ok((StatusCode::OK, Json(IdentityResult {})))
}

#[derive(Serialize, Debug)]
pub(crate) struct IdentityResult {}

fn create_invalid_identity_err(e: &IdentityValidationError) -> ErrResp {
    let code;
    match e {
        IdentityValidationError::InvalidLastNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = err::INVALID_LAST_NAME_LENGTH,
        IdentityValidationError::IllegalCharInLastName(_) => code = err::ILLEGAL_CHAR_IN_LAST_NAME,
        IdentityValidationError::InvalidFirstNameLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = err::INVALID_FIRST_NAME_LENGTH,
        IdentityValidationError::IllegalCharInFirstName(_) => {
            code = err::ILLEGAL_CHAR_IN_FIRST_NAME
        }
        IdentityValidationError::InvalidLastNameFuriganaLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = err::INVALID_LAST_NAME_FURIGANA_LENGTH,
        IdentityValidationError::IllegalCharInLastNameFurigana(_) => {
            code = err::ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA
        }
        IdentityValidationError::InvalidFirstNameFuriganaLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = err::INVALID_FIRST_NAME_FURIGANA_LENGTH,
        IdentityValidationError::IllegalCharInFirstNameFurigana(_) => {
            code = err::ILLEGAL_CHAR_IN_FIRST_NAME_FURIGANA
        }
        IdentityValidationError::IllegalDate {
            year: _,
            month: _,
            day: _,
        } => code = err::ILLEGAL_DATE,
        IdentityValidationError::IllegalAge {
            birth_year: _,
            birth_month: _,
            birth_day: _,
            current_year: _,
            current_month: _,
            current_day: _,
        } => code = err::ILLEGAL_AGE,
        IdentityValidationError::InvalidPrefecture(_) => code = err::INVALID_PREFECTURE,
        IdentityValidationError::InvalidCityLength {
            length: _,
            min_length: _,
            max_length: _,
        } => code = err::INVALID_CITY_LENGTH,
        IdentityValidationError::IllegalCharInCity(_) => code = err::ILLEGAL_CHAR_IN_CITY,
        IdentityValidationError::InvalidAddressLine1Length {
            length: _,
            min_length: _,
            max_length: _,
        } => code = err::INVALID_ADDRESS_LINE1_LENGTH,
        IdentityValidationError::IllegalCharInAddressLine1(_) => {
            code = err::ILLEGAL_CHAR_IN_ADDRESS_LINE1
        }
        IdentityValidationError::InvalidAddressLine2Length {
            length: _,
            min_length: _,
            max_length: _,
        } => code = err::INVALID_ADDRESS_LINE2_LENGTH,
        IdentityValidationError::IllegalCharInAddressLine2(_) => {
            code = err::ILLEGAL_CHAR_IN_ADDRESS_LINE2
        }
        IdentityValidationError::InvalidTelNumFormat(_) => code = err::INVALID_TEL_NUM_FORMAT,
    }
    (StatusCode::BAD_REQUEST, Json(ApiError { code }))
}

fn trim_space_from_identity(identity: Identity) -> Identity {
    Identity {
        last_name: identity.last_name.trim().to_string(),
        first_name: identity.first_name.trim().to_string(),
        last_name_furigana: identity.last_name_furigana.trim().to_string(),
        first_name_furigana: identity.first_name_furigana.trim().to_string(),
        date_of_birth: identity.date_of_birth,
        prefecture: identity.prefecture.trim().to_string(),
        city: identity.city.trim().to_string(),
        address_line1: identity.address_line1.trim().to_string(),
        address_line2: identity
            .address_line2
            .map(|address_line2| address_line2.trim().to_string()),
        telephone_number: identity.telephone_number.trim().to_string(),
    }
}
