// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::async_trait;
use axum::{
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
    Json,
};
use common::{ApiError, ErrResp};
use cookie::SameSite;
use entity::sea_orm::DatabaseConnection;
use serde::Deserialize;
use std::time::Duration;
use tower_cookies::{Cookie, Cookies};

use crate::{
    err::{
        unexpected_err_resp,
        Code::{NoAccountFound, NotTermsOfUseAgreedYet, Unauthorized},
    },
    util::ROOT_PATH,
};

use super::disabled_check::{DisabledCheckOperation, DisabledCheckOperationImpl};
use super::terms_of_use::{
    TermsOfUseLoadOperation, TermsOfUseLoadOperationImpl, TERMS_OF_USE_VERSION,
};

pub(crate) const SESSION_ID_COOKIE_NAME: &str = "session_id";
pub(crate) const KEY_TO_USER_ACCOUNT_ID: &str = "user_account_id";
const LENGTH_OF_MEETING: u64 = 60;
const TIME_FOR_SUBSEQUENT_OPERATIONS: u64 = 10;

/// セッションの有効期限
pub(crate) const LOGIN_SESSION_EXPIRY: Duration =
    Duration::from_secs(60 * (LENGTH_OF_MEETING + TIME_FOR_SUBSEQUENT_OPERATIONS));

/// [SESSION_ID_COOKIE_NAME]を含むSet-Cookie用の文字列を返す。
pub(crate) fn create_cookie_format(session_id_value: &str) -> String {
    Cookie::build(SESSION_ID_COOKIE_NAME, session_id_value)
        .same_site(SameSite::Strict)
        .path(ROOT_PATH)
        .secure(true)
        .http_only(true)
        .finish()
        .to_string()
}

/// [SESSION_ID_COOKIE_NAME]を含む、有効期限切れのSet-Cookie用の文字列を返す<br>
/// ブラウザに保存されたCookieの削除指示を出したいときに使う。
pub(crate) fn create_expired_cookie_format() -> String {
    let mut cookie = Cookie::build(SESSION_ID_COOKIE_NAME, "")
        .same_site(SameSite::Strict)
        .path(ROOT_PATH)
        .secure(true)
        .http_only(true)
        .finish();
    cookie.make_removal();
    cookie.to_string()
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
        let Extension(cookies) = Extension::<Cookies>::from_request(req).await.map_err(|e| {
            tracing::error!("failed to get cookies: {}", e);
            unexpected_err_resp()
        })?;
        let Extension(store) = Extension::<RedisSessionStore>::from_request(req)
            .await
            .map_err(|e| {
                tracing::error!("failed to get session store: {}", e);
                unexpected_err_resp()
            })?;
        let op = RefreshOperationImpl {};
        let user = get_user_by_cookie(cookies, &store, op, LOGIN_SESSION_EXPIRY).await?;

        let Extension(pool) = Extension::<DatabaseConnection>::from_request(req)
            .await
            .map_err(|e| {
                tracing::error!("failed to extract connection pool from req: {}", e);
                unexpected_err_resp()
            })?;

        let disabled_check_op = DisabledCheckOperationImpl::new(pool.clone());
        let _ = ensure_account_is_not_disabled(user.account_id, disabled_check_op).await?;

        let terms_of_use_op = TermsOfUseLoadOperationImpl::new(pool);
        let _ = check_if_user_has_already_agreed(
            user.account_id,
            *TERMS_OF_USE_VERSION,
            terms_of_use_op,
        )
        .await?;

        Ok(user)
    }
}

