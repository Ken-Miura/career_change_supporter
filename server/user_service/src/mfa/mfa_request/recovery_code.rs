// Copyright 2023 Ken Miura

use async_redis_session::RedisSessionStore;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use axum_extra::extract::SignedCookieJar;
use common::{ApiError, RespResult};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{err::Code, util::session::SESSION_ID_COOKIE_NAME};

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
    todo!()
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct RecoveryCodeReq {
    recovery_code: String,
}

#[derive(Serialize)]
pub(crate) struct RecoveryCodeReqResult {}
