// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::{http::StatusCode, Json};
use axum_extra::extract::SignedCookieJar;
use common::{ApiError, AppState, ErrResp};
use serde::Deserialize;
use tracing::{error, info};

use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::{
    get_admin_account_id_by_session_id, RefreshOperationImpl, ADMIN_SESSION_ID_COOKIE_NAME,
    LOGIN_SESSION_EXPIRY,
};

/// 管理者の情報の情報を保持する構造体
///
/// ハンドラ関数内で管理者の情報にアクセスしたい場合、原則としてこの型をパラメータとして受け付ける。
/// この型をパラメータとして受け付けると、ハンドラ関数の処理に入る前に下記の前処理を実施する。
/// <ul>
///   <li>ログインセッションが有効であることを確認</li>
/// </ul>
#[derive(Deserialize, Clone, Debug)]
pub(crate) struct Admin {
    pub(super) account_id: i64,
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
        let op = RefreshOperationImpl {};
        let admin_account_id =
            get_admin_account_id_by_session_id(session_id, &store, op, LOGIN_SESSION_EXPIRY)
                .await?;

        Ok(Admin {
            account_id: admin_account_id,
        })
    }
}
