// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::{async_trait, SessionStore};
use axum::{
    extract::{Extension, FromRequest, RequestParts},
    Json,
};
use common::{ApiError, ErrResp};
use headers::Cookie;
use headers::HeaderMapExt;
use hyper::StatusCode;
use serde::Deserialize;
use std::time::Duration;

use crate::{
    err_code::UNAUTHORIZED,
    util::{unexpected_err_resp, ROOT_PATH},
};

const COOKIE_NAME: &str = "session_id";
pub(crate) const KEY_TO_USER_ACCOUNT_ID: &str = "user_account_id";
const LENGTH_OF_MEETING: u64 = 60;
const TIME_FOR_SUBSEQUENT_OPERATIONS: u64 = 10;

/// セッションの有効期限
pub(crate) const LOGIN_SESSION_EXPIRY: Duration =
    Duration::from_secs(60 * (LENGTH_OF_MEETING + TIME_FOR_SUBSEQUENT_OPERATIONS));

/// [COOKIE_NAME]を含むSet-Cookie用の文字列を返す。
pub(crate) fn create_cookie_format(session_id_value: &str) -> String {
    format!(
        // TODO: SSLのセットアップが完了し次第、Secureを追加する
        //"{}={}; SameSite=Strict; Path={}/; Secure; HttpOnly",
        "{}={}; SameSite=Strict; Path={}/; HttpOnly",
        COOKIE_NAME,
        session_id_value,
        ROOT_PATH
    )
}

/// [COOKIE_NAME]を含む、有効期限切れのSet-Cookie用の文字列を返す<br>
/// ブラウザに保存されたCookieの削除指示を出したいときに使う。
pub(crate) fn create_expired_cookie_format(session_id_value: &str) -> String {
    format!(
        // TODO: SSLのセットアップが完了し次第、Secureを追加する
        //"{}={}; SameSite=Strict; Path={}/; Max-Age=-1; Secure; HttpOnly",
        "{}={}; SameSite=Strict; Path={}/; Max-Age=-1; HttpOnly",
        COOKIE_NAME,
        session_id_value,
        ROOT_PATH
    )
}

/// Cookieが存在し、[COOKIE_NAME]を含む場合、対応する値を返す
pub(crate) fn extract_session_id(option_cookie: Option<Cookie>) -> Option<String> {
    let cookie = match option_cookie {
        Some(c) => c,
        None => {
            tracing::debug!("no cookie");
            return None;
        }
    };
    match cookie.get(COOKIE_NAME) {
        Some(value) => Some(value.to_string()),
        None => {
            tracing::debug!("no {} in cookie", COOKIE_NAME);
            None
        }
    }
}

/// ユーザーの情報にアクセスするためのID
///
/// ハンドラ関数内でユーザーの情報にアクセスしたい場合、原則としてこの型をパラメータとして受け付ける。
/// このパラメータに含むIDを用いて、データベースからユーザー情報を取得できる。
/// この型をパラメータとして受け付けると、ハンドラ関数の処理に入る前に下記の前処理を実施する。
/// <ul>
///   <li>ログインセッションが有効であることを確認</li>
///   <li>TODO: 利用規約に同意済みである確認</li>
/// </ul>
#[derive(Deserialize, Clone)]
pub(crate) struct User {
    pub(crate) account_id: i32,
}

#[async_trait]
impl<B> FromRequest<B> for User
where
    B: Send,
{
    type Rejection = ErrResp;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let headers = match req.headers() {
            Some(h) => h,
            None => {
                tracing::debug!("no headers found");
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ApiError { code: UNAUTHORIZED }),
                ));
            }
        };
        let option_cookie = headers.typed_try_get::<Cookie>().map_err(|e| {
            tracing::error!("failed to get Cookie: {}", e);
            unexpected_err_resp()
        })?;
        let session_id_value = match extract_session_id(option_cookie) {
            Some(s) => s,
            None => {
                tracing::debug!("no valid cookie on request");
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ApiError { code: UNAUTHORIZED }),
                ));
            }
        };
        let Extension(store) = Extension::<RedisSessionStore>::from_request(req)
            .await
            .map_err(|e| {
                tracing::error!("failed to get session store: {}", e);
                unexpected_err_resp()
            })?;
        let option_session = store
            .load_session(session_id_value.clone())
            .await
            .map_err(|e| {
                tracing::error!(
                    "failed to load session (session_id={}): {}",
                    session_id_value,
                    e
                );
                unexpected_err_resp()
            })?;
        let session = match option_session {
            Some(s) => s,
            None => {
                tracing::debug!("no valid session on request");
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ApiError { code: UNAUTHORIZED }),
                ));
            }
        };
        let id = match session.get::<i32>(KEY_TO_USER_ACCOUNT_ID) {
            Some(id) => id,
            None => {
                tracing::error!(
                    "failed to get id from session (session_id={})",
                    session_id_value
                );
                return Err(unexpected_err_resp());
            }
        };
        Ok(User { account_id: id })
    }
}

/// テストコードで共通で使うコードをまとめるモジュール
#[cfg(test)]
pub(crate) mod tests {
    use headers::HeaderValue;

    use super::COOKIE_NAME;

    pub(crate) fn extract_session_id_value(header_value: &HeaderValue) -> String {
        let set_cookie = header_value.to_str().expect("failed to get value");
        let cookie_name = set_cookie
            .split(";")
            .find(|s| s.contains(COOKIE_NAME))
            .expect("failed to get session")
            .trim()
            .split_once("=")
            .expect("failed to get value");
        cookie_name.1.to_string()
    }

    pub(crate) fn extract_cookie_max_age_value(header_value: &HeaderValue) -> String {
        let set_cookie = header_value.to_str().expect("failed to get value");
        let cookie_max_age = set_cookie
            .split(";")
            .find(|s| s.contains("Max-Age"))
            .expect("failed to get Max-Age")
            .trim()
            .split_once("=")
            .expect("failed to get value");
        cookie_max_age.1.to_string()
    }
}
