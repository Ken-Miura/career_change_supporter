// Copyright 2023 Ken Miura

use std::time::Duration;

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

use crate::mfa::mfa_request::get_session_by_session_id;
use crate::mfa::USER_TOTP_ISSUER;
use crate::{
    err::{unexpected_err_resp, Code},
    mfa::{ensure_mfa_is_enabled, verify_pass_code},
    util::{
        login_status::LoginStatus,
        session::{KEY_TO_LOGIN_STATUS, LOGIN_SESSION_EXPIRY, SESSION_ID_COOKIE_NAME},
        user_info::{FindUserInfoOperationImpl, UserInfo},
    },
};

use super::{
    get_account_id_from_session, get_mfa_info_by_account_id, update_login_status, MfaInfo,
};

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
    let op = PassCodeOperationImpl {
        pool,
        expiry: LOGIN_SESSION_EXPIRY,
    };

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

    async fn get_mfa_info_by_account_id(&self, account_id: i64) -> Result<MfaInfo, ErrResp>;

    fn set_login_session_expiry(&self, session: &mut Session);

    async fn update_last_login(
        &self,
        account_id: i64,
        login_time: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct PassCodeOperationImpl {
    pool: DatabaseConnection,
    expiry: Duration,
}

#[async_trait]
impl PassCodeOperation for PassCodeOperationImpl {
    async fn get_user_info_if_available(&self, account_id: i64) -> Result<UserInfo, ErrResp> {
        let op = FindUserInfoOperationImpl::new(&self.pool);
        let user_info = crate::util::get_user_info_if_available(account_id, &op).await?;
        Ok(user_info)
    }

    async fn get_mfa_info_by_account_id(&self, account_id: i64) -> Result<MfaInfo, ErrResp> {
        get_mfa_info_by_account_id(account_id, &self.pool).await
    }

    fn set_login_session_expiry(&self, session: &mut Session) {
        session.expire_in(self.expiry);
    }

    async fn update_last_login(
        &self,
        account_id: i64,
        login_time: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        crate::util::update_last_login(account_id, login_time, &self.pool).await
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
    let mut session = get_session_by_session_id(session_id, store).await?;
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

    let mi = op.get_mfa_info_by_account_id(account_id).await?;
    verify_pass_code(
        account_id,
        &mi.base32_encoded_secret,
        USER_TOTP_ISSUER.as_str(),
        current_date_time,
        pass_code,
    )?;

    update_login_status(&mut session, LoginStatus::Finish)?;
    op.set_login_session_expiry(&mut session);
    let _ = store.store_session(session).await.map_err(|e| {
        error!("failed to store session: {}", e);
        unexpected_err_resp()
    })?;

    op.update_last_login(account_id, current_date_time).await?;

    Ok((StatusCode::OK, Json(PassCodeReqResult {})))
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
