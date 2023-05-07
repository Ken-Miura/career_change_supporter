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
    use common::ErrResp;

    use crate::handlers::session::authentication::authenticated_handlers::user_account_operation::{FindUserAccountInfoOperation, UserAccountInfo};

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
    async fn handle_user_account_retrieval_by_user_account_id_success_none() {
        let user_account_id = 5151;
        let op = FindUserAccountInfoOperationMock {
            user_account_info: None,
        };

        let result = handle_user_account_retrieval_by_user_account_id(user_account_id, &op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(resp.0, StatusCode::OK);
        assert_eq!(resp.1 .0, UserAccountRetrievalResult { user_account: None })
    }
}
