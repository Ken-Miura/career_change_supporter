// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::{http::StatusCode, Json};
use axum_extra::extract::SignedCookieJar;
use chrono::{DateTime, FixedOffset};
use common::{ApiError, AppState, ErrResp};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::Deserialize;
use tracing::{error, info};

use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::authentication::{
    get_admin_account_id_by_session_id, RefreshOperationImpl, LOGIN_SESSION_EXPIRY,
};
use crate::handlers::session::ADMIN_SESSION_ID_COOKIE_NAME;

/// 管理者の情報の情報を保持する構造体
///
/// ハンドラ関数内で管理者の情報にアクセスしたい場合、原則としてこの型をパラメータとして受け付ける。
/// この型をパラメータとして受け付けると、ハンドラ関数の処理に入る前に下記の前処理を実施する。
/// <ul>
///   <li>ログインセッションが有効であることを確認</li>
/// </ul>
#[derive(Deserialize, Clone, Debug)]
pub(crate) struct Admin {
    pub(super) admin_info: AdminInfo,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(super) struct AdminInfo {
    pub(super) account_id: i64,
    pub(super) email_address: String,
    pub(super) mfa_enabled_at: Option<DateTime<FixedOffset>>,
}

#[async_trait]
impl<S> FromRequestParts<S> for Admin
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ErrResp;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let signed_cookies = SignedCookieJar::<AppState>::from_request_parts(parts, state)
            .await
            .map_err(|e| {
                error!("failed to get cookies: {:?}", e);
                unexpected_err_resp()
            })?;
        let option_cookie = signed_cookies.get(ADMIN_SESSION_ID_COOKIE_NAME);
        let session_id = match option_cookie {
            Some(s) => s.value().to_string(),
            None => {
                info!("no sessoin cookie found");
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ApiError {
                        code: Code::Unauthorized as u32,
                    }),
                ));
            }
        };
        let app_state = AppState::from_ref(state);
        let store = app_state.store;
        let reresh_op = RefreshOperationImpl {};
        let admin_account_id =
            get_admin_account_id_by_session_id(session_id, &store, reresh_op, LOGIN_SESSION_EXPIRY)
                .await?;

        let pool = app_state.pool;
        let find_admin_info_op = FindAdminInfoOperationImpl::new(&pool);
        let admin_info =
            get_admin_info_by_account_id(admin_account_id, &find_admin_info_op).await?;

        Ok(Admin { admin_info })
    }
}

async fn get_admin_info_by_account_id(
    account_id: i64,
    op: &impl FindAdminInfoOperation,
) -> Result<AdminInfo, ErrResp> {
    let admin_info = op.find_admin_info_by_account_id(account_id).await?;
    let admin_info = admin_info.ok_or_else(|| {
        error!("no account ({}) found", account_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoAccountFound as u32,
            }),
        )
    })?;
    Ok(admin_info)
}

#[async_trait]
trait FindAdminInfoOperation {
    async fn find_admin_info_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<AdminInfo>, ErrResp>;
}

struct FindAdminInfoOperationImpl<'a> {
    pool: &'a DatabaseConnection,
}

impl<'a> FindAdminInfoOperationImpl<'a> {
    pub(super) fn new(pool: &'a DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> FindAdminInfoOperation for FindAdminInfoOperationImpl<'a> {
    async fn find_admin_info_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<AdminInfo>, ErrResp> {
        let model = entity::admin_account::Entity::find_by_id(account_id)
            .one(self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find admin_account (admin_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| AdminInfo {
            account_id: m.admin_account_id,
            email_address: m.email_address,
            mfa_enabled_at: m.mfa_enabled_at,
        }))
    }
}
