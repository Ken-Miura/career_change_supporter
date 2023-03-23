// Copyright 2021 Ken Miura

use common::ErrResp;

use super::user_info::FindUserInfoOperation;

/// コンサルタントのアカウントが利用可能か確認する。
/// アカウントが存在し、かつ無効化されていない（=利用可能な）場合、trueを返す。そうでない場合、falseを返す。
pub(crate) async fn check_if_consultant_is_available(
    account_id: i64,
    op: &impl FindUserInfoOperation,
) -> Result<bool, ErrResp> {
    let user_info = op.find_user_info_by_account_id(account_id).await?;
    if let Some(u) = user_info {
        if u.disabled_at.is_some() {
            Ok(false)
        } else {
            Ok(true)
        }
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {

    use axum::async_trait;
    use chrono::TimeZone;
    use common::{ErrResp, JAPANESE_TIME_ZONE};

    use crate::util::{
        consultant_disabled_check::check_if_consultant_is_available,
        user_info::{FindUserInfoOperation, UserInfo},
    };

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
    async fn test_check_if_user_account_is_available_returns_false_when_no_user_is_found() {
        let user_info = UserInfo {
            account_id: 6051,
            email_address: "test@test.com".to_string(),
            mfa_enabled_at: None,
            disabled_at: None,
        };
        let op = FindUserInfoOperationMock {
            user_info: &user_info,
        };
        let other_account_id = user_info.account_id + 6501;

        let ret = check_if_consultant_is_available(other_account_id, &op)
            .await
            .expect("failed to get Ok");

        assert!(!ret);
    }

    #[tokio::test]
    async fn test_check_if_user_account_is_available_returns_false_when_user_is_disabled() {
        let user_info = UserInfo {
            account_id: 6051,
            email_address: "test@test.com".to_string(),
            mfa_enabled_at: None,
            disabled_at: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2022, 1, 3, 23, 59, 59)
                    .unwrap(),
            ),
        };
        let op = FindUserInfoOperationMock {
            user_info: &user_info,
        };

        let ret = check_if_consultant_is_available(user_info.account_id, &op)
            .await
            .expect("failed to get Ok");

        assert!(!ret);
    }

    #[tokio::test]
    async fn test_check_if_user_account_is_available_returns_true_when_user_is_not_disabled() {
        let user_info = UserInfo {
            account_id: 6051,
            email_address: "test@test.com".to_string(),
            mfa_enabled_at: None,
            disabled_at: None,
        };
        let op = FindUserInfoOperationMock {
            user_info: &user_info,
        };

        let ret = check_if_consultant_is_available(user_info.account_id, &op)
            .await
            .expect("failed to get Ok");

        assert!(ret);
    }
}
