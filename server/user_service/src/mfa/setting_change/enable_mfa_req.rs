// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::util::session::user::User;

pub(crate) async fn post_enable_mfa_req(
    User { user_info }: User,
    State(pool): State<DatabaseConnection>,
    Json(enable_mfa_req): Json<EnableMfaReq>,
) -> RespResult<EnableMfaReqResult> {
    let account_id = user_info.account_id;
    let mfa_enabled = user_info.mfa_enabled_at.is_some();
    let pass_code = enable_mfa_req.pass_code;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = EnableMfaReqOperationImpl { pool };
    handle_enable_mfa_req(account_id, mfa_enabled, pass_code, current_date_time, op).await
}

#[derive(Deserialize)]
pub(crate) struct EnableMfaReq {
    pub(crate) pass_code: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct EnableMfaReqResult {
    recovery_code: String,
}

async fn handle_enable_mfa_req(
    account_id: i64,
    mfa_enabled: bool,
    pass_code: String,
    current_date_time: DateTime<FixedOffset>,
    op: impl EnableMfaReqOperation,
) -> RespResult<EnableMfaReqResult> {
    todo!()
}

#[async_trait]
trait EnableMfaReqOperation {}

struct EnableMfaReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl EnableMfaReqOperation for EnableMfaReqOperationImpl {}
