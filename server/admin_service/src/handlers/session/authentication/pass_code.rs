// Copyright 2023 Ken Miura

use std::time::Duration;

use async_fred_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::SignedCookieJar;
use chrono::{DateTime, FixedOffset, Utc};
use common::util::validator::pass_code_validator::validate_pass_code;
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::ADMIN_SESSION_ID_COOKIE_NAME;

use super::admin_operation::{AdminInfo, FindAdminInfoOperationImpl};
use super::LOGIN_SESSION_EXPIRY;

pub(crate) async fn post_pass_code(
    jar: SignedCookieJar,
    State(pool): State<DatabaseConnection>,
    State(store): State<RedisSessionStore>,
    Json(req): Json<PassCodeReq>,
) -> RespResult<PassCodeReqResult> {
    let option_cookie = jar.get(ADMIN_SESSION_ID_COOKIE_NAME);
    let session_id = extract_session_id_from_cookie(option_cookie)?;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = PassCodeOperationImpl {
        pool,
        expiry: LOGIN_SESSION_EXPIRY,
    };

    todo!()
}

fn extract_session_id_from_cookie(cookie: Option<Cookie>) -> Result<String, ErrResp> {
    let session_id = match cookie {
        Some(s) => s.value().to_string(),
        None => {
            error!("no sessoin cookie found");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: Code::Unauthorized as u32,
                }),
            ));
        }
    };
    Ok(session_id)
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PassCodeReq {
    pass_code: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct PassCodeReqResult {}

async fn handle_pass_code_req(
    session_id: &str,
    current_date_time: &DateTime<FixedOffset>,
    pass_code: &str,
    issuer: &str,
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
    todo!()
}

#[async_trait]
trait PassCodeOperation {
    async fn get_admin_info_by_account_id(&self, account_id: i64) -> Result<AdminInfo, ErrResp>;

    async fn get_admin_mfa_info_by_account_id(&self, account_id: i64) -> Result<MfaInfo, ErrResp>;

    fn set_login_session_expiry(&self, session: &mut Session);

    async fn update_last_login(
        &self,
        account_id: i64,
        login_time: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

#[derive(Clone, Debug)]
struct MfaInfo {
    base32_encoded_secret: String,
}

struct PassCodeOperationImpl {
    pool: DatabaseConnection,
    expiry: Duration,
}

#[async_trait]
impl PassCodeOperation for PassCodeOperationImpl {
    async fn get_admin_info_by_account_id(&self, account_id: i64) -> Result<AdminInfo, ErrResp> {
        let op = FindAdminInfoOperationImpl::new(&self.pool);
        let admin_info =
            crate::handlers::session::authentication::admin_operation::get_admin_info_by_account_id(
                account_id, &op,
            )
            .await?;
        Ok(admin_info)
    }

    async fn get_admin_mfa_info_by_account_id(&self, account_id: i64) -> Result<MfaInfo, ErrResp> {
        let result = entity::admin_mfa_info::Entity::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find admin_mfa_info (admin_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        let mi = result.ok_or_else(|| {
            error!("no admin_mfa_info (admin_account_id: {}) found", account_id);
            unexpected_err_resp()
        })?;
        Ok(MfaInfo {
            base32_encoded_secret: mi.base32_encoded_secret,
        })
    }

    fn set_login_session_expiry(&self, session: &mut Session) {
        session.expire_in(self.expiry);
    }

    async fn update_last_login(
        &self,
        account_id: i64,
        login_time: &DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        super::update_last_login(account_id, login_time, &self.pool).await
    }
}
