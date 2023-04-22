// Copyright 2021 Ken Miura

use async_session::{Session, SessionStore};
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::{http::StatusCode, Json};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::SignedCookieJar;
use common::{ApiError, AppState, ErrResp};
use entity::sea_orm::DatabaseConnection;
use std::time::Duration;
use tracing::{error, info};

pub(crate) mod agreement_unchecked_user;
pub(crate) mod user;
pub(crate) mod verified_user;

use crate::err::unexpected_err_resp;
use crate::err::Code;

use super::get_user_info_if_available;
use super::identity_check::{IdentityCheckOperation, IdentityCheckOperationImpl};
use super::login_status::LoginStatus;
use super::request_consultation::LENGTH_OF_MEETING_IN_MINUTE;
use super::terms_of_use::{
    TermsOfUseLoadOperation, TermsOfUseLoadOperationImpl, TERMS_OF_USE_VERSION,
};
use super::user_info::{FindUserInfoOperationImpl, UserInfo};

pub(crate) const SESSION_ID_COOKIE_NAME: &str = "session_id";
pub(crate) const KEY_TO_USER_ACCOUNT_ID: &str = "user_account_id";
pub(crate) const KEY_TO_LOGIN_STATUS: &str = "login_status";
const TIME_FOR_SUBSEQUENT_OPERATIONS: u64 = 10;

/// セッションの有効期限
pub(crate) const LOGIN_SESSION_EXPIRY: Duration =
    Duration::from_secs(60 * (LENGTH_OF_MEETING_IN_MINUTE + TIME_FOR_SUBSEQUENT_OPERATIONS));

async fn get_agreement_unchecked_user_info_from_request_parts<S>(
    parts: &mut Parts,
    state: &S,
) -> Result<UserInfo, ErrResp>
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    let signed_cookies = extract_singed_jar_from_request_parts(parts, state).await?;
    let option_cookie = signed_cookies.get(SESSION_ID_COOKIE_NAME);

    let app_state = AppState::from_ref(state);
    let store = app_state.store;
    let pool = app_state.pool;

    let user_info =
        get_agreement_unchecked_user_info_from_cookie(option_cookie, &store, &pool).await?;

    Ok(user_info)
}

async fn get_user_info_from_request_parts<S>(
    parts: &mut Parts,
    state: &S,
) -> Result<UserInfo, ErrResp>
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    let signed_cookies = extract_singed_jar_from_request_parts(parts, state).await?;
    let option_cookie = signed_cookies.get(SESSION_ID_COOKIE_NAME);

    let app_state = AppState::from_ref(state);
    let store = app_state.store;
    let pool = app_state.pool;

    let user_info = get_user_info_from_cookie(option_cookie, &store, &pool).await?;

    Ok(user_info)
}

async fn get_verified_user_info_from_request_parts<S>(
    parts: &mut Parts,
    state: &S,
) -> Result<UserInfo, ErrResp>
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    let signed_cookies = extract_singed_jar_from_request_parts(parts, state).await?;
    let option_cookie = signed_cookies.get(SESSION_ID_COOKIE_NAME);

    let app_state = AppState::from_ref(state);
    let store = app_state.store;
    let pool = app_state.pool;

    let user_info = get_verified_user_info_from_cookie(option_cookie, &store, &pool).await?;

    Ok(user_info)
}

async fn extract_singed_jar_from_request_parts<S>(
    parts: &mut Parts,
    state: &S,
) -> Result<SignedCookieJar<AppState>, ErrResp>
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    let signed_cookies = SignedCookieJar::<AppState>::from_request_parts(parts, state)
        .await
        .map_err(|e| {
            error!("failed to get cookies: {:?}", e);
            unexpected_err_resp()
        })?;
    Ok(signed_cookies)
}

