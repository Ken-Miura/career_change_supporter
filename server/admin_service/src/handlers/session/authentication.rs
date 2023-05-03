// Copyright 2023 Ken Miura

pub(crate) mod authenticated_handlers;
pub(crate) mod login;
pub(crate) mod logout;

use std::time::Duration;

use async_session::{Session, SessionStore};
use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};
use tracing::{error, info};

use super::find_session_by_session_id;
use crate::err::{unexpected_err_resp, Code};

const KEY_TO_ADMIN_ACCOUNT_ID: &str = "admin_account_id";
const ADMIN_OPERATION_TIME: u64 = 15;

/// セッションの有効期限
const LOGIN_SESSION_EXPIRY: Duration = Duration::from_secs(60 * ADMIN_OPERATION_TIME);

/// session_idを使い、storeからセッションを取得する。<br>
///
/// # Errors
/// 下記の場合、ステータスコード401、エラーコード[Unauthorized]を返す<br>
/// <ul>
///   <li>session_idに対応するセッションがstoreに存在しない場合</li>
///   <li>既にセッションの有効期限が切れている場合</li>
/// </ul>
async fn get_session_by_session_id(
    session_id: &str,
    store: &impl SessionStore,
) -> Result<Session, ErrResp> {
    let option_session = find_session_by_session_id(session_id, store).await?;
    let session = match option_session {
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
    Ok(session)
}

/// セッションから管理者を一意に識別する値を取得する。<br>
/// <br>
/// # Errors
/// セッション内にアカウントIDがない場合、ステータスコード401、エラーコード[Unauthorized]を返す。
fn get_authenticated_admin_account_id(session: &Session) -> Result<i64, ErrResp> {
    let account_id = match session.get::<i64>(KEY_TO_ADMIN_ACCOUNT_ID) {
        Some(id) => id,
        None => {
            error!("failed to get account id from session");
            return Err(unexpected_err_resp());
        }
    };
    Ok(account_id)
}

/// セッションの有効期間をexpiryだけ延長する（※）<br>
/// <br>
/// （※）いわゆるアイドルタイムアウト。アブソリュートタイムアウトとリニューアルタイムアウトは実装しない。<br>
/// リニューアルタイムアウトが必要になった場合は、セッションを保存しているキャッシュシステムの定期再起動により実装する。<br>
/// 参考:<br>
///   セッションタイムアウトの種類: <https://cheatsheetseries.owasp.org/cheatsheets/Session_Management_Cheat_Sheet.html#automatic-session-expiration><br>
///   著名なフレームワークにおいてもアイドルタイムアウトが一般的で、アブソリュートタイムアウトは実装されていない<br>
///     1. <https://stackoverflow.com/questions/62964012/how-to-set-absolute-session-timeout-for-a-spring-session><br>
///     2. <https://www.webdevqa.jp.net/ja/authentication/%E3%82%BB%E3%83%83%E3%82%B7%E3%83%A7%E3%83%B3%E3%81%AE%E7%B5%B6%E5%AF%BE%E3%82%BF%E3%82%A4%E3%83%A0%E3%82%A2%E3%82%A6%E3%83%88%E3%81%AF%E3%81%A9%E3%82%8C%E3%81%8F%E3%82%89%E3%81%84%E3%81%AE%E9%95%B7%E3%81%95%E3%81%A7%E3%81%99%E3%81%8B%EF%BC%9F/l968265546/><br>
async fn refresh_login_session(
    mut session: Session,
    store: &impl SessionStore,
    op: &impl RefreshOperation,
    expiry: Duration,
) -> Result<(), ErrResp> {
    op.set_login_session_expiry(&mut session, expiry);
    // 新たなexpiryを設定したsessionをstoreに保存することでセッション期限を延長する
    let _ = store.store_session(session).await.map_err(|e| {
        error!("failed to store session: {}", e);
        unexpected_err_resp()
    })?;
    Ok(())
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

#[cfg(test)]
pub(super) mod tests {
    use async_session::{MemoryStore, Session, SessionStore};
    use axum::http::StatusCode;

    use crate::{
        err,
        handlers::session::tests::{prepare_session, remove_session_from_store},
    };

    use super::*;

    /// 有効期限がないセッションを作成し、そのセッションにアクセスするためのセッションIDを返す
    pub(super) async fn prepare_login_session(
        admin_account_id: i64,
        store: &impl SessionStore,
    ) -> String {
        let mut session = Session::new();
        // 実行環境（PCの性能）に依存させないように、テストコード内ではexpiryは設定しない
        session
            .insert(KEY_TO_ADMIN_ACCOUNT_ID, admin_account_id)
            .expect("failed to get Ok");
        prepare_session(session, store).await
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
    async fn get_session_by_session_id_and_get_authenticated_admin_account_id_success() {
        let store = MemoryStore::new();
        let admin_account_id = 15001;
        let session_id = prepare_login_session(admin_account_id, &store).await;
        assert_eq!(1, store.count().await);

        let session = get_session_by_session_id(&session_id, &store)
            .await
            .expect("failed to get Ok");
        let result = get_authenticated_admin_account_id(&session).expect("failed to get Ok");
        let op = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };
        refresh_login_session(session, &store, &op, LOGIN_SESSION_EXPIRY)
            .await
            .expect("failed to get Ok");

        // 何らかの副作用でセッションを破棄していないか確認
        // 補足説明:
        // 実際の運用ではセッションに有効期限をもたせるので、後にセッションの有効期限が切れて0になることもあり得る。
        // しかし、テストケースではセッションに有効期限を持たせていないため、暗黙のうちに0になる心配はない。
        assert_eq!(1, store.count().await);
        assert_eq!(admin_account_id, result);
    }

    #[tokio::test]
    async fn get_session_by_session_id_fail_session_already_expired() {
        let admin_account_id = 10002;
        let store = MemoryStore::new();
        let session_id = prepare_login_session(admin_account_id, &store).await;
        // リクエストのプリプロセス前ににセッションを削除
        remove_session_from_store(&session_id, &store).await;
        assert_eq!(0, store.count().await);

        let result = get_session_by_session_id(&session_id, &store)
            .await
            .expect_err("failed to get Err");

        assert_eq!(0, store.count().await);
        assert_eq!(StatusCode::UNAUTHORIZED, result.0);
        assert_eq!(err::Code::Unauthorized as u32, result.1 .0.code);
    }
}
