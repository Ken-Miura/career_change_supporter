// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::{async_trait, SessionStore};
use axum::{
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
    Json,
};
use common::{ApiError, ConnectionPool, ErrResp};
use headers::Cookie;
use headers::HeaderMapExt;
use serde::Deserialize;
use std::time::Duration;

use crate::{
    err_code::{NOT_TERMS_OF_USE_AGREED_YET, UNAUTHORIZED},
    util::{unexpected_err_resp, ROOT_PATH},
};

use super::terms_of_use::{
    TermsOfUseLoadOperation, TermsOfUseLoadOperationImpl, TERMS_OF_USE_VERSION,
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
///   <li>利用規約に同意済みである確認</li>
/// </ul>
#[derive(Deserialize, Clone, Debug)]
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
        let Extension(store) = Extension::<RedisSessionStore>::from_request(req)
            .await
            .map_err(|e| {
                tracing::error!("failed to get session store: {}", e);
                unexpected_err_resp()
            })?;
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
        let user = get_user_by_cookie(option_cookie, &store).await?;

        let Extension(pool) = Extension::<ConnectionPool>::from_request(req)
            .await
            .map_err(|e| {
                tracing::error!("failed to extract connection pool from req: {}", e);
                unexpected_err_resp()
            })?;
        let conn = pool.get().map_err(|e| {
            tracing::error!("failed to get connection from pool: {}", e);
            unexpected_err_resp()
        })?;
        let op = TermsOfUseLoadOperationImpl::new(conn);
        let _ = check_if_user_has_already_agreed(user.account_id, *TERMS_OF_USE_VERSION, op)?;

        Ok(user)
    }
}

/// cookieからUserを取得する<br>
/// Userを取得するには、セッションが有効な期間中に呼び出す必要がある<br>
/// <br>
/// # NOTE
/// Userを利用するときは、原則としてハンドラのパラメータにUserを指定する方法を選択する（前記方法だと利用規約の同意しているかの確認も同時に行うため）
/// 本関数は、Userの情報を使いたいが、利用規約の同意を確認したくないケース（ex. ユーザーから利用規約の同意を得るケース）のみに利用する<br>
/// <br>
/// # Errors
/// 下記の場合、ステータスコード401、エラーコード[UNAUTHORIZED]を返す<br>
/// <ul>
///   <li>Cookieがない場合</li>
///   <li>CookieにセッションIDが含まれていない場合</li>
///   <li>既にセッションの有効期限が切れている場合</li>
/// </ul>
pub(crate) async fn get_user_by_cookie(
    option_cookie: Option<Cookie>,
    store: &impl SessionStore,
) -> Result<User, ErrResp> {
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

fn check_if_user_has_already_agreed(
    id: i32,
    terms_of_use_version: i32,
    op: impl TermsOfUseLoadOperation,
) -> Result<(), ErrResp> {
    let results = op.load(id, terms_of_use_version)?;
    let len = results.len();
    if len == 0 {
        tracing::info!(
            "id ({}) has not agreed terms of use version ({}) yet",
            id,
            terms_of_use_version
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: NOT_TERMS_OF_USE_AGREED_YET,
            }),
        ));
    }
    if len > 1 {
        // NOTE: primary keyで検索しているため、ここを通るケースはdieselの障害
        panic!(
            "number of terms of use (id: {}, version: {}): {}",
            id, terms_of_use_version, len
        )
    }
    Ok(())
}

/// テストコードで共通で使うコードをまとめるモジュール
#[cfg(test)]
pub(crate) mod tests {
    use async_session::{MemoryStore, Session, SessionStore};
    use axum::http::StatusCode;
    use chrono::TimeZone;
    use common::{model::user::TermsOfUse, ErrResp};
    use headers::{Cookie, HeaderMap, HeaderMapExt, HeaderValue};

    use crate::{
        err_code,
        util::{
            session::{create_cookie_format, get_user_by_cookie, KEY_TO_USER_ACCOUNT_ID},
            terms_of_use::TermsOfUseLoadOperation,
        },
    };

    use super::{check_if_user_has_already_agreed, COOKIE_NAME};

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

    /// 有効期限がないセッションを作成し、そのセッションにアクセスするためのセッションIDを返す
    pub(crate) async fn prepare_session(user_account_id: i32, store: &impl SessionStore) -> String {
        let mut session = Session::new();
        // 実行環境（PCの性能）に依存させないように、テストコード内ではexpiryは設定しない
        let _ = session
            .insert(KEY_TO_USER_ACCOUNT_ID, user_account_id)
            .expect("failed to get Ok");
        store
            .store_session(session)
            .await
            .expect("failed to get Ok")
            .expect("failed to get value")
    }

    pub(crate) fn prepare_cookie(session_id_value: &str) -> Option<Cookie> {
        let mut headers = HeaderMap::new();
        let header_value = create_cookie_format(session_id_value)
            .parse::<HeaderValue>()
            .expect("failed to get Ok");
        headers.insert("cookie", header_value);
        headers.typed_get::<Cookie>()
    }

