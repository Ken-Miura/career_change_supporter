// Copyright 2021 Ken Miura

pub(crate) mod bank_account;
pub(crate) mod charge_metadata_key;
pub(crate) mod consultation;
pub(crate) mod disabled_check;
pub(crate) mod fee_per_hour_in_yen_range;
pub(crate) mod optional_env_var;
pub(crate) mod rewards;
pub(crate) mod session;
pub(crate) mod terms_of_use;
pub(crate) mod validator;
pub(crate) mod years_of_service_period;

use std::{env::var, io::Cursor};

use axum::{http::StatusCode, Json};
use bytes::Bytes;
use chrono::{DateTime, FixedOffset};
use common::{
    payment_platform::{
        AccessInfo, KEY_TO_PAYMENT_PLATFORM_API_PASSWORD, KEY_TO_PAYMENT_PLATFORM_API_URL,
        KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
    },
    ApiError, ErrResp, ErrRespStruct, JAPANESE_TIME_ZONE,
};
use entity::{
    document,
    prelude::ConsultationReq,
    sea_orm::{
        ActiveModelTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QuerySelect, Set,
    },
};
use image::{ImageError, ImageFormat};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

pub(crate) const ROOT_PATH: &str = "/api";

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

#[derive(Clone, Debug)]
pub(crate) struct UserAccount {
    pub(crate) email_address: String,
    pub(crate) disabled_at: Option<DateTime<FixedOffset>>,
}

/// ユーザーが存在する場合、[UserAccount]を返す。存在しない場合、Noneを返す。
async fn get_if_user_exists(
    pool: &DatabaseConnection,
    user_account_id: i64,
) -> Result<Option<UserAccount>, ErrResp> {
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
    Ok(model.map(|m| UserAccount {
        email_address: m.email_address,
        disabled_at: m.disabled_at,
    }))
}

/// ユーザーが利用可能な場合（UserAccountが存在し、かつdisabled_atがNULLである場合）、[UserAccount]を返す
pub(crate) async fn get_if_user_account_is_available(
    pool: &DatabaseConnection,
    user_account_id: i64,
) -> Result<Option<UserAccount>, ErrResp> {
    let user = get_if_user_exists(pool, user_account_id).await?;
    let result = match user {
        Some(u) => {
            if u.disabled_at.is_none() {
                Some(u)
            } else {
                None
            }
        }
        None => None,
    };
    Ok(result)
}

/// ユーザーが利用可能か確認する。
/// UserAccountが存在し、かつdisabled_atがNULLである場合、trueを返す。そうでない場合、falseを返す。
pub(crate) async fn check_if_user_account_is_available(
    pool: &DatabaseConnection,
    user_account_id: i64,
) -> Result<bool, ErrResp> {
    let user = get_if_user_account_is_available(pool, user_account_id).await?;
    Ok(user.is_some())
}

pub(crate) fn validate_consultation_req_id_is_positive(
    consultation_req_id: i64,
) -> Result<(), ErrResp> {
    if !consultation_req_id.is_positive() {
        error!(
            "consultation_req_id ({}) is not positive",
            consultation_req_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonPositiveConsultationReqId as u32,
            }),
        ));
    }
    Ok(())
}

/// 相談申し込み
#[derive(Clone, Debug)]
pub(crate) struct ConsultationRequest {
    pub(crate) consultation_req_id: i64,
    pub(crate) user_account_id: i64,
    pub(crate) consultant_id: i64,
    pub(crate) fee_per_hour_in_yen: i32,
    pub(crate) first_candidate_date_time_in_jst: DateTime<FixedOffset>,
    pub(crate) second_candidate_date_time_in_jst: DateTime<FixedOffset>,
    pub(crate) third_candidate_date_time_in_jst: DateTime<FixedOffset>,
    pub(crate) charge_id: String,
    pub(crate) latest_candidate_date_time_in_jst: DateTime<FixedOffset>,
}

