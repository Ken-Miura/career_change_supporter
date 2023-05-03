// Copyright 2021 Ken Miura

pub(crate) mod authentication;

use async_session::{Session, SessionStore};
use common::ErrResp;
use tracing::error;

use crate::err::unexpected_err_resp;

const ADMIN_SESSION_ID_COOKIE_NAME: &str = "admin_session_id";

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

#[cfg(test)]
pub(super) mod tests {

    use async_session::MemoryStore;

    use super::*;

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
}
