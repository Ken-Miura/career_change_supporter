// Copyright 2023 Ken Miura

use std::time::Duration;

use async_fred_session::RedisSessionStore;
use async_session::Session;
use axum::async_trait;
use axum::{extract::State, Json};
use axum_extra::extract::SignedCookieJar;
use chrono::{DateTime, FixedOffset};
use common::{ErrResp, RespResult};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::unexpected_err_resp;

use super::admin_operation::{AdminInfo, FindAdminInfoOperationImpl};

pub(crate) async fn post_pass_code(
    jar: SignedCookieJar,
    State(pool): State<DatabaseConnection>,
    State(store): State<RedisSessionStore>,
    Json(req): Json<PassCodeReq>,
) -> RespResult<PassCodeReqResult> {
    todo!()
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PassCodeReq {
    pass_code: String,
}

#[derive(Debug, Serialize, PartialEq)]
pub(crate) struct PassCodeReqResult {}

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
