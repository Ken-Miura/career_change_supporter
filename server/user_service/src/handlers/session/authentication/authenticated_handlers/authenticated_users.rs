// Copyright 2023 Ken Miura

pub(crate) mod agreement_unchecked_user;
pub(crate) mod user;
pub(crate) mod verified_user;

use crate::handlers::session::authentication::user_operation::{
    get_user_info_if_available, FindUserInfoOperationImpl, UserInfo,
};
use crate::handlers::session::authentication::{
    get_authenticated_user_account_id, get_session_by_session_id, refresh_login_session,
};
use crate::handlers::session::authentication::{RefreshOperationImpl, LOGIN_SESSION_EXPIRY};
use async_session::SessionStore;
use axum::{async_trait, Json};
use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::SignedCookieJar;
use common::{ApiError, AppState, ErrResp};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use hyper::StatusCode;
use tracing::{error, info};

use crate::err::{unexpected_err_resp, Code};

use super::terms_of_use::{
    TermsOfUseLoadOperation, TermsOfUseLoadOperationImpl, TERMS_OF_USE_VERSION,
};

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

async fn get_agreement_unchecked_user_info_from_cookie(
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

    let session = get_session_by_session_id(&session_id, store).await?;
    let user_account_id = get_authenticated_user_account_id(&session).await?;
    let refresh_op = RefreshOperationImpl {};
    refresh_login_session(session, store, &refresh_op, LOGIN_SESSION_EXPIRY).await?;

    let find_user_op = FindUserInfoOperationImpl::new(pool);
    let user_info = get_user_info_if_available(user_account_id, &find_user_op).await?;

    Ok(user_info)
}

pub(in crate::handlers::session::authentication) async fn get_user_info_from_cookie(
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

async fn get_verified_user_info_from_cookie(
    option_cookie: Option<Cookie<'_>>,
    store: &impl SessionStore,
    pool: &DatabaseConnection,
) -> Result<UserInfo, ErrResp> {
    let user_info = get_user_info_from_cookie(option_cookie, store, pool).await?;

    let op = IdentityCheckOperationImpl::new(pool);
    ensure_identity_exists(user_info.account_id, &op).await?;

    Ok(user_info)
}

#[async_trait]
trait IdentityCheckOperation {
    /// Identityが存在するか確認する。存在する場合、trueを返す。そうでない場合、falseを返す。
    ///
    /// 個人情報の登録をしていないと使えないAPIに関して、処理を継続してよいか確認するために利用する。
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
}

struct IdentityCheckOperationImpl<'a> {
    pool: &'a DatabaseConnection,
}

impl<'a> IdentityCheckOperationImpl<'a> {
    fn new(pool: &'a DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> IdentityCheckOperation for IdentityCheckOperationImpl<'a> {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        let model = entity::identity::Entity::find_by_id(account_id)
            .one(self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find identity (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.is_some())
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

#[cfg(test)]
mod tests {

    use crate::handlers::session::authentication::authenticated_handlers::terms_of_use::TermsOfUseData;

    use super::*;

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
}
