// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::{http::StatusCode, Json};
use chrono::{DateTime, FixedOffset};
use common::{ApiError, ErrResp, ErrRespStruct};
use entity::sea_orm::{DatabaseConnection, DatabaseTransaction, EntityTrait, QuerySelect};
use serde::Deserialize;
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(super) struct UserInfo {
    pub(super) account_id: i64,
    pub(super) email_address: String,
    pub(super) mfa_enabled_at: Option<DateTime<FixedOffset>>,
    pub(super) disabled_at: Option<DateTime<FixedOffset>>,
}

#[async_trait]
pub(super) trait FindUserInfoOperation {
    async fn find_user_info_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<UserInfo>, ErrResp>;
}

pub(super) struct FindUserInfoOperationImpl<'a> {
    pool: &'a DatabaseConnection,
}

impl<'a> FindUserInfoOperationImpl<'a> {
    pub(super) fn new(pool: &'a DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> FindUserInfoOperation for FindUserInfoOperationImpl<'a> {
    async fn find_user_info_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Option<UserInfo>, ErrResp> {
        let model = entity::user_account::Entity::find_by_id(account_id)
            .one(self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find user_account (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| UserInfo {
            account_id: m.user_account_id,
            email_address: m.email_address,
            mfa_enabled_at: m.mfa_enabled_at,
            disabled_at: m.disabled_at,
        }))
    }
}

/// ユーザー情報を取得する
///
/// アカウントが存在しない場合、NoAccountFoundを返し、
/// アカウントが無効化されている場合、Unauthorizedを返す
pub(super) async fn get_user_info_if_available(
    account_id: i64,
    op: &impl FindUserInfoOperation,
) -> Result<UserInfo, ErrResp> {
    let user = op.find_user_info_by_account_id(account_id).await?;
    let user = user.ok_or_else(|| {
        error!("no account (account id: {}) found", account_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoAccountFound as u32,
            }),
        )
    })?;
    if user.disabled_at.is_some() {
        error!("account (account id: {}) is disabled", account_id);
        // セッションチェックの際に無効化を検出した際は、Unauthorizedを返すことでログイン画面へ遷移させる
        // ログイン画面でログインしようとした際に無効化を知らせるメッセージを表示
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiError {
                code: Code::Unauthorized as u32,
            }),
        ));
    }
    Ok(user)
}

pub(super) async fn find_user_account_by_user_account_id_with_exclusive_lock(
    txn: &DatabaseTransaction,
    user_account_id: i64,
) -> Result<Option<entity::user_account::Model>, ErrRespStruct> {
    let model = entity::user_account::Entity::find_by_id(user_account_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find user_account (user_account_id): {}): {}",
                user_account_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(model)
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::TimeZone;
    use common::{ErrResp, JAPANESE_TIME_ZONE};

    use crate::err::Code;

    use super::{get_user_info_if_available, FindUserInfoOperation, UserInfo};

    struct FindUserInfoOperationMock<'a> {
        user_info: &'a UserInfo,
    }

    #[async_trait]
    impl<'a> FindUserInfoOperation for FindUserInfoOperationMock<'a> {
        async fn find_user_info_by_account_id(
            &self,
            account_id: i64,
        ) -> Result<Option<UserInfo>, ErrResp> {
            if self.user_info.account_id != account_id {
                return Ok(None);
            }
            Ok(Some(self.user_info.clone()))
        }
    }

    #[tokio::test]
    async fn get_user_info_if_available_success() {
        let user_info = UserInfo {
            account_id: 2345,
            email_address: "test@test.com".to_string(),
            mfa_enabled_at: None,
            disabled_at: None,
        };
        let op_mock = FindUserInfoOperationMock {
            user_info: &user_info,
        };

        let result = get_user_info_if_available(user_info.account_id, &op_mock)
            .await
            .expect("failed to get Ok");

        assert_eq!(user_info, result);
    }

    #[tokio::test]
    async fn get_user_info_if_available_fail_no_account_found() {
        let user_info = UserInfo {
            account_id: 2345,
            email_address: "test@test.com".to_string(),
            mfa_enabled_at: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2021, 12, 31, 23, 59, 59)
                    .unwrap(),
            ),
            disabled_at: None,
        };
        let op_mock = FindUserInfoOperationMock {
            user_info: &user_info,
        };

        let other_account_id = user_info.account_id + 51051;
        let result = get_user_info_if_available(other_account_id, &op_mock)
            .await
            .expect_err("failed to get Err");

        assert_eq!(StatusCode::BAD_REQUEST, result.0);
        assert_eq!(Code::NoAccountFound as u32, result.1 .0.code);
    }

    #[tokio::test]
    async fn get_user_info_if_available_fail_account_disabled() {
        let user_info = UserInfo {
            account_id: 2345,
            email_address: "test@test.com".to_string(),
            mfa_enabled_at: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2021, 12, 31, 23, 59, 59)
                    .unwrap(),
            ),
            disabled_at: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2022, 1, 3, 23, 59, 59)
                    .unwrap(),
            ),
        };
        let op_mock = FindUserInfoOperationMock {
            user_info: &user_info,
        };

        let result = get_user_info_if_available(user_info.account_id, &op_mock)
            .await
            .expect_err("failed to get Err");

        assert_eq!(StatusCode::UNAUTHORIZED, result.0);
        assert_eq!(Code::Unauthorized as u32, result.1 .0.code);
    }
}
