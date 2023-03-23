// Copyright 2021 Ken Miura

pub(crate) mod bank_account;
pub(crate) mod charge_metadata_key;
pub(crate) mod consultant_disabled_check;
pub(crate) mod consultation_request;
pub(crate) mod document_operation;
pub(crate) mod fee_per_hour_in_yen_range;
pub(crate) mod identity_check;
pub(crate) mod image_converter;
pub(crate) mod multipart;
pub(crate) mod optional_env_var;
pub(crate) mod platform_fee_rate;
pub(crate) mod request_consultation;
pub(crate) mod rewards;
pub(crate) mod session;
pub(crate) mod terms_of_use;
pub(crate) mod the_other_person_account;
pub(crate) mod user_info;
pub(crate) mod validator;
pub(crate) mod years_of_service_period;

use std::env::var;

use common::{
    payment_platform::{
        AccessInfo, KEY_TO_PAYMENT_PLATFORM_API_PASSWORD, KEY_TO_PAYMENT_PLATFORM_API_URL,
        KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
    },
    ErrRespStruct,
};
use entity::{
    sea_orm::{DatabaseTransaction, EntityTrait, QuerySelect},
    user_account,
};
use once_cell::sync::Lazy;
use tracing::error;

use crate::err::unexpected_err_resp;

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

/// 通常のテストコードに加え、共通で使うモックをまとめる
#[cfg(test)]
pub(crate) mod tests {
    use std::io::Cursor;

    use axum::async_trait;
    use common::{smtp::SendMail, ErrResp};
    use image::{ImageBuffer, ImageOutputFormat, RgbImage};

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

    pub(super) fn create_dummy_jpeg_image() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(128, 128);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        img.write_to(&mut bytes, ImageOutputFormat::Jpeg(85))
            .expect("failed to get Ok");
        bytes
    }
}