/// 相談申し込みを取得する
///
/// 取得した相談申し込みは、consultant_idがリクエスト送信元のユーザーIDと一致するか（操作可能なユーザーか）必ずチェックする
pub(crate) async fn find_consultation_req_by_consultation_req_id(
    pool: &DatabaseConnection,
    consultation_req_id: i64,
) -> Result<Option<ConsultationRequest>, ErrResp> {
    let model = ConsultationReq::find_by_id(consultation_req_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find consultation_req (consultation_req_id: {}): {}",
                consultation_req_id, e
            );
            unexpected_err_resp()
        })?;
    Ok(model.map(|m| ConsultationRequest {
        consultation_req_id: m.consultation_req_id,
        user_account_id: m.user_account_id,
        consultant_id: m.consultant_id,
        fee_per_hour_in_yen: m.fee_per_hour_in_yen,
        first_candidate_date_time_in_jst: m
            .first_candidate_date_time
            .with_timezone(&(*JAPANESE_TIME_ZONE)),
        second_candidate_date_time_in_jst: m
            .second_candidate_date_time
            .with_timezone(&(*JAPANESE_TIME_ZONE)),
        third_candidate_date_time_in_jst: m
            .third_candidate_date_time
            .with_timezone(&(*JAPANESE_TIME_ZONE)),
        charge_id: m.charge_id,
        latest_candidate_date_time_in_jst: m
            .latest_candidate_date_time
            .with_timezone(&(*JAPANESE_TIME_ZONE)),
    }))
}

/// 取得した相談申し込みの存在確認をする
pub(crate) fn consultation_req_exists(
    consultation_request: Option<ConsultationRequest>,
    consultation_req_id: i64,
) -> Result<ConsultationRequest, ErrResp> {
    let req = consultation_request.ok_or_else(|| {
        error!(
            "no consultation_req (consultation_req_id: {}) found",
            consultation_req_id
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationReqFound as u32,
            }),
        )
    })?;
    Ok(req)
}

pub(crate) fn convert_payment_err_to_err_resp(e: &common::payment_platform::Error) -> ErrResp {
    match e {
        common::payment_platform::Error::RequestProcessingError(_) => unexpected_err_resp(),
        common::payment_platform::Error::ApiError(err_info) => {
            let err_detail = &err_info.error;
            // https://pay.jp/docs/api/#error
            // status、typeとcodeがエラーハンドリングに使用可能に見える。
            // そのうち、typeはどのような場合に発生するエラーなのか説明が抽象的すぎてわからない。そのため、エラーハンドリングにはcodeとstatusを用いる。
            // codeの方がより詳細なエラーを示している。そのため、まずはcodeがあるか確認し、存在する場合はそちらをもとにエラーハンドリングし、なければstatusを用いる。
            if let Some(code) = err_detail.code.clone() {
                create_err_resp_from_code(code.as_str())
            } else {
                create_err_resp_from_status(err_detail.status)
            }
        }
    }
}

fn create_err_resp_from_code(code: &str) -> ErrResp {
    if code == "incorrect_card_data" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PayJpCodeIncorrectCardData as u32,
            }),
        )
    } else if code == "card_declined" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PayJpCodeCardDeclined as u32,
            }),
        )
    } else if code == "card_flagged" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PayJpCodeCardFlagged as u32,
            }),
        )
    } else if code == "unacceptable_brand" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PayJpCodeUnacceptableBrand as u32,
            }),
        )
    } else if code == "over_capacity" {
        (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ApiError {
                code: Code::ReachPaymentPlatformRateLimit as u32,
            }),
        )
    } else if code == "three_d_secure_incompleted" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PayJpCodeThreeDSecureIncompleted as u32,
            }),
        )
    } else if code == "three_d_secure_failed" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PayJpCodeThreeDSecureFailed as u32,
            }),
        )
    } else if code == "not_in_three_d_secure_flow" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PayJpCodeNotInThreeDSecureFlow as u32,
            }),
        )
    } else {
        // 上記で記載のcode以外は、ユーザーが利用するサービスでは想定していないもののため、unexpected_err_resp() で丸めて返却する
        unexpected_err_resp()
    }
}

