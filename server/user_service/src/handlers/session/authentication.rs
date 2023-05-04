// Copyright 2023 Ken Miura

pub(crate) mod authenticated_handlers;
pub(crate) mod login;
pub(crate) mod logout;
pub(crate) mod mfa;
mod user_operation;

use async_session::{Session, SessionStore};
use axum::{http::StatusCode, Json};
use chrono::{DateTime, FixedOffset};
use common::ApiError;
use common::ErrResp;
use entity::sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde::Serialize;
use std::time::Duration;
use tracing::{error, info};

use crate::err::unexpected_err_resp;
use crate::err::Code;

use super::find_session_by_session_id;

const KEY_TO_USER_ACCOUNT_ID: &str = "user_account_id";
const KEY_TO_LOGIN_STATUS: &str = "login_status";

const LOGIN_STATUS_FINISH: &str = "Finish";
const LOGIN_STATUS_NEED_MORE_VERIFICATION: &str = "NeedMoreVerification";

const LENGTH_OF_MEETING_IN_MINUTE: u64 = 60;
const TIME_FOR_SUBSEQUENT_OPERATIONS: u64 = 10;
/// セッションの有効期限
const LOGIN_SESSION_EXPIRY: Duration =
    Duration::from_secs(60 * (LENGTH_OF_MEETING_IN_MINUTE + TIME_FOR_SUBSEQUENT_OPERATIONS));

#[derive(Serialize, Debug, Clone, PartialEq)]
enum LoginStatus {
    Finish,
    NeedMoreVerification,
}

impl From<String> for LoginStatus {
    fn from(ls: String) -> Self {
        if ls == LOGIN_STATUS_FINISH {
            LoginStatus::Finish
        } else if ls == LOGIN_STATUS_NEED_MORE_VERIFICATION {
            LoginStatus::NeedMoreVerification
        } else {
            panic!("never reach here!")
        }
    }
}

impl From<LoginStatus> for String {
    fn from(ls: LoginStatus) -> Self {
        match ls {
            LoginStatus::Finish => LOGIN_STATUS_FINISH.to_string(),
            LoginStatus::NeedMoreVerification => LOGIN_STATUS_NEED_MORE_VERIFICATION.to_string(),
        }
    }
}

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

