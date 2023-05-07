// Copyright 2023 Ken Miura

use axum::{extract::State, http::StatusCode, Json};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::Deserialize;

use crate::handlers::session::authentication::authenticated_handlers::user_account_operation::{
    FindUserAccountInfoOperation, FindUserAccountInfoOperationImpl,
};

use super::{
    super::admin::Admin, validate_user_account_id_is_positive, UserAccount,
    UserAccountRetrievalResult,
};

pub(crate) async fn post_user_account_retrieval_by_user_account_id(
    Admin { admin_info: _ }: Admin,
    State(pool): State<DatabaseConnection>,
    Json(req): Json<UserAccountRetrievalByUserAccountIdReq>,
) -> RespResult<UserAccountRetrievalResult> {
    let op = FindUserAccountInfoOperationImpl::new(&pool);
    handle_user_account_retrieval_by_user_account_id(req.user_account_id, &op).await
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct UserAccountRetrievalByUserAccountIdReq {
    user_account_id: i64,
}

async fn handle_user_account_retrieval_by_user_account_id(
    user_account_id: i64,
    op: &impl FindUserAccountInfoOperation,
) -> RespResult<UserAccountRetrievalResult> {
    validate_user_account_id_is_positive(user_account_id)?;

    let result = op
        .find_user_account_info_by_account_id(user_account_id)
        .await?;
    let uarr = if let Some(ua) = result {
        UserAccountRetrievalResult {
            user_account: Some(UserAccount {
                user_account_id: ua.account_id,
                email_address: ua.email_address,
                last_login_time: ua.last_login_time.map(|m| m.to_rfc3339()),
                created_at: ua.created_at.to_rfc3339(),
                mfa_enabled_at: ua.mfa_enabled_at.map(|m| m.to_rfc3339()),
                disabled_at: ua.disabled_at.map(|m| m.to_rfc3339()),
            }),
        }
    } else {
        UserAccountRetrievalResult { user_account: None }
    };
    Ok((StatusCode::OK, Json(uarr)))
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use chrono::TimeZone;
    use common::{ErrResp, JAPANESE_TIME_ZONE};

    use crate::handlers::session::authentication::authenticated_handlers::user_account_operation::{FindUserAccountInfoOperation, UserAccountInfo};
    use crate::err::Code;

    use super::*;

    struct FindUserAccountInfoOperationMock {
        user_account_info: Option<UserAccountInfo>,
    }

    #[async_trait]
    impl FindUserAccountInfoOperation for FindUserAccountInfoOperationMock {
        async fn find_user_account_info_by_account_id(
            &self,
            account_id: i64,
        ) -> Result<Option<UserAccountInfo>, ErrResp> {
            if let Some(uai) = self.user_account_info.clone() {
                assert_eq!(uai.account_id, account_id)
            }
            Ok(self.user_account_info.clone())
        }

        async fn find_user_account_info_by_email_address(
            &self,
            _email_address: &str,
        ) -> Result<Option<UserAccountInfo>, ErrResp> {
            panic!("never called in this test cases");
        }
    }

    #[tokio::test]
    async fn handle_user_account_retrieval_by_user_account_id_success() {
        let user_account_id = 5151;
        let user_account = UserAccountInfo {
            account_id: user_account_id,
            email_address: "test@test.com".to_string(),
            last_login_time: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2023, 4, 7, 0, 1, 7)
                    .unwrap(),
            ),
            created_at: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 4, 5, 0, 1, 7)
                .unwrap(),
            mfa_enabled_at: None,
            disabled_at: None,
        };
        let op = FindUserAccountInfoOperationMock {
            user_account_info: Some(user_account.clone()),
        };

        let result = handle_user_account_retrieval_by_user_account_id(user_account_id, &op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(resp.0, StatusCode::OK);
        assert_eq!(
            resp.1 .0,
            UserAccountRetrievalResult {
                user_account: Some(UserAccount {
                    user_account_id: user_account.account_id,
                    email_address: user_account.email_address,
                    last_login_time: user_account.last_login_time.map(|m| m.to_rfc3339()),
                    created_at: user_account.created_at.to_rfc3339(),
                    mfa_enabled_at: user_account.mfa_enabled_at.map(|m| m.to_rfc3339()),
                    disabled_at: user_account.disabled_at.map(|m| m.to_rfc3339())
                })
            }
        )
    }

    #[tokio::test]
    async fn handle_user_account_retrieval_by_user_account_id_success_user_account_already_deleted()
    {
        let user_account_id = 5151;
        let op = FindUserAccountInfoOperationMock {
            user_account_info: None,
        };

        let result = handle_user_account_retrieval_by_user_account_id(user_account_id, &op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(resp.0, StatusCode::OK);
        assert_eq!(resp.1 .0, UserAccountRetrievalResult { user_account: None })
    }

    #[tokio::test]
    async fn handle_user_account_retrieval_by_user_account_id_fail_user_account_id_is_zero() {
        let user_account_id = 0;
        let op = FindUserAccountInfoOperationMock {
            user_account_info: None,
        };

        let result = handle_user_account_retrieval_by_user_account_id(user_account_id, &op).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::UserAccountIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn handle_user_account_retrieval_by_user_account_id_fail_user_account_id_is_negative() {
        let user_account_id = -1;
        let op = FindUserAccountInfoOperationMock {
            user_account_info: None,
        };

        let result = handle_user_account_retrieval_by_user_account_id(user_account_id, &op).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::UserAccountIdIsNotPositive as u32)
    }
}
