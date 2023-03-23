// Copyright 2022 Ken Miura

use common::ErrResp;

use super::user_info::{FindUserInfoOperation, UserInfo};

/// アカウントが存在し、かつ無効化されていない場合、UserInfoを返す
pub(crate) async fn get_the_other_person_info_if_available(
    account_id: i64,
    op: &impl FindUserInfoOperation,
) -> Result<Option<UserInfo>, ErrResp> {
    let user_info = op.find_user_info_by_account_id(account_id).await?;
    if let Some(u) = user_info {
        if u.disabled_at.is_some() {
            Ok(None)
        } else {
            Ok(Some(u))
        }
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use chrono::TimeZone;
    use common::{ErrResp, JAPANESE_TIME_ZONE};

    use crate::util::{
        the_other_person_account::get_the_other_person_info_if_available,
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
    async fn test_get_the_other_person_info_if_available_returns_none_when_no_account_is_found() {
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

        let ret = get_the_other_person_info_if_available(other_account_id, &op)
            .await
            .expect("failed to get Ok");

        assert_eq!(ret, None);
    }

    #[tokio::test]
    async fn test_get_the_other_person_info_if_available_returns_none_when_user_is_disabled() {
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

        let ret = get_the_other_person_info_if_available(user_info.account_id, &op)
            .await
            .expect("failed to get Ok");

        assert_eq!(ret, None);
    }

    #[tokio::test]
    async fn test_get_the_other_person_info_if_available_returns_user_info_when_user_is_not_disabled(
    ) {
        let user_info = UserInfo {
            account_id: 6051,
            email_address: "test@test.com".to_string(),
            mfa_enabled_at: None,
            disabled_at: None,
        };
        let op = FindUserInfoOperationMock {
            user_info: &user_info,
        };

        let ret = get_the_other_person_info_if_available(user_info.account_id, &op)
            .await
            .expect("failed to get Ok");

        assert_eq!(ret, Some(user_info));
    }
}
