// Copyright 2021 Ken Miura

pub(crate) mod disabled_check;
pub(crate) mod session;
pub(crate) mod terms_of_use;
pub(crate) mod validator;

use std::{env::var, io::Cursor};

use axum::{http::StatusCode, Json};
use bytes::Bytes;
use common::{
    payment_platform::{
        AccessInfo, KEY_TO_PAYMENT_PLATFORM_API_PASSWORD, KEY_TO_PAYMENT_PLATFORM_API_URL,
        KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
    },
    ApiError, ErrResp, ErrRespStruct,
};
use entity::{
    document,
    sea_orm::{
        ActiveModelTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QuerySelect, Set,
    },
};
use image::{ImageError, ImageFormat};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

pub(crate) const MIN_FEE_PER_HOUR_IN_YEN: i32 = 3000;
pub(crate) const MAX_FEE_PER_HOUR_IN_YEN: i32 = 50000;

pub(crate) const VALID_YEARS_OF_SERVICE_PERIOD_THREE: i32 = 3;
pub(crate) const VALID_YEARS_OF_SERVICE_PERIOD_FIVE: i32 = 5;
pub(crate) const VALID_YEARS_OF_SERVICE_PERIOD_TEN: i32 = 10;
pub(crate) const VALID_YEARS_OF_SERVICE_PERIOD_FIFTEEN: i32 = 15;
pub(crate) const VALID_YEARS_OF_SERVICE_PERIOD_TWENTY: i32 = 20;

pub(crate) const ROOT_PATH: &str = "/api";

pub(crate) const EXPIRY_DAYS: u32 = 7;

pub(crate) const KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ: &str = "consultant_id";
pub(crate) const KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ: &str = "first_candidate_in_jst";
pub(crate) const KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ: &str = "second_candidate_in_jst";
pub(crate) const KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ: &str = "third_candidate_in_jst";

pub(crate) type FileNameAndBinary = (String, Cursor<Vec<u8>>);

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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct BankAccount {
    pub bank_code: String,
    pub branch_code: String,
    pub account_type: String,
    pub account_number: String,
    pub account_holder_name: String,
}

/// jpeg画像をpng画像に変換する<br>
/// <br>
/// 画像ファイルの中のメタデータに悪意ある内容が含まれている場合が考えられるので、画像情報以外のメタデータを取り除く必要がある。
/// メタデータを取り除くのに画像形式を変換するのが最も容易な実装のため、画像形式の変換を行っている。
pub(crate) fn convert_jpeg_to_png(data: Bytes) -> Result<Cursor<Vec<u8>>, ErrResp> {
    let img = image::io::Reader::with_format(Cursor::new(data), ImageFormat::Jpeg)
        .decode()
        .map_err(|e| {
            error!("failed to decode jpeg image: {}", e);
            match e {
                ImageError::Decoding(_) => (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidJpegImage as u32,
                    }),
                ),
                _ => unexpected_err_resp(),
            }
        })?;
    let mut bytes = Cursor::new(vec![]);
    img.write_to(&mut bytes, image::ImageOutputFormat::Png)
        .map_err(|e| {
            error!("failed to write image on buffer: {}", e);
            unexpected_err_resp()
        })?;
    Ok(bytes)
}

/// 引数が存在する場合、ファイル名のみ複製を行う
pub(crate) fn clone_file_name_if_exists(
    file_name_and_binary_option: Option<FileNameAndBinary>,
) -> (Option<FileNameAndBinary>, Option<String>) {
    if let Some(file_name_and_binary) = file_name_and_binary_option {
        let image2 = Some((file_name_and_binary.0.clone(), file_name_and_binary.1));
        let image2_file_name_without_ext = Some(file_name_and_binary.0);
        return (image2, image2_file_name_without_ext);
    };
    (None, None)
}

/// 共有ロックを行い、documentテーブルからドキュメントIDを取得する
///
/// opensearch呼び出しとセットで利用するため、トランザクション内で利用することが前提となる
pub(crate) async fn find_document_model_by_user_account_id_with_shared_lock(
    txn: &DatabaseTransaction,
    user_account_id: i64,
) -> Result<Option<document::Model>, ErrRespStruct> {
    let doc_option = document::Entity::find_by_id(user_account_id)
        .lock_shared()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find document (user_account_id: {}): {}",
                user_account_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(doc_option)
}

