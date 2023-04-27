// Copyright 2021 Ken Miura

pub(crate) mod authentication;
pub(crate) mod password_change;

use async_session::{Session, SessionStore};
use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};
use std::time::Duration;
use tracing::{error, info};

use crate::err::unexpected_err_resp;
use crate::err::Code;
use crate::util::login_status::LoginStatus;

const SESSION_ID_COOKIE_NAME: &str = "session_id";
const KEY_TO_USER_ACCOUNT_ID: &str = "user_account_id";
const KEY_TO_LOGIN_STATUS: &str = "login_status";

const LENGTH_OF_MEETING_IN_MINUTE: u64 = 60;
const TIME_FOR_SUBSEQUENT_OPERATIONS: u64 = 10;

/// セッションの有効期限
const LOGIN_SESSION_EXPIRY: Duration =
    Duration::from_secs(60 * (LENGTH_OF_MEETING_IN_MINUTE + TIME_FOR_SUBSEQUENT_OPERATIONS));

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

async fn destroy_session_if_exists(
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

#[cfg(test)]
mod tests {
    use async_session::{MemoryStore, Session, SessionStore};
    use axum::http::StatusCode;

    use crate::{
        err::Code,
        handlers::session::{
            destroy_session_if_exists, get_user_account_id_by_session_id, LOGIN_SESSION_EXPIRY,
        },
        util::login_status::LoginStatus,
    };

    use super::{RefreshOperation, KEY_TO_LOGIN_STATUS, KEY_TO_USER_ACCOUNT_ID};

    /// 有効期限がないセッションを作成し、そのセッションにアクセスするためのセッションIDを返す
    pub(super) async fn prepare_session(
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

    pub(super) async fn remove_session_from_store(session_id: &str, store: &impl SessionStore) {
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
