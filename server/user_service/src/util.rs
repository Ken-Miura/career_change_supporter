// Copyright 2021 Ken Miura

pub(crate) mod available_user_account;
pub(crate) mod bank_account;
pub(crate) mod charge_metadata_key;
pub(crate) mod consultation;
pub(crate) mod consultation_request;
pub(crate) mod disabled_check;
pub(crate) mod document_operation;
pub(crate) mod fee_per_hour_in_yen_range;
pub(crate) mod identity_checker;
pub(crate) mod image_converter;
pub(crate) mod multipart;
pub(crate) mod optional_env_var;
pub(crate) mod rewards;
pub(crate) mod session;
pub(crate) mod terms_of_use;
pub(crate) mod validator;
pub(crate) mod years_of_service_period;

use std::env::var;

use common::{
    payment_platform::{
        AccessInfo, KEY_TO_PAYMENT_PLATFORM_API_PASSWORD, KEY_TO_PAYMENT_PLATFORM_API_URL,
        KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
    },
    ErrResp,
};
use entity::{
    sea_orm::{DatabaseConnection, EntityTrait},
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

async fn find_user_account_by_user_account_id(
    pool: &DatabaseConnection,
    user_account_id: i64,
) -> Result<Option<user_account::Model>, ErrResp> {
    let model = entity::prelude::UserAccount::find_by_id(user_account_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find user_account (user_account_id): {}): {}",
                user_account_id, e
            );
            unexpected_err_resp()
        })?;
    Ok(model)
}

/// 小数点以下2桁目を四捨五入し、小数点以下1桁目までを示す少数を文字列表現として返す。
pub(crate) fn round_to_one_decimal_places(rating: f64) -> String {
    let result = (rating * 10.0).round() / 10.0;
    // format!("{:.1}", rating) のみで少数点以下2桁目を四捨五入し、小数点以下1桁まで求める動作となる。
    // しかし、下記のドキュメントに、その動作（四捨五入）に関して正式な仕様として記載がないため、四捨五入の箇所は自身で実装する。
    // https://doc.rust-lang.org/std/fmt/
    format!("{:.1}", result)
}

/// 通常のテストコードに加え、共通で使うモックをまとめる
#[cfg(test)]
pub(crate) mod tests {
    use std::io::Cursor;

    use axum::async_trait;
    use common::{smtp::SendMail, ErrResp};
    use image::{ImageBuffer, ImageOutputFormat, RgbImage};
    use once_cell::sync::Lazy;

    use super::round_to_one_decimal_places;

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

    #[derive(Debug)]
    struct RoundToOneDecimalPlacesTestCase {
        name: String,
        input: f64,
        expected: String,
    }

    static ROUNT_TO_ONE_DECIMAL_PLACES_TEST_CASE_SET: Lazy<Vec<RoundToOneDecimalPlacesTestCase>> =
        Lazy::new(|| {
            vec![
                RoundToOneDecimalPlacesTestCase {
                    name: "x.x4 -> round down".to_string(),
                    input: 3.64,
                    expected: "3.6".to_string(),
                },
                RoundToOneDecimalPlacesTestCase {
                    name: "x.x5 -> round up".to_string(),
                    input: 3.65,
                    expected: "3.7".to_string(),
                },
                RoundToOneDecimalPlacesTestCase {
                    name: "x.95 -> round up".to_string(),
                    input: 3.95,
                    expected: "4.0".to_string(),
                },
                RoundToOneDecimalPlacesTestCase {
                    name: "x.x0 -> round down".to_string(),
                    input: 4.10,
                    expected: "4.1".to_string(),
                },
                RoundToOneDecimalPlacesTestCase {
                    name: "x.x9 -> round up".to_string(),
                    input: 2.19,
                    expected: "2.2".to_string(),
                },
            ]
        });

    #[test]
    fn test_round_to_one_decimal_places() {
        for test_case in ROUNT_TO_ONE_DECIMAL_PLACES_TEST_CASE_SET.iter() {
            let result = round_to_one_decimal_places(test_case.input);
            let message = format!("test case \"{}\" failed", test_case.name.clone());
            assert_eq!(test_case.expected, result, "{}", message);
        }
    }
}