/// storeからcookieを使い、Userを取得する。<br>
/// Userを取得するには、セッションが有効な期間中に呼び出す必要がある。<br>
/// Userの取得に成功した場合、セッションの有効期間がexpiryだけ延長される（※）<br>
/// <br>
/// （※）いわゆるアイドルタイムアウト。アブソリュートタイムアウトとリニューアルタイムアウトは実装しない。<br>
/// リニューアルタイムアウトが必要になった場合は、セッションを保存しているキャッシュシステムの定期再起動により実装する。<br>
/// 参考:
///   セッションタイムアウトの種類: <https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html#automatic-session-expiration>
///   著名なフレームワークにおいてもアイドルタイムアウトが一般的で、アブソリュートタイムアウトは実装されていない
///     1. <https://stackoverflow.com/questions/62964012/how-to-set-absolute-session-timeout-for-a-spring-session>
///     2. <https://www.webdevqa.jp.net/ja/authentication/%E3%82%BB%E3%83%83%E3%82%B7%E3%83%A7%E3%83%B3%E3%81%AE%E7%B5%B6%E5%AF%BE%E3%82%BF%E3%82%A4%E3%83%A0%E3%82%A2%E3%82%A6%E3%83%88%E3%81%AF%E3%81%A9%E3%82%8C%E3%81%8F%E3%82%89%E3%81%84%E3%81%AE%E9%95%B7%E3%81%95%E3%81%A7%E3%81%99%E3%81%8B%EF%BC%9F/l968265546/>
/// # NOTE
/// Userを利用するときは、原則としてハンドラのパラメータにUserを指定する方法を選択する（前記方法だと利用規約の同意しているかの確認も同時に行うため）
/// 本関数は、Userの情報を使いたいが、利用規約の同意を確認したくないケース（ex. ユーザーから利用規約の同意を得るケース）のみに利用する<br>
/// <br>
/// # Errors
/// 下記の場合、ステータスコード401、エラーコード[Unauthorized]を返す<br>
/// <ul>
///   <li>Cookieがない場合</li>
///   <li>CookieにセッションIDが含まれていない場合</li>
///   <li>既にセッションの有効期限が切れている場合</li>
/// </ul>
pub(crate) async fn get_user_by_cookie(
    cookies: Cookies,
    store: &impl SessionStore,
    op: impl RefreshOperation,
    expiry: Duration,
) -> Result<User, ErrResp> {
    let option_cookie = cookies.get(SESSION_ID_COOKIE_NAME);
    let session_id_value = match option_cookie {
        Some(session_id) => session_id.value().to_string(),
        None => {
            tracing::debug!("no valid cookie on request");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: Unauthorized as u32,
                }),
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
    let mut session = match option_session {
        Some(s) => s,
        None => {
            tracing::debug!("no valid session on request");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: Unauthorized as u32,
                }),
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
    op.set_login_session_expiry(&mut session, expiry);
    // 新たなexpiryを設定したsessionをstoreに保存することでセッション期限を延長する
    let _ = store.store_session(session).await.map_err(|e| {
        tracing::error!(
            "failed to store session (session_id={}): {}",
            session_id_value,
            e
        );
        unexpected_err_resp()
    })?;
    Ok(User { account_id: id })
}

pub(crate) trait RefreshOperation {
    fn set_login_session_expiry(&self, session: &mut Session, expiry: Duration);
}

pub(crate) struct RefreshOperationImpl {}

impl RefreshOperation for RefreshOperationImpl {
    fn set_login_session_expiry(&self, session: &mut Session, expiry: Duration) {
        session.expire_in(expiry);
    }
}

async fn check_if_user_has_already_agreed(
    account_id: i32,
    terms_of_use_version: i32,
    op: impl TermsOfUseLoadOperation,
) -> Result<(), ErrResp> {
    let option = op.find(account_id, terms_of_use_version).await?;
    let terms_of_use_data = option.ok_or_else(|| {
        tracing::info!(
            "account id ({}) has not agreed terms of use version {} yet",
            account_id,
            terms_of_use_version
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: NotTermsOfUseAgreedYet as u32,
            }),
        )
    })?;
    tracing::debug!("accound (id: {}, email address: {}) has already agreed with terms of use (version: {}) at {}", 
        terms_of_use_data.user_account_id,
        terms_of_use_data.email_address,
        terms_of_use_data.ver,
        terms_of_use_data.agreed_at);
    Ok(())
}

async fn ensure_account_is_not_disabled(
    account_id: i32,
    op: impl DisabledCheckOperation,
) -> Result<(), ErrResp> {
    let result = op
        .check_if_account_is_disabled(account_id)
        .await
        .map_err(|e| {
            tracing::error!(
                "failed to ensure account is not disabled (status code: {}, code: {})",
                e.0,
                e.1 .0.code
            );
            unexpected_err_resp()
        })?;
    if let Some(disabled) = result {
        if disabled {
            // セッションチェックの際に無効化を検出した際は、Unauthorizedを返すことでログイン画面へ遷移させる
            // ログイン画面でログインしようとした際に無効化を知らせるメッセージを表示
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: Unauthorized as u32,
                }),
            ));
        };
        Ok(())
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: NoAccountFound as u32,
            }),
        ))
    }
}

