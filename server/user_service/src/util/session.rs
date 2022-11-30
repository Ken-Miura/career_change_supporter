// Copyright 2021 Ken Miura

use async_redis_session::RedisSessionStore;
use async_session::{Session, SessionStore};
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::{extract::Extension, http::StatusCode, Json};
use common::{ApiError, ErrResp};
use entity::sea_orm::DatabaseConnection;
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::env::var;
use std::time::Duration;
use tower_cookies::{Cookies, Key};
use tracing::{error, info};

use crate::err::{
    unexpected_err_resp,
    Code::{NoAccountFound, NotTermsOfUseAgreedYet, Unauthorized},
};

use super::disabled_check::{DisabledCheckOperation, DisabledCheckOperationImpl};
use super::terms_of_use::{
    TermsOfUseLoadOperation, TermsOfUseLoadOperationImpl, TERMS_OF_USE_VERSION,
};

pub(crate) const KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_USER_APP: &str =
    "KEY_OF_SIGNED_COOKIE_FOR_USER_APP";
pub(crate) static KEY_OF_SIGNED_COOKIE_FOR_USER_APP: Lazy<Key> = Lazy::new(|| {
    let key_str = var(KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_USER_APP).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_USER_APP
        )
    });
    let size = key_str.len();
    if size < 64 {
        panic!(
            "Size of \"{}\" value regarded as utf-8 encoding must be at least 64 bytes",
            KEY_TO_KEY_OF_SIGNED_COOKIE_FOR_USER_APP
        )
    }
    Key::from(key_str.as_bytes())
});

pub(crate) const SESSION_ID_COOKIE_NAME: &str = "session_id";
pub(crate) const KEY_TO_USER_ACCOUNT_ID: &str = "user_account_id";
const LENGTH_OF_MEETING: u64 = 60;
const TIME_FOR_SUBSEQUENT_OPERATIONS: u64 = 10;

/// セッションの有効期限
pub(crate) const LOGIN_SESSION_EXPIRY: Duration =
    Duration::from_secs(60 * (LENGTH_OF_MEETING + TIME_FOR_SUBSEQUENT_OPERATIONS));

/// ユーザーの情報にアクセスするためのID
///
/// ハンドラ関数内でユーザーの情報にアクセスしたい場合、原則としてこの型をパラメータとして受け付ける。
/// このパラメータに含むIDを用いて、データベースからユーザー情報を取得できる。
/// この型をパラメータとして受け付けると、ハンドラ関数の処理に入る前に下記の前処理を実施する。
/// <ul>
///   <li>ログインセッションが有効であることを確認</li>
///   <li>アカウントが無効でないこと</li>
///   <li>利用規約に同意済みである確認</li>
/// </ul>
#[derive(Deserialize, Clone, Debug)]
pub(crate) struct User {
    pub(crate) account_id: i64,
}

#[async_trait]
impl<S> FromRequestParts<S> for User
where
    S: Send + Sync,
{
    type Rejection = ErrResp;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Extension(cookies) = Extension::<Cookies>::from_request_parts(parts, state)
            .await
            .map_err(|e| {
                error!("failed to get cookies: {}", e);
                unexpected_err_resp()
            })?;
        let signed_cookies = cookies.signed(&KEY_OF_SIGNED_COOKIE_FOR_USER_APP);
        let option_cookie = signed_cookies.get(SESSION_ID_COOKIE_NAME);
        let session_id = match option_cookie {
            Some(s) => s.value().to_string(),
            None => {
                info!("no sessoin cookie found");
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(ApiError {
                        code: Unauthorized as u32,
                    }),
                ));
            }
        };
        let Extension(store) = Extension::<RedisSessionStore>::from_request_parts(parts, state)
            .await
            .map_err(|e| {
                error!("failed to get session store: {}", e);
                unexpected_err_resp()
            })?;
        let op = RefreshOperationImpl {};
        let user = get_user_by_session_id(session_id, &store, op, LOGIN_SESSION_EXPIRY).await?;

        let Extension(pool) = Extension::<DatabaseConnection>::from_request_parts(parts, state)
            .await
            .map_err(|e| {
                error!("failed to extract connection pool from parts: {}", e);
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

/// session_idを使いstoreから、Userを取得する。<br>
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
///   <li>既にセッションの有効期限が切れている場合</li>
/// </ul>
pub(crate) async fn get_user_by_session_id(
    session_id: String,
    store: &impl SessionStore,
    op: impl RefreshOperation,
    expiry: Duration,
) -> Result<User, ErrResp> {
    let option_session = store.load_session(session_id.clone()).await.map_err(|e| {
        error!("failed to load session: {}", e);
        unexpected_err_resp()
    })?;
    let mut session = match option_session {
        Some(s) => s,
        None => {
            info!("no session found");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: Unauthorized as u32,
                }),
            ));
        }
    };
    let account_id = match session.get::<i64>(KEY_TO_USER_ACCOUNT_ID) {
        Some(id) => id,
        None => {
            error!("failed to get account id from session");
            return Err(unexpected_err_resp());
        }
    };
    op.set_login_session_expiry(&mut session, expiry);
    // 新たなexpiryを設定したsessionをstoreに保存することでセッション期限を延長する
    let _ = store.store_session(session).await.map_err(|e| {
        error!("failed to store session: {}", e);
        unexpected_err_resp()
    })?;
    Ok(User { account_id })
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
    account_id: i64,
    terms_of_use_version: i32,
    op: impl TermsOfUseLoadOperation,
) -> Result<(), ErrResp> {
    let option = op.find(account_id, terms_of_use_version).await?;
    let _ = option.ok_or_else(|| {
        error!(
            "account id ({}) has not agreed terms of use (version {}) yet",
            account_id, terms_of_use_version
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: NotTermsOfUseAgreedYet as u32,
            }),
        )
    })?;
    Ok(())
}

