// Copyright 2023 Ken Miura

use std::time::Duration;

use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use axum_extra::extract::SignedCookieJar;
use chrono::{DateTime, FixedOffset, Utc};
use common::mfa::is_recovery_code_match;
use common::util::validator::uuid_validator::validate_uuid;
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::unexpected_err_resp;
use crate::mfa::ensure_mfa_is_enabled;
use crate::mfa::mfa_request::get_session_by_session_id;
use crate::util::session::LOGIN_SESSION_EXPIRY;
use crate::util::user_info::{FindUserInfoOperationImpl, UserInfo};
use crate::{err::Code, util::session::SESSION_ID_COOKIE_NAME};

use super::{get_account_id_from_session, get_mfa_info_by_account_id, MfaInfo};

pub(crate) async fn post_recovery_code(
    jar: SignedCookieJar,
    State(pool): State<DatabaseConnection>,
    State(store): State<RedisSessionStore>,
    Json(req): Json<RecoveryCodeReq>,
) -> RespResult<RecoveryCodeReqResult> {
    let option_cookie = jar.get(SESSION_ID_COOKIE_NAME);
    let session_id = match option_cookie {
        Some(s) => s.value().to_string(),
        None => {
            error!("no sessoin cookie found on recovery code req");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: Code::Unauthorized as u32,
                }),
            ));
        }
    };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = RecoveryCodeOperationImpl {
        pool,
        expiry: LOGIN_SESSION_EXPIRY,
    };

    handle_recovery_code(
        session_id.as_str(),
        &current_date_time,
        req.recovery_code.as_str(),
        &op,
        &store,
    )
    .await
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RecoveryCodeReq {
    recovery_code: String,
}

#[derive(Serialize)]
pub(crate) struct RecoveryCodeReqResult {}

#[async_trait]
trait RecoveryCodeOperation {
    async fn get_user_info_if_available(&self, account_id: i64) -> Result<UserInfo, ErrResp>;

    async fn get_mfa_info_by_account_id(&self, account_id: i64) -> Result<MfaInfo, ErrResp>;
}

struct RecoveryCodeOperationImpl {
    pool: DatabaseConnection,
    expiry: Duration,
}

#[async_trait]
impl RecoveryCodeOperation for RecoveryCodeOperationImpl {
    async fn get_user_info_if_available(&self, account_id: i64) -> Result<UserInfo, ErrResp> {
        let op = FindUserInfoOperationImpl::new(&self.pool);
        let user_info = crate::util::get_user_info_if_available(account_id, &op).await?;
        Ok(user_info)
    }

    async fn get_mfa_info_by_account_id(&self, account_id: i64) -> Result<MfaInfo, ErrResp> {
        get_mfa_info_by_account_id(account_id, &self.pool).await
    }
}

async fn handle_recovery_code(
    session_id: &str,
    current_date_time: &DateTime<FixedOffset>,
    recovery_code: &str,
    op: &impl RecoveryCodeOperation,
    store: &impl SessionStore,
) -> RespResult<RecoveryCodeReqResult> {
    validate_uuid(recovery_code).map_err(|e| {
        error!("failed to validate {}: {}", recovery_code, e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: common::err::Code::InvalidUuidFormat as u32,
            }),
        )
    })?;
    let mut session = get_session_by_session_id(session_id, store).await?;
    let account_id = get_account_id_from_session(&session)?;
    let user_info = op.get_user_info_if_available(account_id).await?;
    ensure_mfa_is_enabled(user_info.mfa_enabled_at.is_some())?;

    // この関数は二段階認証を無効化する処理を含む
    // 従って（post_pass_codeはFinishの場合に早期リターンしてログイン成功扱いする一方で）LoginStatusの値によらずに処理は続行する

    let mi = op.get_mfa_info_by_account_id(account_id).await?;
    verify_recovery_code(recovery_code, &mi.hashed_recovery_code)?;

    // 二段階認証の設定を削除し、無効化する
    // セッション内のLoginStatusを更新
    // セッション内のexpiryを更新
    // ログイン日時を更新
    todo!()
}

fn verify_recovery_code(recovery_code: &str, hashed_recovery_code: &[u8]) -> Result<(), ErrResp> {
    let matched = is_recovery_code_match(recovery_code, hashed_recovery_code).map_err(|e| {
        error!("failed is_recovery_code_match: {}", e);
        unexpected_err_resp()
    })?;
    if !matched {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::RecoveryCodeDoesNotMatch as u32,
            }),
        ));
    }
    Ok(())
}
