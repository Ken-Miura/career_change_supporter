// Copyright 2021 Ken Miura

use axum::{
    extract::{ContentLengthLimit, Multipart},
    http::StatusCode,
    Json,
};
use common::RespResult;
use serde::Serialize;

use crate::util::session::User;

pub(crate) async fn post_identity(
    User { account_id }: User,
    ContentLengthLimit(mut multipart): ContentLengthLimit<
        Multipart,
        {
            250 * 1024 * 1024 /* 250mb */
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
    }
    Ok((StatusCode::OK, Json(IdentityResult {})))
}

#[derive(Serialize, Debug)]
pub(crate) struct IdentityResult {}