/// テストコードで共通で使うコードをまとめるモジュール
#[cfg(test)]
pub(crate) mod tests {
    use async_session::{MemoryStore, Session, SessionStore};
    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::TimeZone;
    use common::ErrResp;
    use cookie::{Cookie, SameSite};
    use headers::HeaderValue;
    use tower_cookies::Cookies;

    use crate::{
        err,
        util::{
            disabled_check::DisabledCheckOperation,
            session::{get_user_by_cookie, KEY_TO_USER_ACCOUNT_ID, LOGIN_SESSION_EXPIRY},
            terms_of_use::{TermsOfUseData, TermsOfUseLoadOperation},
            JAPANESE_TIME_ZONE, ROOT_PATH,
        },
    };

    use super::{
        check_if_user_has_already_agreed, ensure_account_is_not_disabled, RefreshOperation,
        SESSION_ID_COOKIE_NAME,
    };

    pub(crate) fn extract_session_id_value(header_value: &HeaderValue) -> String {
        let set_cookie = header_value.to_str().expect("failed to get value");
        let cookie_name = set_cookie
            .split(';')
            .find(|s| s.contains(SESSION_ID_COOKIE_NAME))
            .expect("failed to get session")
            .trim()
            .split_once("=")
            .expect("failed to get value");
        cookie_name.1.to_string()
    }

    pub(crate) fn extract_cookie_max_age_value(header_value: &HeaderValue) -> String {
        let set_cookie = header_value.to_str().expect("failed to get value");
        let cookie_max_age = set_cookie
            .split(';')
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

    pub(crate) fn prepare_cookies(session_id_value: &str) -> Cookies {
        let cookie = Cookie::build(SESSION_ID_COOKIE_NAME, session_id_value.to_string())
            .same_site(SameSite::Strict)
            .path(ROOT_PATH)
            .secure(true)
            .http_only(true)
            .finish();
        let cookies = Cookies::default();
        cookies.add(cookie.clone());
        cookies
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

    struct RefreshOperationMock {
        expiry: std::time::Duration,
    }

    impl RefreshOperation for RefreshOperationMock {
        fn set_login_session_expiry(
            &self,
            _session: &mut async_session::Session,
            expiry: std::time::Duration,
        ) {
            assert_eq!(self.expiry, expiry);
        }
    }

    #[tokio::test]
    async fn get_user_by_cookie_success() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id_value = prepare_session(user_account_id, &store).await;
        let cookies = prepare_cookies(&session_id_value);
        assert_eq!(1, store.count().await);

        let op = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };
        let user = get_user_by_cookie(cookies, &store, op, LOGIN_SESSION_EXPIRY)
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
        let cookies = Cookies::default();
        let store = MemoryStore::new();

        let op = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };
        let result = get_user_by_cookie(cookies, &store, op, LOGIN_SESSION_EXPIRY)
            .await
            .expect_err("failed to get Err");

