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

/// session_idを使いstoreから、管理者のアカウントIDを取得する。<br>
/// 管理者のアカウントIDを取得するには、セッションが有効な期間中に呼び出す必要がある。<br>
/// 管理者のアカウントIDの取得に成功した場合、セッションの有効期間がexpiryだけ延長される（※）<br>
/// <br>
/// （※）いわゆるアイドルタイムアウト。アブソリュートタイムアウトとリニューアルタイムアウトは実装しない。<br>
/// # NOTE
/// Adminを利用するときは、原則としてハンドラのパラメータにAdminを指定する方法を選択する<br>
/// <br>
/// # Errors
/// 下記の場合、ステータスコード401、エラーコード[Unauthorized]を返す<br>
/// <ul>
///   <li>既にセッションの有効期限が切れている場合</li>
/// </ul>
async fn get_admin_account_id_by_session_id(
    session_id: String,
    store: &impl SessionStore,
    op: impl RefreshOperation,
    expiry: Duration,
) -> Result<i64, ErrResp> {
    let option_session = find_session_by_session_id(&session_id, store).await?;
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
    let admin_account_id = match session.get::<i64>(KEY_TO_ADMIN_ACCOUNT_ID) {
        Some(id) => id,
        None => {
            error!("failed to get admin account id from session");
            return Err(unexpected_err_resp());
        }
    };
    op.set_login_session_expiry(&mut session, expiry);
    // 新たなexpiryを設定したsessionをstoreに保存することでセッション期限を延長する
    let _ = store.store_session(session).await.map_err(|e| {
        error!("failed to store session: {}", e);
        unexpected_err_resp()
    })?;
    Ok(admin_account_id)
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
    async fn get_admin_account_id_by_session_id_success() {
        let store = MemoryStore::new();
        let admin_account_id = 15001;
        let session_id = prepare_login_session(admin_account_id, &store).await;
        assert_eq!(1, store.count().await);

        let op = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };
        let result =
            get_admin_account_id_by_session_id(session_id, &store, op, LOGIN_SESSION_EXPIRY)
                .await
                .expect("failed to get Ok");

        // get_admin_account_id_by_session_idが何らかの副作用でセッションを破棄していないか確認
        // 補足説明:
        // 実際の運用ではセッションに有効期限をもたせるので、
        // get_admin_account_id_by_session_idの後にセッションの有効期限が切れて0になることもあり得る。
        // しかし、テストケースではセッションに有効期限を持たせていないため、暗黙のうちに0になる心配はない。
        assert_eq!(1, store.count().await);
        assert_eq!(admin_account_id, result);
    }

    #[tokio::test]
    async fn get_admin_account_id_by_session_id_fail_session_already_expired() {
        let admin_account_id = 10002;
        let store = MemoryStore::new();
        let session_id = prepare_login_session(admin_account_id, &store).await;
        // リクエストのプリプロセス前ににセッションを削除
        remove_session_from_store(&session_id, &store).await;
        assert_eq!(0, store.count().await);

        let op = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };
        let result =
            get_admin_account_id_by_session_id(session_id, &store, op, LOGIN_SESSION_EXPIRY)
                .await
                .expect_err("failed to get Err");

        assert_eq!(0, store.count().await);
        assert_eq!(StatusCode::UNAUTHORIZED, result.0);
        assert_eq!(err::Code::Unauthorized as u32, result.1 .0.code);
    }
}
