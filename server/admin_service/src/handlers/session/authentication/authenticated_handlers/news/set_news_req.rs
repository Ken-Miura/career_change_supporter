// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::util::validator::{has_control_char, has_non_new_line_control_char};
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::Code;
use crate::handlers::session::authentication::authenticated_handlers::admin::Admin;

const MIN_TITLE_SIZE: usize = 1;
const MAX_TITLE_SIZE: usize = 256;

const MIN_BODY_SIZE: usize = 1;
const MAX_BODY_SIZE: usize = 16384;

pub(crate) async fn post_set_news_req(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<SetNewsReq>,
) -> RespResult<SetNewsReqResult> {
    let op = SetNewsReqOperationImpl { pool };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    handle_set_news_req(req.title, req.body, current_date_time, &op).await
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct SetNewsReq {
    title: String,
    body: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct SetNewsReqResult {}

#[async_trait]
trait SetNewsReqOperation {}

struct SetNewsReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl SetNewsReqOperation for SetNewsReqOperationImpl {}

async fn handle_set_news_req(
    title: String,
    body: String,
    current_date_time: DateTime<FixedOffset>,
    op: &impl SetNewsReqOperation,
) -> RespResult<SetNewsReqResult> {
    validate_title(&title)?;
    validate_body(&body)?;
    todo!()
}

fn validate_title(title: &str) -> Result<(), ErrResp> {
    let length = title.len();
    if !(MIN_TITLE_SIZE..=MAX_TITLE_SIZE).contains(&length) {
        error!("invalid title length ({})", length);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidTitleLength as u32,
            }),
        ));
    }
    if has_control_char(title) {
        error!("title has control char ({:?})", title.as_bytes());
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::IllegalTitle as u32,
            }),
        ));
    }
    Ok(())
}

fn validate_body(body: &str) -> Result<(), ErrResp> {
    let length = body.len();
    if !(MIN_BODY_SIZE..=MAX_BODY_SIZE).contains(&length) {
        error!("invalid body length ({})", length);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidBodyLength as u32,
            }),
        ));
    }
    if has_non_new_line_control_char(body) {
        error!("body has non new line control char ({:?})", body.as_bytes());
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::IllegalBody as u32,
            }),
        ));
    }
    Ok(())
}
