// Copyright 2023 Ken Miura

pub(crate) mod pass_code;
pub(crate) mod recovery_code;

use std::env;

use axum::{http::StatusCode, Json};
use chrono::{DateTime, FixedOffset};
use common::{mfa::check_if_pass_code_matches, ApiError, ErrResp, ErrRespStruct};
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionError, TransactionTrait,
};
use once_cell::sync::Lazy;
use tracing::error;

use crate::{
    err::{unexpected_err_resp, Code},
    handlers::session::LoginStatus,
};

use async_session::{Session, SessionStore};
use axum_extra::extract::cookie::Cookie;

use crate::handlers::session::{KEY_TO_LOGIN_STATUS, KEY_TO_USER_ACCOUNT_ID};

use super::user_operation::find_user_account_by_user_account_id_with_exclusive_lock;

pub(crate) const KEY_TO_USER_TOTP_ISSUER: &str = "USER_TOTP_ISSUER";
pub(super) static USER_TOTP_ISSUER: Lazy<String> = Lazy::new(|| {
    let issuer = env::var(KEY_TO_USER_TOTP_ISSUER).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_USER_TOTP_ISSUER
        )
    });
    if issuer.contains(':') {
        panic!("USER_TOTP_ISSUER must not contain \":\": {}", issuer);
    }
    issuer
});

pub(super) fn ensure_mfa_is_enabled(mfa_enabled: bool) -> Result<(), ErrResp> {
    if !mfa_enabled {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::MfaIsNotEnabled as u32,
            }),
        ));
    }
    Ok(())
}

pub(super) fn verify_pass_code(
    account_id: i64,
    base32_encoded_secret: &str,
    issuer: &str,
    current_date_time: &DateTime<FixedOffset>,
    pass_code: &str,
) -> Result<(), ErrResp> {
    let matched = check_if_pass_code_matches(
        account_id,
        base32_encoded_secret,
        issuer,
        current_date_time,
        pass_code,
    )?;
    if !matched {
        error!(
            "failed to check pass code (account_id: {}, current_date_time: {})",
            account_id, current_date_time
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PassCodeDoesNotMatch as u32,
            }),
        ));
    }
    Ok(())
}

pub(super) async fn disable_mfa(account_id: i64, pool: &DatabaseConnection) -> Result<(), ErrResp> {
    pool.transaction::<_, (), ErrRespStruct>(|txn| {
        Box::pin(async move {
            let user_model =
                find_user_account_by_user_account_id_with_exclusive_lock(txn, account_id).await?;
            let user_model = user_model.ok_or_else(|| {
                error!(
                    "failed to find user_account (user_account_id: {})",
                    account_id
                );
                ErrRespStruct {
                    err_resp: unexpected_err_resp(),
                }
            })?;

            let _ = entity::mfa_info::Entity::delete_by_id(account_id)
                .exec(txn)
                .await
                .map_err(|e| {
                    error!(
                        "failed to delete mfa_info (user_account_id: {}): {}",
                        account_id, e
                    );
                    ErrRespStruct {
                        err_resp: unexpected_err_resp(),
                    }
                })?;

            let mut user_active_model: entity::user_account::ActiveModel = user_model.into();
            user_active_model.mfa_enabled_at = Set(None);
            let _ = user_active_model.update(txn).await.map_err(|e| {
                error!(
                    "failed to update mfa_enabled_at in user_account (user_account_id: {}): {}",
                    account_id, e
                );
                ErrRespStruct {
                    err_resp: unexpected_err_resp(),
                }
            })?;

            Ok(())
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(db_err) => {
            error!("connection error: {}", db_err);
            unexpected_err_resp()
        }
        TransactionError::Transaction(err_resp_struct) => {
            error!("failed to disable_mfa: {}", err_resp_struct);
            err_resp_struct.err_resp
        }
    })?;
    Ok(())
}

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

    use async_session::MemoryStore;
    use chrono::TimeZone;
    use common::JAPANESE_TIME_ZONE;

    use crate::handlers::session::{
        tests::{prepare_session, remove_session_from_store},
        SESSION_ID_COOKIE_NAME,
    };

    use super::*;

    #[test]
    fn ensure_mfa_is_enabled_success() {
        let mfa_enabled = true;
        ensure_mfa_is_enabled(mfa_enabled).expect("failed to get Ok");
    }

    #[test]
    fn ensure_mfa_is_enabled_error() {
        let mfa_enabled = false;
        let result = ensure_mfa_is_enabled(mfa_enabled).expect_err("failed to get Err");

        assert_eq!(StatusCode::BAD_REQUEST, result.0);
        assert_eq!(Code::MfaIsNotEnabled as u32, result.1.code);
    }

    #[test]
    fn verify_pass_code_success() {
        let account_id = 413;
        let base32_encoded_secret = "NKQHIV55R4LJV3MD6YSC4Z4UCMT3NDYD";
        let issuer = "Issuer";
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 4, 5, 0, 1, 7)
            .unwrap();
        // 上記のbase32_encoded_secretとcurrent_date_timeでGoogle Authenticatorが実際に算出した値
        let pass_code = "540940";

        verify_pass_code(
            account_id,
            base32_encoded_secret,
            issuer,
            &current_date_time,
            pass_code,
        )
        .expect("failed to get Ok");
    }

    #[test]
    fn verify_pass_code_success_one_step_after() {
        let account_id = 413;
        let base32_encoded_secret = "NKQHIV55R4LJV3MD6YSC4Z4UCMT3NDYD";
        let issuer = "Issuer";
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(
                2023,
                4,
                5,
                0,
                1,
                7 + 30, /* 30s = 本サービスにおける1ステップ */
            )
            .unwrap();
        // 上記のbase32_encoded_secretとcurrent_date_timeでGoogle Authenticatorが実際に算出した値
        let pass_code = "540940";

        verify_pass_code(
            account_id,
            base32_encoded_secret,
            issuer,
            &current_date_time,
            pass_code,
        )
        .expect("failed to get Ok");
    }

    #[test]
    fn verify_pass_code_fail_two_step_after() {
        let account_id = 413;
        let base32_encoded_secret = "NKQHIV55R4LJV3MD6YSC4Z4UCMT3NDYD";
        let issuer = "Issuer";
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(
                2023, 4, 5, 0,
                2, /* 30s = 本サービスにおける1ステップ。2ステップ = 1m */
                7,
            )
            .unwrap();
        // 上記のbase32_encoded_secretとcurrent_date_timeでGoogle Authenticatorが実際に算出した値
        let pass_code = "540940";

        let result = verify_pass_code(
            account_id,
            base32_encoded_secret,
            issuer,
            &current_date_time,
            pass_code,
        )
        .expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1.code, Code::PassCodeDoesNotMatch as u32);
    }

    #[test]
    fn verify_pass_code_fail_incorrect_pass_code() {
        let account_id = 413;
        let base32_encoded_secret = "NKQHIV55R4LJV3MD6YSC4Z4UCMT3NDYD";
        let issuer = "Issuer";
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 4, 5, 0, 1, 7)
            .unwrap();
        let pass_code = "123456";

        let result = verify_pass_code(
            account_id,
            base32_encoded_secret,
            issuer,
            &current_date_time,
            pass_code,
        )
        .expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1.code, Code::PassCodeDoesNotMatch as u32);
    }

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
