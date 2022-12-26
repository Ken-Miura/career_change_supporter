// Copyright 2021 Ken Miura

use axum::async_trait;
use common::ErrResp;
use entity::{
    prelude::UserAccount,
    sea_orm::{DatabaseConnection, EntityTrait},
};
use tracing::error;

use crate::err::unexpected_err_resp;

#[async_trait]
pub(crate) trait DisabledCheckOperation {
    /// アカウントが無効化されているかどうか
    ///
    /// - アカウントが存在しない場合、Noneを返す
    /// - アカウントが存在する場合で
    ///   - アカウントが無効化されている場合、Some(true)を返す
    ///   - アカウントが無効化されていない場合、Some(false)を返す
    async fn check_if_account_is_disabled(&self, account_id: i64) -> Result<Option<bool>, ErrResp>;
}

pub(crate) struct DisabledCheckOperationImpl<'a> {
    pool: &'a DatabaseConnection,
}

impl<'a> DisabledCheckOperationImpl<'a> {
    pub(crate) fn new(pool: &'a DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> DisabledCheckOperation for DisabledCheckOperationImpl<'a> {
    async fn check_if_account_is_disabled(&self, account_id: i64) -> Result<Option<bool>, ErrResp> {
        let model = UserAccount::find_by_id(account_id)
            .one(self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find user_account (user_accound_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.disabled_at.is_some()))
    }
}

/// ユーザーが利用可能か確認する。
/// アカウントが存在し、かつ無効化されていない（=利用可能な）場合、trueを返す。そうでない場合、falseを返す。
pub(crate) async fn check_if_user_account_is_available(
    user_account_id: i64,
    op: impl DisabledCheckOperation,
) -> Result<bool, ErrResp> {
    let result_option = op.check_if_account_is_disabled(user_account_id).await?;
    if let Some(disabled) = result_option {
        Ok(!disabled)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {

    use axum::async_trait;
    use common::ErrResp;

    use super::{check_if_user_account_is_available, DisabledCheckOperation};

    struct DisabledCheckOperationMock {
        account_id: i64,
        disabled: Option<bool>,
    }

    #[async_trait]
    impl DisabledCheckOperation for DisabledCheckOperationMock {
        async fn check_if_account_is_disabled(
            &self,
            account_id: i64,
        ) -> Result<Option<bool>, ErrResp> {
            assert_eq!(self.account_id, account_id);
            Ok(self.disabled)
        }
    }

    #[tokio::test]
    async fn test_check_if_user_account_is_available_returns_false_when_no_user_is_found() {
        let account_id = 5325;
        let op = DisabledCheckOperationMock {
            account_id,
            disabled: None,
        };

        let ret = check_if_user_account_is_available(account_id, op)
            .await
            .expect("failed to get Ok");

        assert!(!ret);
    }

    #[tokio::test]
    async fn test_check_if_user_account_is_available_returns_false_when_user_is_disabled() {
        let account_id = 5325;
        let op = DisabledCheckOperationMock {
            account_id,
            disabled: Some(true),
        };

        let ret = check_if_user_account_is_available(account_id, op)
            .await
            .expect("failed to get Ok");

        assert!(!ret);
    }

    #[tokio::test]
    async fn test_check_if_user_account_is_available_returns_true_when_user_is_not_disabled() {
        let account_id = 5325;
        let op = DisabledCheckOperationMock {
            account_id,
            disabled: Some(false),
        };

        let ret = check_if_user_account_is_available(account_id, op)
            .await
            .expect("failed to get Ok");

        assert!(ret);
    }
}
