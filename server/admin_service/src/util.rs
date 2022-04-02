// Copyright 2021 Ken Miura

pub(crate) mod session;
pub(crate) mod validator;

use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};
use serde::Deserialize;

use crate::err::Code;

pub(crate) const ROOT_PATH: &str = "/admin/api";

#[derive(Deserialize)]
pub(crate) struct Pagination {
    pub(crate) page: usize,
    pub(crate) per_page: usize,
}

const MAX_PAGE_SIZE: usize = 50;

pub(crate) fn validate_page_size(page_size: usize) -> Result<(), ErrResp> {
    if page_size > MAX_PAGE_SIZE {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::IllegalPageSize as u32,
            }),
        ));
    }
    Ok(())
}

/// PAY.JPにアクセスするための情報を保持する変数
// pub(crate) static ACCESS_INFO: Lazy<AccessInfo> = Lazy::new(|| {
//     let url_without_path = var(KEY_TO_PAYMENT_PLATFORM_API_URL).unwrap_or_else(|_| {
//         panic!(
//             "Not environment variable found: environment variable \"{}\" must be set",
//             KEY_TO_PAYMENT_PLATFORM_API_URL
//         )
//     });
//     let username = var(KEY_TO_PAYMENT_PLATFORM_API_USERNAME).unwrap_or_else(|_| {
//         panic!(
//             "Not environment variable found: environment variable \"{}\" must be set",
//             KEY_TO_PAYMENT_PLATFORM_API_USERNAME
//         )
//     });
//     let password = var(KEY_TO_PAYMENT_PLATFORM_API_PASSWORD).unwrap_or_else(|_| {
//         panic!(
//             "Not environment variable found: environment variable \"{}\" must be set",
//             KEY_TO_PAYMENT_PLATFORM_API_PASSWORD
//         )
//     });
//     let access_info = AccessInfo::new(url_without_path, username, password);
//     access_info.expect("failed to get Ok")
// });

/// テストコードで共通で使うコードをまとめるモジュール
#[cfg(test)]
pub(crate) mod tests {

    use axum::http::StatusCode;

    use crate::err::Code;

    use super::{validate_page_size, MAX_PAGE_SIZE};

    #[test]
    fn validate_page_size_sucees() {
        let _ = validate_page_size(MAX_PAGE_SIZE).expect("failed to get Ok");
    }

    #[test]
    fn validate_page_size_fail() {
        let err_resp = validate_page_size(MAX_PAGE_SIZE + 1).expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(Code::IllegalPageSize as u32, err_resp.1.code);
    }

    use common::{smtp::SendMail, ErrResp};

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

    impl SendMail for SendMailMock {
        fn send_mail(
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
