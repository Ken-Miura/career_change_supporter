// Copyright 2021 Ken Miura

pub(crate) mod authentication;
pub(crate) mod password_change;

use async_session::{Session, SessionStore};
use common::ErrResp;
use tracing::{error, info};

use crate::err::unexpected_err_resp;

const SESSION_ID_COOKIE_NAME: &str = "session_id";

async fn find_session_by_session_id(
    session_id: &str,
    store: &impl SessionStore,
) -> Result<Option<Session>, ErrResp> {
    let option_session = store
        .load_session(session_id.to_string())
        .await
        .map_err(|e| {
            error!("failed to load session: {}", e);
            unexpected_err_resp()
        })?;
    Ok(option_session)
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

    use async_session::MemoryStore;

    use super::*;

    /// 有効期限がないセッションを作成し、そのセッションにアクセスするためのセッションIDを返す
    pub(super) async fn prepare_session(session: Session, store: &impl SessionStore) -> String {
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

    #[tokio::test]
    async fn find_session_by_session_id_finds_session() {
        let store = MemoryStore::new();
        let mut session = Session::new();
        session.insert("key", "value").expect("failed to get Ok");
        let cloned_session = session.clone();
        let session_id = prepare_session(session, &store).await;
        assert_eq!(1, store.count().await);

        let result = find_session_by_session_id(&session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(1, store.count().await);
        assert_eq!(cloned_session, result.expect("failed to get Ok"));
    }

    #[tokio::test]
    async fn find_session_by_session_id_no_session_found() {
        let store = MemoryStore::new();
        // dummy session id
        let session_id = "KBvGQJJVyQquK5yuEcwlbfJfjNHBMAXIKRnHbVO/0QzBMHLak1xmqhaTbDuscJSeEPL2qwZfTP5BalDDMmR8eA==";
        assert_eq!(0, store.count().await);

        let result = find_session_by_session_id(session_id, &store)
            .await
            .expect("failed to get Ok");

        assert_eq!(0, store.count().await);
        assert_eq!(None, result);
    }

    #[tokio::test]
    async fn destroy_session_if_exists_destorys_session() {
        let store = MemoryStore::new();
        let mut session = Session::new();
        session.insert("key", "value").expect("failed to get Ok");
        let session_id = prepare_session(session, &store).await;
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