pub(crate) async fn get_agreement_unchecked_user_info_from_cookie(
    option_cookie: Option<Cookie<'_>>,
    store: &impl SessionStore,
    pool: &DatabaseConnection,
) -> Result<UserInfo, ErrResp> {
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

    let refresh_op = RefreshOperationImpl {};
    let user_account_id =
        get_user_account_id_by_session_id(session_id, store, refresh_op, LOGIN_SESSION_EXPIRY)
            .await?;

    let find_user_op = FindUserInfoOperationImpl::new(pool);
    let user_info = get_user_info_if_available(user_account_id, &find_user_op).await?;

    Ok(user_info)
}

pub(crate) async fn get_user_info_from_cookie(
    option_cookie: Option<Cookie<'_>>,
    store: &impl SessionStore,
    pool: &DatabaseConnection,
) -> Result<UserInfo, ErrResp> {
    let user_info =
        get_agreement_unchecked_user_info_from_cookie(option_cookie, store, pool).await?;

    let terms_of_use_op = TermsOfUseLoadOperationImpl::new(pool);
    check_if_user_has_already_agreed(user_info.account_id, *TERMS_OF_USE_VERSION, terms_of_use_op)
        .await?;

    Ok(user_info)
}

pub(crate) async fn get_verified_user_info_from_cookie(
    option_cookie: Option<Cookie<'_>>,
    store: &impl SessionStore,
    pool: &DatabaseConnection,
) -> Result<UserInfo, ErrResp> {
    let user_info = get_user_info_from_cookie(option_cookie, store, pool).await?;

    let op = IdentityCheckOperationImpl::new(pool);
    ensure_identity_exists(user_info.account_id, &op).await?;

    Ok(user_info)
}

/// session_idを使い、storeからユーザーを一意に識別する値を取得する。<br>
/// この値を取得するには、セッションが有効な期間中に呼び出す必要がある。<br>
/// 取得に成功した後、ログイン処理が完了の状態に限り、セッションの有効期間がexpiryだけ延長される（※）<br>
/// <br>
/// （※）いわゆるアイドルタイムアウト。アブソリュートタイムアウトとリニューアルタイムアウトは実装しない。<br>
/// リニューアルタイムアウトが必要になった場合は、セッションを保存しているキャッシュシステムの定期再起動により実装する。<br>
/// 参考:
///   セッションタイムアウトの種類: <https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html#automatic-session-expiration>
///   著名なフレームワークにおいてもアイドルタイムアウトが一般的で、アブソリュートタイムアウトは実装されていない
///     1. <https://stackoverflow.com/questions/62964012/how-to-set-absolute-session-timeout-for-a-spring-session>
///     2. <https://www.webdevqa.jp.net/ja/authentication/%E3%82%BB%E3%83%83%E3%82%B7%E3%83%A7%E3%83%B3%E3%81%AE%E7%B5%B6%E5%AF%BE%E3%82%BF%E3%82%A4%E3%83%A0%E3%82%A2%E3%82%A6%E3%83%88%E3%81%AF%E3%81%A9%E3%82%8C%E3%81%8F%E3%82%89%E3%81%84%E3%81%AE%E9%95%B7%E3%81%95%E3%81%A7%E3%81%99%E3%81%8B%EF%BC%9F/l968265546/>
/// <br>
/// # Errors
/// 下記の場合、ステータスコード401、エラーコード[Unauthorized]を返す<br>
/// <ul>
///   <li>セッション内に存在するログイン処理の状態が完了以外を示す場合</li>
///   <li>既にセッションの有効期限が切れている場合</li>
/// </ul>
async fn get_user_account_id_by_session_id(
    session_id: String,
    store: &impl SessionStore,
    op: impl RefreshOperation,
    expiry: Duration,
) -> Result<i64, ErrResp> {
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
                    code: Code::Unauthorized as u32,
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
    let login_status = match session.get::<String>(KEY_TO_LOGIN_STATUS) {
        Some(ls) => ls,
        None => {
            error!("failed to get login status from session");
            return Err(unexpected_err_resp());
        }
    };
    ensure_login_seq_has_already_finished(account_id, login_status)?;
    op.set_login_session_expiry(&mut session, expiry);
    // 新たなexpiryを設定したsessionをstoreに保存することでセッション期限を延長する
    let _ = store.store_session(session).await.map_err(|e| {
        error!("failed to store session: {}", e);
        unexpected_err_resp()
    })?;
    Ok(account_id)
}

