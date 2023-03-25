// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
    let uuid = Uuid::new_v4().simple().to_string();
    let op = EnableMfaReqOperationImpl { pool };
    handle_enable_mfa_req(
        account_id,
        mfa_enabled,
        pass_code,
        current_date_time,
        uuid,
        op,
    )
    .await
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
    recovery_code: String,
    op: impl EnableMfaReqOperation,
) -> RespResult<EnableMfaReqResult> {
    // MFAが有効化されていないことを確認する
    // temp_mfa_secretを取得する
    // 取得したtemp_mfa_secretからTOTPインスタンスを生成
    // TOTPインスタンスを用いてpass_codeをチェック
    // 設定を有効化する
    //   トランザクション内で以下を実施
    //   UserAccountの値の変更
    //   temp_mfa_secretの削除
    //   mfa_infoの挿入
    Ok((StatusCode::OK, Json(EnableMfaReqResult { recovery_code })))
}

#[async_trait]
trait EnableMfaReqOperation {}

struct EnableMfaReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl EnableMfaReqOperation for EnableMfaReqOperationImpl {}
