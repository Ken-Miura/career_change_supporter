// Copyright 2023 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::SessionStore;
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::SignedCookieJar;
use chrono::{DateTime, FixedOffset, Utc};
use common::{ApiError, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{err::Code, util::session::SESSION_ID_COOKIE_NAME};

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

trait PassCodeOperation {}

struct PassCodeOperationImpl {
    pool: DatabaseConnection,
}

impl PassCodeOperation for PassCodeOperationImpl {}

async fn handle_pass_code_req(
    session_id: &str,
    current_date_time: &DateTime<FixedOffset>,
    pass_code: &str,
    op: &impl PassCodeOperation,
    store: &impl SessionStore,
) -> RespResult<PassCodeReqResult> {
    // pass_codeのvalidation
    // セッションを取得する。なければUnauthirized
    // セッション内のアカウントIDからUserInfoを取得
    // Disabledチェック
    // 二段階認証の有効化チェック（シークレットが存在することを前提とした処理をするために事前チェックは必要）
    // セッション内のLoginStatusのチェック（Finishなら何もせずに早期リターン。いらないかも）
    // シークレット、現在時刻に対してパスコードが一致するか確認
    // セッションのLoginStatusを更新（セッションの期限も更新する）
    // 最終ログイン時刻を更新
    todo!()
}
