// Copyright 2023 Ken Miura

use async_session::log::error;
use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::err::unexpected_err_resp;
use crate::mfa::{create_totp, ensure_mfa_is_not_enabled, filter_temp_mfa_secret_order_by_dsc};
use crate::mfa::{get_latest_temp_mfa_secret, TempMfaSecret};
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
    ensure_mfa_is_not_enabled(mfa_enabled)?;

    let temp_mfa_secrets = op
        .filter_temp_mfa_secret_order_by_dsc(account_id, current_date_time)
        .await?;
    let temp_mfa_secret = get_latest_temp_mfa_secret(temp_mfa_secrets)?;

    let totp = create_totp(account_id, temp_mfa_secret.base32_encoded_secret.clone())?;
    let ts = create_timestamp(&current_date_time)?;
    let is_valid = totp.check(pass_code.as_str(), ts);
    // 設定を有効化する
    //   トランザクション内で以下を実施
    //   UserAccountの値の変更
    //   temp_mfa_secretの削除 -> どうせユーザーには期限が過ぎたら見えなくなる（設定が有効化されても見えなくなる）、かつ定期削除が入るので削除処理はいらない？
    //   mfa_infoの挿入
    Ok((StatusCode::OK, Json(EnableMfaReqResult { recovery_code })))
}

#[async_trait]
trait EnableMfaReqOperation {
    async fn filter_temp_mfa_secret_order_by_dsc(
        &self,
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<TempMfaSecret>, ErrResp>;
}

struct EnableMfaReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl EnableMfaReqOperation for EnableMfaReqOperationImpl {
    async fn filter_temp_mfa_secret_order_by_dsc(
        &self,
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<TempMfaSecret>, ErrResp> {
        filter_temp_mfa_secret_order_by_dsc(account_id, current_date_time, &self.pool).await
    }
}

fn create_timestamp(current_date_time: &DateTime<FixedOffset>) -> Result<u64, ErrResp> {
    // chronoのタイムスタンプはi64のため、他のタイムスタンプでよく使われるu64に変換する必要がある
    // https://github.com/chronotope/chrono/issues/326
    // 上記によると、chronoのタイムスタンプがi64であるのはUTC 1970年1月1日午前0時より前の時間を表すため。
    // 従って、現代に生きる我々にとってi64の値が負の値になることはなく、u64へのキャストが失敗することはない。
    let chrono_ts = current_date_time.timestamp();
    let ts = u64::try_from(current_date_time.timestamp()).map_err(|e| {
        error!("failed to convert {} to type u64: {}", chrono_ts, e);
        unexpected_err_resp()
    })?;
    Ok(ts)
}