fn create_err_resp_from_status(status: u32) -> ErrResp {
    if status == 402 {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::CardAuthPaymentError as u32,
            }),
        )
    } else if status == 429 {
        (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ApiError {
                code: Code::ReachPaymentPlatformRateLimit as u32,
            }),
        )
    } else {
        unexpected_err_resp()
    }
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

    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use bytes::Bytes;
    use common::{
        payment_platform::{ErrorDetail, ErrorInfo},
        smtp::SendMail,
        ApiError, ErrResp,
    };
    use image::{ImageBuffer, ImageFormat, ImageOutputFormat, RgbImage};
    use once_cell::sync::Lazy;

    use crate::err::Code;

    use super::{
        clone_file_name_if_exists, convert_jpeg_to_png, convert_payment_err_to_err_resp,
        round_to_one_decimal_places,
    };

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
        img.write_to(&mut bytes, ImageOutputFormat::Jpeg(85))
            .expect("failed to get Ok");
        bytes
    }

    fn create_dummy_png_image() -> Cursor<Vec<u8>> {
        let img: RgbImage = ImageBuffer::new(128, 128);
        let mut bytes = Cursor::new(Vec::with_capacity(50 * 1024));
        img.write_to(&mut bytes, ImageOutputFormat::Png)
            .expect("failed to get Ok");
        bytes
    }

    fn create_converted_image(jpg_img: Vec<u8>) -> Cursor<Vec<u8>> {
        let img = image::io::Reader::with_format(Cursor::new(jpg_img), ImageFormat::Jpeg)
            .decode()
            .expect("failed to get Ok");
        let mut png_img = Cursor::new(vec![]);
        img.write_to(&mut png_img, image::ImageOutputFormat::Png)
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

    #[derive(Debug)]
    struct ConvertPaymentErrToErrRespTestCase {
        name: String,
        input: ConvertPaymentErrToErrRespInput,
        expected: ErrResp,
    }

    #[derive(Debug)]
    struct ConvertPaymentErrToErrRespInput {
        err: common::payment_platform::Error,
    }

    static CONVERT_PAYMENT_ERR_TO_ERR_RESP_TEST_CASE_SET: Lazy<
        Vec<ConvertPaymentErrToErrRespTestCase>,
    > = Lazy::new(|| {
        // ErrorDetailのメンバーは、実際に返ってくる値が不明なため使う値のみ正確に埋める。
        // pay.jpを使う中で実際に正確な値がわかった場合、随時更新していく。
        vec![
            ConvertPaymentErrToErrRespTestCase {
                name: "status 402".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 402,
                            r#type: "type".to_string(),
                            code: None,
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::CardAuthPaymentError as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "status 429".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 429,
                            r#type: "type".to_string(),
                            code: None,
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ApiError {
                        code: Code::ReachPaymentPlatformRateLimit as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code incorrect_card_data".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("incorrect_card_data".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeIncorrectCardData as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code card_declined".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("card_declined".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeCardDeclined as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code card_flagged".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("card_flagged".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeCardFlagged as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code unacceptable_brand".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("unacceptable_brand".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeUnacceptableBrand as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code over_capacity".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("over_capacity".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ApiError {
                        code: Code::ReachPaymentPlatformRateLimit as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code three_d_secure_incompleted".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("three_d_secure_incompleted".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeThreeDSecureIncompleted as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code three_d_secure_failed".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("three_d_secure_failed".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeThreeDSecureFailed as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code not_in_three_d_secure_flow".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("not_in_three_d_secure_flow".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeNotInThreeDSecureFlow as u32,
                    }),
                ),
            },
        ]
    });

    #[test]
    fn test_convert_payment_err_to_err_resp() {
        for test_case in CONVERT_PAYMENT_ERR_TO_ERR_RESP_TEST_CASE_SET.iter() {
            let err_resp = convert_payment_err_to_err_resp(&test_case.input.err);
            let message = format!("test case \"{}\" failed", test_case.name.clone());
            assert_eq!(test_case.expected.0, err_resp.0, "{}", message);
            assert_eq!(test_case.expected.1 .0, err_resp.1 .0, "{}", message);
        }
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
