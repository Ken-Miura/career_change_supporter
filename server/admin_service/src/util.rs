// Copyright 2021 Ken Miura

pub(crate) mod session;
pub(crate) mod validator;

use axum::{http::StatusCode, Json};
use common::{
    storage::{self, IDENTITY_IMAGES_BUCKET_NAME},
    ApiError, ErrResp, ErrRespStruct,
};
use entity::{
    sea_orm::{DatabaseTransaction, EntityTrait, QuerySelect},
    user_account,
};
use serde::Deserialize;
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

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

/// 承認、拒否を行う際にユーザーがアカウントを削除しないことを保証するために明示的にロックを取得し、user_accountを取得する
pub(crate) async fn find_user_model_by_user_account_id(
    txn: &DatabaseTransaction,
    user_account_id: i64,
) -> Result<Option<user_account::Model>, ErrRespStruct> {
    let user_model_option = user_account::Entity::find_by_id(user_account_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find user_account (user_account_id: {}): {}",
                user_account_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(user_model_option)
}

pub(crate) async fn delete_identity_images(
    user_account_id: i64,
    image1_file_name_without_ext: String,
    image2_file_name_without_ext: Option<String>,
) -> Result<(), ErrRespStruct> {
    let image1_key = format!("{}/{}.png", user_account_id, image1_file_name_without_ext);
    let _ = storage::delete_object(IDENTITY_IMAGES_BUCKET_NAME, image1_key.as_str())
        .await
        .map_err(|e| {
            error!(
                "failed to delete identity image1 (key: {}): {}",
                image1_key, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;

    if let Some(image2_file_name_without_ext) = image2_file_name_without_ext {
        let image2_key = format!("{}/{}.png", user_account_id, image2_file_name_without_ext);
        let _ = storage::delete_object(IDENTITY_IMAGES_BUCKET_NAME, image2_key.as_str())
            .await
            .map_err(|e| {
                error!(
                    "failed to delete identity image2 (key: {}): {}",
                    image2_key, e
                );
                ErrRespStruct {
                    err_resp: unexpected_err_resp(),
                }
            })?;
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
