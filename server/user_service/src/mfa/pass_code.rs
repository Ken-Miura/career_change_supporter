// Copyright 2023 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::{log::info, Session, SessionStore};
use axum::{async_trait, extract::State, http::StatusCode, Json};
use axum_extra::extract::SignedCookieJar;
use chrono::{DateTime, FixedOffset, Utc};
use common::{
    util::validator::pass_code_validator::validate_pass_code, ApiError, ErrResp, RespResult,
    JAPANESE_TIME_ZONE,
};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    err::{unexpected_err_resp, Code},
    util::{
        login_status::LoginStatus,
        session::{KEY_TO_LOGIN_STATUS, KEY_TO_USER_ACCOUNT_ID, SESSION_ID_COOKIE_NAME},
        user_info::{FindUserInfoOperationImpl, UserInfo},
    },
};

use super::ensure_mfa_is_enabled;

pub(crate) async fn post_pass_code(
    jar: SignedCookieJar,
    State(pool): State<DatabaseConnection>,
    State(store): State<RedisSessionStore>,
    Json(req): Json<PassCodeReq>,
) -> RespResult<PassCodeReqResult> {
    let option_cookie = jar.get(SESSION_ID_COOKIE_NAME);
    let session_id = match option_cookie {
        Some(s) => s.value().to_string(),
        None => {
            error!("no sessoin cookie found on pass code req");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: Code::Unauthorized as u32,
                }),
            ));
        }
    };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = PassCodeOperationImpl { pool };

    handle_pass_code_req(
        session_id.as_str(),
        &current_date_time,
        req.pass_code.as_str(),
        &op,
        &store,
    )
    .await
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PassCodeReq {
    pass_code: String,
}

#[derive(Serialize)]
pub(crate) struct PassCodeReqResult {}

#[async_trait]
trait PassCodeOperation {
    async fn get_user_info_if_available(&self, account_id: i64) -> Result<UserInfo, ErrResp>;
}

struct PassCodeOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl PassCodeOperation for PassCodeOperationImpl {
    async fn get_user_info_if_available(&self, account_id: i64) -> Result<UserInfo, ErrResp> {
        let op = FindUserInfoOperationImpl::new(&self.pool);
        let user_info = crate::util::get_user_info_if_available(account_id, &op).await?;
        Ok(user_info)
    }
}

async fn handle_pass_code_req(
    session_id: &str,
    current_date_time: &DateTime<FixedOffset>,
    pass_code: &str,
    op: &impl PassCodeOperation,
    store: &impl SessionStore,
) -> RespResult<PassCodeReqResult> {
    validate_pass_code(pass_code).map_err(|e| {
        error!("invalid pass code format: {}", e);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidPassCode as u32,
            }),
        )
    })?;
    let session = get_session_by_session_id(session_id, store).await?;
    let account_id = get_account_id_from_session(&session)?;
    let user_info = op.get_user_info_if_available(account_id).await?;
    ensure_mfa_is_enabled(user_info.mfa_enabled_at.is_some())?;
    let ls = get_login_status_from_session(&session)?;
    if ls == LoginStatus::Finish {
        info!(
            "LoginStatus has already been Finish (account_id: {})",
            account_id
        );
        return Ok((StatusCode::OK, Json(PassCodeReqResult {})));
    };
    // シークレット、現在時刻に対してパスコードが一致するか確認
    // セッションのLoginStatusを更新（セッションの期限も更新する）
    // 最終ログイン時刻を更新
    todo!()
}

async fn get_session_by_session_id(
    session_id: &str,
    store: &impl SessionStore,
) -> Result<Session, ErrResp> {
    let option_session = store
        .load_session(session_id.to_string())
        .await
        .map_err(|e| {
            error!("failed to load session: {}", e);
            unexpected_err_resp()
        })?;
    let session = match option_session {
        Some(s) => s,
        None => {
            error!("no session found on pass code req");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: Code::Unauthorized as u32,
                }),
            ));
        }
    };
    Ok(session)
}

fn get_account_id_from_session(session: &Session) -> Result<i64, ErrResp> {
    let account_id = match session.get::<i64>(KEY_TO_USER_ACCOUNT_ID) {
        Some(id) => id,
        None => {
            error!("failed to get account id from session");
            return Err(unexpected_err_resp());
        }
    };
    Ok(account_id)
}

fn get_login_status_from_session(session: &Session) -> Result<LoginStatus, ErrResp> {
    let login_status = match session.get::<String>(KEY_TO_LOGIN_STATUS) {
        Some(ls) => ls,
        None => {
            error!("failed to get login status from session");
            return Err(unexpected_err_resp());
        }
    };
    Ok(LoginStatus::from(login_status))
}
