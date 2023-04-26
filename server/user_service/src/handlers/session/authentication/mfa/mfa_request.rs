// Copyright 2023 Ken Miura

pub(crate) mod pass_code;
pub(crate) mod recovery_code;

use async_session::{Session, SessionStore};
use axum::{http::StatusCode, Json};
use axum_extra::extract::cookie::Cookie;
use common::{ApiError, ErrResp};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use tracing::error;

use crate::{
    err::{unexpected_err_resp, Code},
    handlers::session::{KEY_TO_LOGIN_STATUS, KEY_TO_USER_ACCOUNT_ID},
    util::login_status::LoginStatus,
};

fn extract_session_id_from_cookie(cookie: Option<Cookie>) -> Result<String, ErrResp> {
    let session_id = match cookie {
        Some(s) => s.value().to_string(),
        None => {
            error!("no sessoin cookie found");
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(ApiError {
                    code: Code::Unauthorized as u32,
                }),
            ));
        }
    };
    Ok(session_id)
}

#[derive(Clone, Debug)]
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
    use async_session::{MemoryStore, Session};
    use axum::http::StatusCode;
    use axum_extra::extract::cookie::Cookie;

    use crate::{
        err::Code,
        handlers::session::{
            authentication::mfa::mfa_request::{get_session_by_session_id, update_login_status},
            tests::{prepare_session, remove_session_from_store},
            KEY_TO_LOGIN_STATUS, KEY_TO_USER_ACCOUNT_ID, SESSION_ID_COOKIE_NAME,
        },
        util::login_status::LoginStatus,
    };

    use super::{extract_session_id_from_cookie, get_account_id_from_session};

    #[test]
    fn extract_session_id_from_cookie_success() {
        let value = "4d/UQZs+7mY0kF16rdf8qb07y2TzyHM2LCooSqBJB4GuF5LHw8h5jFLoJmbR3wYbwpy9bGQB2DExLM4lxvD62A==";
        let cookie = Cookie::build(SESSION_ID_COOKIE_NAME, value).finish();

        let result = extract_session_id_from_cookie(Some(cookie)).expect("failed to get Ok");

        assert_eq!(result, value);
    }

    #[test]
    fn extract_session_id_from_cookie_fail() {
        let result = extract_session_id_from_cookie(None).expect_err("failed to get Err");
        assert_eq!(result.0, StatusCode::UNAUTHORIZED);
        assert_eq!(result.1.code, Code::Unauthorized as u32);
    }

    #[tokio::test]
    async fn get_session_by_session_id_success1() {
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

    #[tokio::test]
    async fn get_session_by_session_id_success2() {
        let store = MemoryStore::new();
        let user_account_id = 15001;
        let session_id = prepare_session(user_account_id, LoginStatus::Finish, &store).await;
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
            prepare_session(user_account_id, LoginStatus::NeedMoreVerification, &store).await;
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

    #[test]
    fn get_account_id_from_session_success() {
        let user_account_id = 5115;
        let login_status = LoginStatus::NeedMoreVerification;
        let mut session = Session::new();
        // 実行環境（PCの性能）に依存させないように、テストコード内ではexpiryは設定しない
        session
            .insert(KEY_TO_USER_ACCOUNT_ID, user_account_id)
            .expect("failed to get Ok");
        session
            .insert(KEY_TO_LOGIN_STATUS, String::from(login_status))
            .expect("failed to get Ok");

        let result = get_account_id_from_session(&session).expect("failed to get Ok");

        assert_eq!(result, user_account_id);
    }

    #[test]
    fn update_login_status_success() {
        let user_account_id = 5115;
        let mut session = Session::new();
        // 実行環境（PCの性能）に依存させないように、テストコード内ではexpiryは設定しない
        session
            .insert(KEY_TO_USER_ACCOUNT_ID, user_account_id)
            .expect("failed to get Ok");
        session
            .insert(
                KEY_TO_LOGIN_STATUS,
                String::from(LoginStatus::NeedMoreVerification),
            )
            .expect("failed to get Ok");

        update_login_status(&mut session, LoginStatus::Finish).expect("failed to get Ok");

        let result1 = session
            .get::<i64>(KEY_TO_USER_ACCOUNT_ID)
            .expect("failed to get Ok");
        let result2 = session
            .get::<String>(KEY_TO_LOGIN_STATUS)
            .expect("failed to get Ok");
        assert_eq!(result1, user_account_id);
        assert_eq!(result2, String::from(LoginStatus::Finish));
    }
}