/// documentテーブルにドキュメントIDを挿入する
///
/// opensearchにドキュメントを登録する際、そのドキュメントIDをDBに保管しておくために利用する<br>
/// opensearch呼び出しとセットで利用するため、トランザクション内で利用することが前提となる
pub(crate) async fn insert_document(
    txn: &DatabaseTransaction,
    user_account_id: i64,
    document_id: i64,
) -> Result<(), ErrRespStruct> {
    let document = document::ActiveModel {
        user_account_id: Set(user_account_id),
        document_id: Set(document_id),
    };
    let _ = document.insert(txn).await.map_err(|e| {
        error!(
            "failed to insert document (user_account_id: {}, document_id: {}): {}",
            user_account_id, document_id, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

/// Identityが存在するか確認する。存在する場合、trueを返す。そうでない場合、falseを返す。
///
/// 個人情報の登録をしていないと使えないAPIに関して、処理を継続してよいか確認するために利用する。
pub(crate) async fn check_if_identity_exists(
    pool: &DatabaseConnection,
    account_id: i64,
) -> Result<bool, ErrResp> {
    let model = entity::prelude::Identity::find_by_id(account_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find identity (user_account_id: {}): {}",
                account_id, e
            );
            unexpected_err_resp()
        })?;
    Ok(model.is_some())
}

/// コンサルタントが利用可能か確認する。
/// コンサルタントのUserAccountが存在し、かつdisabled_atがNULLである場合、trueを返す。そうでない場合、falseを返す。
pub(crate) async fn check_if_consultant_is_available(
    pool: &DatabaseConnection,
    consultant_id: i64,
) -> Result<bool, ErrResp> {
    let model = entity::prelude::UserAccount::find_by_id(consultant_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find user_account (user_account_id): {}): {}",
                consultant_id, e
            );
            unexpected_err_resp()
        })?;
    let available = match model {
        Some(user) => user.disabled_at.is_none(),
        None => false,
    };
    Ok(available)
}

/// 通常のテストコードに加え、共通で使うモックをまとめる
#[cfg(test)]
pub(crate) mod tests {
    use std::io::Cursor;

    use axum::async_trait;
    use axum::http::StatusCode;
    use bytes::Bytes;
    use common::{smtp::SendMail, ErrResp};
    use image::{ImageBuffer, ImageFormat, ImageOutputFormat, RgbImage};

    use crate::err::Code;

    use super::{clone_file_name_if_exists, convert_jpeg_to_png};

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

    #[test]
    fn convert_jpeg_to_png_returns_ok_if_convert_is_success() {
        let jpeg_image = create_dummy_jpeg_image();
        let expected_image = create_converted_image(jpeg_image.clone().into_inner());

        let result = convert_jpeg_to_png(Bytes::from(jpeg_image.into_inner()));

        let result_image = result.expect("failed to get Ok");
        assert_eq!(expected_image, result_image);
    }

    #[test]
    fn convert_jpeg_to_png_returns_err_if_format_other_than_jpg_is_passed() {
        let png_image = create_dummy_png_image();

        let result = convert_jpeg_to_png(Bytes::from(png_image.into_inner()));

        let result = result.expect_err("failed to get Err");
        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1.code, Code::InvalidJpegImage as u32);
    }

    fn create_dummy_jpeg_image() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(128, 128);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        let _ = img
            .write_to(&mut bytes, ImageOutputFormat::Jpeg(85))
            .expect("failed to get Ok");
        bytes
    }

    fn create_dummy_png_image() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(128, 128);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        let _ = img
            .write_to(&mut bytes, ImageOutputFormat::Png)
            .expect("failed to get Ok");
        bytes
    }

    fn create_converted_image(jpg_img: Vec<u8>) -> Cursor<Vec<u8>> {
        let img = image::io::Reader::with_format(Cursor::new(jpg_img), ImageFormat::Jpeg)
            .decode()
            .expect("failed to get Ok");
        let mut png_img = Cursor::new(vec![]);
        let _ = img
            .write_to(&mut png_img, image::ImageOutputFormat::Png)
            .expect("failed to get Ok");
        png_img
    }

    #[test]
    fn clone_file_name_if_exists_returns_none_if_none_is_passed() {
        let (ret1, ret2) = clone_file_name_if_exists(None);
        assert_eq!(None, ret1);
        assert_eq!(None, ret2);
    }

    #[test]
    fn clone_file_name_if_exists_returns_arg_and_file_name_if_value_is_passed() {
        let file_name = "c89bfd885f6df5fd-345306a47b7dd758";
        let binary = create_dummy_jpeg_image();
        let file_name_and_binary = (file_name.to_string(), binary);

        let (ret1, ret2) = clone_file_name_if_exists(Some(file_name_and_binary.clone()));

        assert_eq!(Some(file_name_and_binary), ret1);
        assert_eq!(Some(file_name.to_string()), ret2);
    }
}
