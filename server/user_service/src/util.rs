// Copyright 2021 Ken Miura

pub(crate) mod document_operation;
pub(crate) mod login_status;
pub(crate) mod terms_of_use;

use chrono::{DateTime, FixedOffset};
use common::ErrResp;
use entity::sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use tracing::error;

use crate::err::unexpected_err_resp;

pub(crate) const ROOT_PATH: &str = "/api";

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
    use common::{smtp::SendMail, ErrResp};

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
}