        assert_eq!(StatusCode::UNAUTHORIZED, result.0);
        assert_eq!(err::Code::Unauthorized as u32, result.1 .0.code);
    }

    #[tokio::test]
    async fn get_user_by_cookie_fail_incorrect_cookie() {
        let cookies = Cookies::default();
        let cookie = Cookie::new("name", "taro");
        cookies.add(cookie);
        let store = MemoryStore::new();

        let op = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };
        let result = get_user_by_cookie(cookies, &store, op, LOGIN_SESSION_EXPIRY)
            .await
            .expect_err("failed to get Err");

        assert_eq!(StatusCode::UNAUTHORIZED, result.0);
        assert_eq!(err::Code::Unauthorized as u32, result.1 .0.code);
    }

    #[tokio::test]
    async fn get_user_by_cookie_fail_session_already_expired() {
        let user_account_id = 10002;
        let store = MemoryStore::new();
        let session_id_value = prepare_session(user_account_id, &store).await;
        let cookies = prepare_cookies(&session_id_value);
        // リクエストのプリプロセス前ににセッションを削除
        let _ = remove_session_from_store(&session_id_value, &store).await;
        assert_eq!(0, store.count().await);

        let op = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };
        let result = get_user_by_cookie(cookies, &store, op, LOGIN_SESSION_EXPIRY)
            .await
            .expect_err("failed to get Err");

        assert_eq!(0, store.count().await);
        assert_eq!(StatusCode::UNAUTHORIZED, result.0);
        assert_eq!(err::Code::Unauthorized as u32, result.1 .0.code);
    }

    struct TermsOfUseLoadOperationMock {
        has_already_agreed: bool,
    }

    impl TermsOfUseLoadOperationMock {
        fn new(has_already_agreed: bool) -> Self {
            Self { has_already_agreed }
        }
    }

    #[async_trait]
    impl TermsOfUseLoadOperation for TermsOfUseLoadOperationMock {
        async fn find(
            &self,
            account_id: i32,
            terms_of_use_version: i32,
        ) -> Result<Option<TermsOfUseData>, ErrResp> {
            if !self.has_already_agreed {
                return Ok(None);
            }
            let terms_of_use_data = TermsOfUseData {
                user_account_id: account_id,
                ver: terms_of_use_version,
                email_address: "test@example.com".to_string(),
                agreed_at: chrono::Utc
                    .ymd(2021, 11, 5)
                    .and_hms(20, 00, 40)
                    .with_timezone(&JAPANESE_TIME_ZONE.to_owned()),
            };
            Ok(Some(terms_of_use_data))
        }
    }

    #[tokio::test]
    async fn check_if_user_has_already_agreed_success_user_has_already_agreed() {
        let user_account_id = 10002;
        let terms_of_use_version = 1;
        let op = TermsOfUseLoadOperationMock::new(true);

        let result =
            check_if_user_has_already_agreed(user_account_id, terms_of_use_version, op).await;

        result.expect("failed to get Ok");
    }

    #[tokio::test]
    async fn check_if_user_has_already_agreed_fail_user_has_not_agreed_yet() {
        let user_account_id = 10002;
        let terms_of_use_version = 1;
        let op = TermsOfUseLoadOperationMock::new(false);

        let result = check_if_user_has_already_agreed(user_account_id, terms_of_use_version, op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(StatusCode::BAD_REQUEST, result.0);
        assert_eq!(err::Code::NotTermsOfUseAgreedYet as u32, result.1 .0.code);
    }

    struct DisabledCheckOperationMock {
        account_id: i32,
        no_account_found: bool,
        account_disabled: bool,
    }

    #[async_trait]
    impl DisabledCheckOperation for DisabledCheckOperationMock {
        async fn check_if_account_is_disabled(
            &self,
            account_id: i32,
        ) -> Result<Option<bool>, ErrResp> {
            assert_eq!(self.account_id, account_id);
            if self.no_account_found {
                return Ok(None);
            }
            Ok(Some(self.account_disabled))
        }
    }

    #[tokio::test]
    async fn ensure_account_is_not_disabled_success() {
        let account_id = 2345;
        let op_mock = DisabledCheckOperationMock {
            account_id,
            no_account_found: false,
            account_disabled: false,
        };

        let result = ensure_account_is_not_disabled(account_id, op_mock).await;

        let _ = result.expect("failed to get Ok");
    }

    #[tokio::test]
    async fn ensure_account_is_not_disabled_fail_no_account_found() {
        let account_id = 2345;
        let op_mock = DisabledCheckOperationMock {
            account_id,
            no_account_found: true,
            account_disabled: false,
        };

        let result = ensure_account_is_not_disabled(account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(err::Code::NoAccountFound as u32, resp.1 .0.code);
    }

    #[tokio::test]
    async fn ensure_account_is_not_disabled_fail_unauthorized() {
        let account_id = 2345;
        let op_mock = DisabledCheckOperationMock {
            account_id,
            no_account_found: false,
            account_disabled: true,
        };

        let result = ensure_account_is_not_disabled(account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::UNAUTHORIZED, resp.0);
        assert_eq!(err::Code::Unauthorized as u32, resp.1 .0.code);
    }
}
