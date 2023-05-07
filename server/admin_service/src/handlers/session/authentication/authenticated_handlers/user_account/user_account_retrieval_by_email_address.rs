// Copyright 2023 Ken Miura

use axum::{extract::State, http::StatusCode, Json};
use common::{
    util::validator::email_address_validator::validate_email_address, ApiError, RespResult,
};
use entity::sea_orm::DatabaseConnection;
use serde::Deserialize;
use tracing::error;

use crate::handlers::session::authentication::authenticated_handlers::user_account_operation::{
    FindUserAccountInfoOperation, FindUserAccountInfoOperationImpl,
};

use super::{super::admin::Admin, UserAccount, UserAccountRetrievalResult};

pub(crate) async fn post_user_account_retrieval_by_email_address(
    Admin { admin_info: _ }: Admin,
    State(pool): State<DatabaseConnection>,
    Json(req): Json<UserAccountRetrievalByEmailAddressReq>,
) -> RespResult<UserAccountRetrievalResult> {
    let op = FindUserAccountInfoOperationImpl::new(&pool);
    handle_user_account_retrieval_by_email_address(&req.email_address, &op).await
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct UserAccountRetrievalByEmailAddressReq {
    email_address: String,
}

async fn handle_user_account_retrieval_by_email_address(
    email_address: &str,
    op: &impl FindUserAccountInfoOperation,
) -> RespResult<UserAccountRetrievalResult> {
    validate_email_address(email_address).map_err(|e| {
        error!(
            "failed to validate email address ({}): {}",
            email_address, e,
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: common::err::Code::InvalidEmailAddressFormat as u32,
            }),
        )
    })?;

    let result = op
        .find_user_account_info_by_email_address(email_address)
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

    // use super::*;

    // TODO
}