/// セッションからユーザーを一意に識別する値を取得する。<br>
/// <br>
/// # Errors
/// セッション内に存在するログイン処理の状態が完了（認証済み）以外を示す場合、ステータスコード401、エラーコード[Unauthorized]を返す。
fn get_authenticated_user_account_id(session: &Session) -> Result<i64, ErrResp> {
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

async fn update_last_login(
    account_id: i64,
    login_time: &DateTime<FixedOffset>,
    pool: &DatabaseConnection,
) -> Result<(), ErrResp> {
    let user_account_model = entity::user_account::ActiveModel {
        user_account_id: Set(account_id),
        last_login_time: Set(Some(*login_time)),
        ..Default::default()
    };
    let _ = user_account_model.update(pool).await.map_err(|e| {
        error!(
            "failed to update user_account (user_account_id: {}): {}",
            account_id, e
        );
        unexpected_err_resp()
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use crate::handlers::session::{
        destroy_session_if_exists,
        tests::{prepare_session, remove_session_from_store},
    };
    use async_session::MemoryStore;

    use super::*;

    /// 有効期限がないセッションを作成し、そのセッションにアクセスするためのセッションIDを返す
    pub(super) async fn prepare_login_session(
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
    async fn get_session_by_session_id_success1() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id =
            prepare_login_session(user_account_id, LoginStatus::NeedMoreVerification, &store).await;
        assert_eq!(1, store.count().await);

        let result = get_session_by_session_id(&session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(1, store.count().await);
        assert_eq!(
            user_account_id,
            result
                .get::<i64>(KEY_TO_USER_ACCOUNT_ID)
                .expect("failed to get Ok")
        );
        assert_eq!(
            String::from(LoginStatus::NeedMoreVerification),
            result
                .get::<String>(KEY_TO_LOGIN_STATUS)
                .expect("failed to get Ok")
        );
    }

    #[tokio::test]
    async fn get_session_by_session_id_success2() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id = prepare_login_session(user_account_id, LoginStatus::Finish, &store).await;
        assert_eq!(1, store.count().await);

        let result = get_session_by_session_id(&session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(1, store.count().await);
        assert_eq!(
            user_account_id,
            result
                .get::<i64>(KEY_TO_USER_ACCOUNT_ID)
                .expect("failed to get Ok")
        );
        assert_eq!(
            String::from(LoginStatus::Finish),
            result
                .get::<String>(KEY_TO_LOGIN_STATUS)
                .expect("failed to get Ok")
        );
    }

    #[tokio::test]
    async fn get_session_by_session_id_fail() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id =
            prepare_login_session(user_account_id, LoginStatus::NeedMoreVerification, &store).await;
        // リクエストのプリプロセス前ににセッションを削除
        remove_session_from_store(&session_id, &store).await;
        assert_eq!(0, store.count().await);

        let result = get_session_by_session_id(&session_id, &store)
            .await
            .expect_err("failed to get Err");

        assert_eq!(0, store.count().await);
        assert_eq!(StatusCode::UNAUTHORIZED, result.0);
        assert_eq!(Code::Unauthorized as u32, result.1 .0.code);
    }

    #[tokio::test]
    async fn get_session_by_session_id_and_get_authenticated_user_account_id_success() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id = prepare_login_session(user_account_id, LoginStatus::Finish, &store).await;
        assert_eq!(1, store.count().await);

        let session = get_session_by_session_id(&session_id, &store)
            .await
            .expect("failed to get Ok");
        let result = get_authenticated_user_account_id(&session).expect("failed to get Ok");
        let op = RefreshOperationMock {
            expiry: LOGIN_SESSION_EXPIRY,
        };
        refresh_login_session(session, &store, &op, LOGIN_SESSION_EXPIRY)
            .await
            .expect("failed to get Ok");

        // 各関数が何らかの副作用でセッションを破棄していないか確認
        // 補足説明:
        // 実際の運用ではセッションに有効期限をもたせるので、
        // 各関数の後にセッションの有効期限が切れて0になることもあり得る。
        // しかし、テストケースではセッションに有効期限を持たせていないため、暗黙のうちに0になる心配はない。
        assert_eq!(1, store.count().await);
        assert_eq!(user_account_id, result);
    }

    #[tokio::test]
    async fn get_session_by_session_id_and_get_authenticated_user_account_id_fail_login_seq_han_not_finished_yet(
    ) {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id =
            prepare_login_session(user_account_id, LoginStatus::NeedMoreVerification, &store).await;
        assert_eq!(1, store.count().await);

        let session = get_session_by_session_id(&session_id, &store)
            .await
            .expect("failed to get Ok");
        let result = get_authenticated_user_account_id(&session).expect_err("failed to get Err");

        // 各関数が何らかの副作用でセッションを破棄していないか確認
        // + ログイン処理がまだ途中である値を示している場合、エラーは返すがそのエラーに遭遇したことによりセッションを破棄するような処理はしない
        assert_eq!(1, store.count().await);
        assert_eq!(StatusCode::UNAUTHORIZED, result.0);
        assert_eq!(Code::Unauthorized as u32, result.1 .0.code);
    }

    #[tokio::test]
    async fn get_session_by_session_id_fail_session_already_expired() {
        let user_account_id = 10002;
        let store = MemoryStore::new();
        let session_id = prepare_login_session(user_account_id, LoginStatus::Finish, &store).await;
        // リクエストのプリプロセス前ににセッションを削除
        remove_session_from_store(&session_id, &store).await;
        assert_eq!(0, store.count().await);

        let result = get_session_by_session_id(&session_id, &store)
            .await
            .expect_err("failed to get Err");

        assert_eq!(0, store.count().await);
        assert_eq!(StatusCode::UNAUTHORIZED, result.0);
        assert_eq!(Code::Unauthorized as u32, result.1 .0.code);
    }

    #[tokio::test]
    async fn destroy_login_session_if_exists_destorys_session() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id = prepare_login_session(user_account_id, LoginStatus::Finish, &store).await;
        assert_eq!(1, store.count().await);

        destroy_session_if_exists(&session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
    }

    #[tokio::test]
    async fn destroy_login_session_if_exists_destorys_session_during_mfa_login_sequence() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id =
            prepare_login_session(user_account_id, LoginStatus::NeedMoreVerification, &store).await;
        assert_eq!(1, store.count().await);

        destroy_session_if_exists(&session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
    }
}
