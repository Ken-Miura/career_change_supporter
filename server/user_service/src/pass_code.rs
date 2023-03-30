// Copyright 2023 Ken Miura

use async_redis_session::RedisSessionStore;
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::SignedCookieJar;
use common::{ApiError, RespResult};
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
    // セッションを取得する。なければUnauthirized
    // セッション内のアカウントIDからUserInfoを取得
    // Disabledチェック
    // 二段階認証の有効化チェック（シークレットが存在することを前提とした処理をするために事前チェックは必要）
    // セッション内のLoginStatusのチェック（Finishなら何もせずに早期リターン。いらないかも）
    // シークレット、現在時刻に対してパスコードが一致するか確認
    // セッションのLoginStatusを更新（セッションの期限も更新する）
    // 最終ログイン時刻を更新
    // NOTE: Sessionは更新するが、Cookieを更新するわけではない。従ってSignedCookieJarをハンドラのレスポンスに含める必要はないように見える。要確認
    todo!()
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct PassCodeReq {
    pass_code: String,
}

#[derive(Serialize)]
pub(crate) struct PassCodeReqResult {}