async fn ensure_account_is_not_disabled(
    account_id: i64,
    op: impl DisabledCheckOperation,
) -> Result<(), ErrResp> {
    let result = op
        .check_if_account_is_disabled(account_id)
        .await
        .map_err(|e| {
            error!(
                "failed to check if account is disabled (status code: {}, code: {})",
                e.0, e.1 .0.code
            );
            unexpected_err_resp()
        })?;
    if let Some(disabled) = result {
        if disabled {
            error!("account (account id: {}) is disabled", account_id);
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
        error!("no account (account id: {}) found", account_id);
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
    use common::ErrResp;

    use crate::{
        err,
        util::{
            disabled_check::DisabledCheckOperation,
            session::{get_user_by_session_id, KEY_TO_USER_ACCOUNT_ID, LOGIN_SESSION_EXPIRY},
            terms_of_use::{TermsOfUseData, TermsOfUseLoadOperation},
        },
    };

    use super::{
        check_if_user_has_already_agreed, ensure_account_is_not_disabled, RefreshOperation,
    };

    /// 有効期限がないセッションを作成し、そのセッションにアクセスするためのセッションIDを返す
    pub(crate) async fn prepare_session(user_account_id: i64, store: &impl SessionStore) -> String {
        let mut session = Session::new();
        // 実行環境（PCの性能）に依存させないように、テストコード内ではexpiryは設定しない
        session
            .insert(KEY_TO_USER_ACCOUNT_ID, user_account_id)
            .expect("failed to get Ok");
        store
            .store_session(session)
            .await
            .expect("failed to get Ok")
            .expect("failed to get value")
    }

    pub(crate) async fn remove_session_from_store(session_id: &str, store: &impl SessionStore) {
        let loaded_session = store
            .load_session(session_id.to_string())
            .await
            .expect("failed to get Ok")
            .expect("failed to get value");
        store
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
    async fn get_user_by_session_id_success() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id = prepare_session(user_account_id, &store).await;
        assert_eq!(1, store.count().await);

        let op = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };
        let user = get_user_by_session_id(session_id, &store, op, LOGIN_SESSION_EXPIRY)
            .await
            .expect("failed to get Ok");

        // get_user_by_session_idが何らかの副作用でセッションを破棄していないか確認
        // 補足説明:
        // 実際の運用ではセッションに有効期限をもたせるので、
        // get_user_by_session_idの後にセッションの有効期限が切れて0になることもあり得る。
        // しかし、テストケースではセッションに有効期限を持たせていないため、暗黙のうちに0になる心配はない。
        assert_eq!(1, store.count().await);
        assert_eq!(user_account_id, user.account_id);
    }

    #[tokio::test]
    async fn get_user_by_session_id_fail_session_already_expired() {
        let user_account_id = 10002;
        let store = MemoryStore::new();
        let session_id = prepare_session(user_account_id, &store).await;
        // リクエストのプリプロセス前ににセッションを削除
        remove_session_from_store(&session_id, &store).await;
        assert_eq!(0, store.count().await);

        let op = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };
        let result = get_user_by_session_id(session_id, &store, op, LOGIN_SESSION_EXPIRY)
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
            _account_id: i64,
            _terms_of_use_version: i32,
        ) -> Result<Option<TermsOfUseData>, ErrResp> {
            if !self.has_already_agreed {
                return Ok(None);
            }
            let terms_of_use_data = TermsOfUseData {};
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
        account_id: i64,
        no_account_found: bool,
        account_disabled: bool,
    }

    #[async_trait]
    impl DisabledCheckOperation for DisabledCheckOperationMock {
        async fn check_if_account_is_disabled(
            &self,
            account_id: i64,
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

        result.expect("failed to get Ok");
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
