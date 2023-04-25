// Copyright 2021 Ken Miura

pub(crate) mod document_operation;
pub(crate) mod login_status;
pub(crate) mod optional_env_var;
pub(crate) mod request_consultation;
pub(crate) mod rewards;
pub(crate) mod session;
pub(crate) mod terms_of_use;
pub(crate) mod the_other_person_account;
pub(crate) mod user_info;
pub(crate) mod years_of_service_period;

use std::env::var;

use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, FixedOffset};
use common::{
    payment_platform::{
        AccessInfo, KEY_TO_PAYMENT_PLATFORM_API_PASSWORD, KEY_TO_PAYMENT_PLATFORM_API_URL,
        KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
    },
    ApiError, ErrResp, ErrRespStruct,
};
use entity::{
    sea_orm::{
        ActiveModelTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QuerySelect, Set,
    },
    user_account,
};
use once_cell::sync::Lazy;
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

use self::user_info::{FindUserInfoOperation, UserInfo};

pub(crate) const ROOT_PATH: &str = "/api";

/// PAY.JPにアクセスするための情報を保持する変数
pub(crate) static ACCESS_INFO: Lazy<AccessInfo> = Lazy::new(|| {
    let url_without_path = var(KEY_TO_PAYMENT_PLATFORM_API_URL).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_PAYMENT_PLATFORM_API_URL
        )
    });
    let username = var(KEY_TO_PAYMENT_PLATFORM_API_USERNAME).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_PAYMENT_PLATFORM_API_USERNAME
        )
    });
    let password = var(KEY_TO_PAYMENT_PLATFORM_API_PASSWORD).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_PAYMENT_PLATFORM_API_PASSWORD
        )
    });
    let access_info = AccessInfo::new(url_without_path, username, password);
    access_info.expect("failed to get Ok")
});

pub(crate) async fn find_user_account_by_user_account_id_with_exclusive_lock(
    txn: &DatabaseTransaction,
    user_account_id: i64,
) -> Result<Option<user_account::Model>, ErrRespStruct> {
    let model = entity::prelude::UserAccount::find_by_id(user_account_id)
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

pub(crate) async fn get_user_info_if_available(
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

pub(crate) async fn update_last_login(
    account_id: i64,
    login_time: &DateTime<FixedOffset>,
    pool: &DatabaseConnection,
) -> Result<(), ErrResp> {
    let user_account_model = entity::user_account::ActiveModel {
        user_account_id: Set(account_id),
        last_login_time: Set(Some(*login_time)),
        ..Default::default()
    };
    let _ = user_account_model.update(pool).await.map_err(|e| {
        error!(
            "failed to update user_account (user_account_id: {}): {}",
            account_id, e
        );
        unexpected_err_resp()
    })?;
    Ok(())
}

/// 通常のテストコードに加え、共通で使うモックをまとめる
#[cfg(test)]
pub(crate) mod tests {

    use axum::async_trait;
    use axum::http::StatusCode;
    use chrono::TimeZone;
    use common::{smtp::SendMail, ErrResp, JAPANESE_TIME_ZONE};

    use crate::{
        err::Code,
        util::{get_user_info_if_available, user_info::UserInfo},
    };

    use super::user_info::FindUserInfoOperation;

    #[derive(Clone, Debug)]
    pub(crate) struct SendMailMock {
        to: String,
        from: String,
        subject: String,
        text: String,
    }

    impl SendMailMock {
        pub(crate) fn new(to: String, from: String, subject: String, text: String) -> Self {
            Self {
                to,
                from,
                subject,
                text,
            }
        }
    }

    #[async_trait]
    impl SendMail for SendMailMock {
        async fn send_mail(
            &self,
            to: &str,
            from: &str,
            subject: &str,
            text: &str,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.to, to);
            assert_eq!(self.from, from);
            assert_eq!(self.subject, subject);
            assert_eq!(self.text, text);
            Ok(())
        }
    }

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
