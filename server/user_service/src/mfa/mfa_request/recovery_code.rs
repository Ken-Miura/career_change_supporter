// Copyright 2023 Ken Miura

use std::time::Duration;

use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use axum_extra::extract::SignedCookieJar;
use chrono::{DateTime, FixedOffset, Utc};
use common::{ApiError, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::util::session::LOGIN_SESSION_EXPIRY;
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
trait RecoveryCodeOperation {}

struct RecoveryCodeOperationImpl {
    pool: DatabaseConnection,
    expiry: Duration,
}

#[async_trait]
impl RecoveryCodeOperation for RecoveryCodeOperationImpl {}

async fn handle_recovery_code(
    session_id: &str,
    current_date_time: &DateTime<FixedOffset>,
    recovery_code: &str,
    op: &impl RecoveryCodeOperation,
    store: &impl SessionStore,
) -> RespResult<RecoveryCodeReqResult> {
    // リカバリーコードのvalidation
    // セッションIDでセッションを取得
    // セッションからアカウントIDを取得
    // アカウントIDからUserInfo取得（取得の際にDisabledチェック)
    // 二段階認証が有効化されていることを確認
    // (LoginStatusのチェックはしない。既にFinishでも処理は続行させる。二段階認証を無効化する処理を含むので)
    // アカウントIDからMfaInfoを取得
    // リカバリーコードを比較
    // 二段階認証の設定を削除し、無効化する
    // セッション内のLoginStatusを更新
    // セッション内のexpiryを更新
    // ログイン日時を更新
    todo!()
}