fn ensure_login_seq_has_already_finished(
    account_id: i64,
    login_status: String,
) -> Result<(), ErrResp> {
    match LoginStatus::from(login_status) {
        LoginStatus::Finish => Ok(()),
        LoginStatus::NeedMoreVerification => {
            error!(
                "account_id ({}) has not finished login sequence yet",
                account_id
            );
            Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: Code::Unauthorized as u32,
                }),
            ))
        }
    }
}

trait RefreshOperation {
    fn set_login_session_expiry(&self, session: &mut Session, expiry: Duration);
}

struct RefreshOperationImpl {}

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
                code: Code::NotTermsOfUseAgreedYet as u32,
            }),
        )
    })?;
    Ok(())
}

async fn ensure_identity_exists(
    account_id: i64,
    op: &impl IdentityCheckOperation,
) -> Result<(), ErrResp> {
    let identity_exists = op.check_if_identity_exists(account_id).await?;
    if !identity_exists {
        error!("identity is not registered (account_id: {})", account_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoIdentityRegistered as u32,
            }),
        ));
    }
    Ok(())
}

pub(crate) async fn destroy_session_if_exists(
    session_id: &str,
    store: &impl SessionStore,
) -> Result<(), ErrResp> {
    let option_session = store
        .load_session(session_id.to_string())
        .await
        .map_err(|e| {
            error!("failed to load session: {}", e);
            unexpected_err_resp()
        })?;
    let session = match option_session {
        Some(s) => s,
        None => {
            info!("no session in session store");
            return Ok(());
        }
    };
    store.destroy_session(session).await.map_err(|e| {
        error!("failed to destroy session: {}", e);
        unexpected_err_resp()
    })?;
    Ok(())
}

/// テストコードで共通で使うコードをまとめるモジュール
#[cfg(test)]
pub(crate) mod tests {
    use async_session::{MemoryStore, Session, SessionStore};
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::ErrResp;

    use crate::{
        err::Code,
        util::{
            identity_check::IdentityCheckOperation,
            login_status::LoginStatus,
            session::{
                destroy_session_if_exists, get_user_account_id_by_session_id,
                KEY_TO_USER_ACCOUNT_ID, LOGIN_SESSION_EXPIRY,
            },
            terms_of_use::{TermsOfUseData, TermsOfUseLoadOperation},
        },
    };

    use super::{
        check_if_user_has_already_agreed, ensure_identity_exists, RefreshOperation,
        KEY_TO_LOGIN_STATUS,
    };

    /// 有効期限がないセッションを作成し、そのセッションにアクセスするためのセッションIDを返す
    pub(crate) async fn prepare_session(
        user_account_id: i64,
        login_status: LoginStatus,
        store: &impl SessionStore,
    ) -> String {
        let mut session = Session::new();
        // 実行環境（PCの性能）に依存させないように、テストコード内ではexpiryは設定しない
        session
            .insert(KEY_TO_USER_ACCOUNT_ID, user_account_id)
            .expect("failed to get Ok");
        session
            .insert(KEY_TO_LOGIN_STATUS, String::from(login_status))
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
    async fn get_user_account_id_by_session_id_success() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id = prepare_session(user_account_id, LoginStatus::Finish, &store).await;
        assert_eq!(1, store.count().await);

        let op = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };
        let result =
            get_user_account_id_by_session_id(session_id, &store, op, LOGIN_SESSION_EXPIRY)
                .await
                .expect("failed to get Ok");