    pub(crate) async fn remove_session_from_store(
        session_id_value: &str,
        store: &impl SessionStore,
    ) {
        let loaded_session = store
            .load_session(session_id_value.to_string())
            .await
            .expect("failed to get Ok")
            .expect("failed to get value");
        let _ = store
            .destroy_session(loaded_session)
            .await
            .expect("failed to get Ok");
    }

    #[tokio::test]
    async fn get_user_by_cookie_success() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id_value = prepare_session(user_account_id, &store).await;
        let option_cookie = prepare_cookie(&session_id_value);
        assert_eq!(1, store.count().await);

        let user = get_user_by_cookie(option_cookie, &store)
            .await
            .expect("failed to get Ok");

        // get_user_by_cookieが何らかの副作用でセッションを破棄していないか確認
        // 補足説明:
        // 実際の運用ではセッションに有効期限をもたせるので、
        // get_user_by_cookieの後にセッションの有効期限が切れて0になることもあり得る。
        // しかし、テストケースではセッションに有効期限を持たせていないため、0にはならない。
        assert_eq!(1, store.count().await);
        assert_eq!(user_account_id, user.account_id);
    }

    #[tokio::test]
    async fn get_user_by_cookie_fail_no_cookie() {
        let option_cookie: Option<Cookie> = None;
        let store = MemoryStore::new();

        let result = get_user_by_cookie(option_cookie, &store)
            .await
            .expect_err("failed to get Err");

        assert_eq!(StatusCode::UNAUTHORIZED, result.0);
        assert_eq!(err_code::UNAUTHORIZED, result.1 .0.code);
    }

    #[tokio::test]
    async fn get_user_by_cookie_fail_incorrect_cookie() {
        let mut headers = HeaderMap::new();
        let header_value = "name=taro"
            .parse::<HeaderValue>()
            .expect("failed to get Ok");
        headers.insert("cookie", header_value);
        let option_cookie = headers.typed_get::<Cookie>();
        let store = MemoryStore::new();

        let result = get_user_by_cookie(option_cookie, &store)
            .await
            .expect_err("failed to get Err");

        assert_eq!(StatusCode::UNAUTHORIZED, result.0);
        assert_eq!(err_code::UNAUTHORIZED, result.1 .0.code);
    }

    #[tokio::test]
    async fn get_user_by_cookie_fail_session_already_expired() {
        let user_account_id = 10002;
        let store = MemoryStore::new();
        let session_id_value = prepare_session(user_account_id, &store).await;
        let option_cookie = prepare_cookie(&session_id_value);
        // リクエストのプリプロセス前ににセッションを削除
        let _ = remove_session_from_store(&session_id_value, &store).await;
        assert_eq!(0, store.count().await);

        let result = get_user_by_cookie(option_cookie, &store)
            .await
            .expect_err("failed to get Err");

        assert_eq!(0, store.count().await);
        assert_eq!(StatusCode::UNAUTHORIZED, result.0);
        assert_eq!(err_code::UNAUTHORIZED, result.1 .0.code);
    }

    struct TermsOfUseLoadOperationMock {
        has_already_agreed: bool,
    }

    impl TermsOfUseLoadOperationMock {
        fn new(has_already_agreed: bool) -> Self {
            Self { has_already_agreed }
        }
    }

    impl TermsOfUseLoadOperation for TermsOfUseLoadOperationMock {
        fn load(&self, id: i32, terms_of_use_version: i32) -> Result<Vec<TermsOfUse>, ErrResp> {
            if !self.has_already_agreed {
                return Ok(vec![]);
            }
            let terms_of_use = TermsOfUse {
                user_account_id: id,
                ver: terms_of_use_version,
                email_address: "test@example.com".to_string(),
                agreed_at: chrono::Utc.ymd(2021, 11, 5).and_hms(20, 00, 40),
            };
            Ok(vec![terms_of_use])
        }
    }

    #[test]
    fn check_if_user_has_already_agreed_success_user_has_already_agreed() {
        let user_account_id = 10002;
        let terms_of_use_version = 1;
        let op = TermsOfUseLoadOperationMock::new(true);

        let result = check_if_user_has_already_agreed(user_account_id, terms_of_use_version, op);

        result.expect("failed to get Ok");
    }

    #[test]
    fn check_if_user_has_already_agreed_fail_user_has_not_agreed_yet() {
        let user_account_id = 10002;
        let terms_of_use_version = 1;
        let op = TermsOfUseLoadOperationMock::new(false);

        let result = check_if_user_has_already_agreed(user_account_id, terms_of_use_version, op)
            .expect_err("failed to get Err");

        assert_eq!(StatusCode::BAD_REQUEST, result.0);
        assert_eq!(err_code::NOT_TERMS_OF_USE_AGREED_YET, result.1 .0.code);
    }
}
