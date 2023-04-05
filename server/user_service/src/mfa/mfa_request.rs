// Copyright 2023 Ken Miura

pub(crate) mod pass_code;
pub(crate) mod recovery_code;

use async_session::{Session, SessionStore};
use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use tracing::error;

use crate::{
    err::{unexpected_err_resp, Code},
    util::{
        login_status::LoginStatus,
        session::{KEY_TO_LOGIN_STATUS, KEY_TO_USER_ACCOUNT_ID},
    },
};

struct MfaInfo {
    base32_encoded_secret: String,
    hashed_recovery_code: Vec<u8>,
}

async fn get_mfa_info_by_account_id(
    account_id: i64,
    pool: &DatabaseConnection,
) -> Result<MfaInfo, ErrResp> {
    let result = entity::mfa_info::Entity::find_by_id(account_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find mfa_info (user_account_id: {}): {}",
                account_id, e
            );
            unexpected_err_resp()
        })?;
    let mi = result.ok_or_else(|| {
        error!("no mfa_info (user_account_id: {}) found", account_id);
        unexpected_err_resp()
    })?;
    Ok(MfaInfo {
        base32_encoded_secret: mi.base32_encoded_secret,
        hashed_recovery_code: mi.hashed_recovery_code,
    })
}

async fn get_session_by_session_id(
    session_id: &str,
    store: &impl SessionStore,
) -> Result<Session, ErrResp> {
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
            error!("no session found");
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

fn get_account_id_from_session(session: &Session) -> Result<i64, ErrResp> {
    let account_id = match session.get::<i64>(KEY_TO_USER_ACCOUNT_ID) {
        Some(id) => id,
        None => {
            error!("failed to get account id from session");
            return Err(unexpected_err_resp());
        }
    };
    Ok(account_id)
}

fn update_login_status(session: &mut Session, ls: LoginStatus) -> Result<(), ErrResp> {
    session
        .insert(KEY_TO_LOGIN_STATUS, ls.clone())
        .map_err(|e| {
            error!(
                "failed to insert login_status ({}) into session: {}",
                String::from(ls),
                e
            );
            unexpected_err_resp()
        })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use async_session::MemoryStore;

    use crate::{
        mfa::mfa_request::get_session_by_session_id,
        util::{
            login_status::LoginStatus,
            session::{tests::prepare_session, KEY_TO_LOGIN_STATUS, KEY_TO_USER_ACCOUNT_ID},
        },
    };

    #[tokio::test]
    async fn get_session_by_session_id_success() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id =
            prepare_session(user_account_id, LoginStatus::NeedMoreVerification, &store).await;
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
}