        // get_user_account_id_by_session_idが何らかの副作用でセッションを破棄していないか確認
        // 補足説明:
        // 実際の運用ではセッションに有効期限をもたせるので、
        // get_user_account_id_by_session_idの後にセッションの有効期限が切れて0になることもあり得る。
        // しかし、テストケースではセッションに有効期限を持たせていないため、暗黙のうちに0になる心配はない。
        assert_eq!(1, store.count().await);
        assert_eq!(user_account_id, result);
    }

    #[tokio::test]
    async fn get_user_account_id_by_session_id_fail_login_seq_han_not_finished_yet() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id =
            prepare_session(user_account_id, LoginStatus::NeedMoreVerification, &store).await;
        assert_eq!(1, store.count().await);

        let op = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };
        let result =
            get_user_account_id_by_session_id(session_id, &store, op, LOGIN_SESSION_EXPIRY)
                .await
                .expect_err("failed to get Err");

        // get_user_account_id_by_session_idが何らかの副作用でセッションを破棄していないか確認
        // + ログイン処理がまだ途中である値を示している場合、エラーは返すがそのエラーに遭遇したことによりセッションを破棄するような処理はしない
        assert_eq!(1, store.count().await);
        assert_eq!(StatusCode::UNAUTHORIZED, result.0);
        assert_eq!(Code::Unauthorized as u32, result.1 .0.code);
    }

    #[tokio::test]
    async fn get_user_account_id_by_session_id_fail_session_already_expired() {
        let user_account_id = 10002;
        let store = MemoryStore::new();
        let session_id = prepare_session(user_account_id, LoginStatus::Finish, &store).await;
        // リクエストのプリプロセス前ににセッションを削除
        remove_session_from_store(&session_id, &store).await;
        assert_eq!(0, store.count().await);

        let op = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };
        let result =
            get_user_account_id_by_session_id(session_id, &store, op, LOGIN_SESSION_EXPIRY)
                .await
                .expect_err("failed to get Err");

        assert_eq!(0, store.count().await);
        assert_eq!(StatusCode::UNAUTHORIZED, result.0);
        assert_eq!(Code::Unauthorized as u32, result.1 .0.code);
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
        assert_eq!(Code::NotTermsOfUseAgreedYet as u32, result.1 .0.code);
    }

    struct IdentityCheckOperationMock {
        account_id: i64,
    }

    #[async_trait]
    impl IdentityCheckOperation for IdentityCheckOperationMock {
        async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
            if self.account_id != account_id {
                return Ok(false);
            }
            Ok(true)
        }
    }

    #[tokio::test]
    async fn ensure_identity_exists_success() {
        let account_id = 670;
        let op = IdentityCheckOperationMock { account_id };

        let result = ensure_identity_exists(account_id, &op).await;

        result.expect("failed to get Ok")
    }

    #[tokio::test]
    async fn ensure_identity_exists_fail_identity_is_not_registered() {
        let account_id = 670;
        let op = IdentityCheckOperationMock { account_id };
        let other_account_id = account_id + 51;

        let result = ensure_identity_exists(other_account_id, &op)
            .await
            .expect_err("failed to get Err");

        assert_eq!(StatusCode::BAD_REQUEST, result.0);
        assert_eq!(Code::NoIdentityRegistered as u32, result.1 .0.code);
    }

    #[tokio::test]
    async fn destroy_session_if_exists_destorys_session() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id = prepare_session(user_account_id, LoginStatus::Finish, &store).await;
        assert_eq!(1, store.count().await);

        destroy_session_if_exists(&session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
    }

    #[tokio::test]
    async fn destroy_session_if_exists_destorys_session_during_mfa_login_sequence() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id =
            prepare_session(user_account_id, LoginStatus::NeedMoreVerification, &store).await;
        assert_eq!(1, store.count().await);

        destroy_session_if_exists(&session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
    }

    #[tokio::test]
    async fn destroy_session_if_exists_returns_ok_if_no_session_exists() {
        let store = MemoryStore::new();
        // dummy session id
        let session_id = "KBvGQJJVyQquK5yuEcwlbfJfjNHBMAXIKRnHbVO/0QzBMHLak1xmqhaTbDuscJSeEPL2qwZfTP5BalDDMmR8eA==";
        assert_eq!(0, store.count().await);

        destroy_session_if_exists(session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
    }
}
