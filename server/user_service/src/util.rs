// Copyright 2021 Ken Miura

pub(crate) mod disabled_check;
pub(crate) mod rewards;
pub(crate) mod session;
pub(crate) mod terms_of_use;
pub(crate) mod validator;

use std::{env::var, io::Cursor};

use axum::{http::StatusCode, Json};
use bytes::Bytes;
use chrono::TimeZone;
use common::{
    payment_platform::{
        AccessInfo, KEY_TO_PAYMENT_PLATFORM_API_PASSWORD, KEY_TO_PAYMENT_PLATFORM_API_URL,
        KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
    },
    ApiError, ErrResp, ErrRespStruct, JAPANESE_TIME_ZONE,
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
pub(crate) const MAX_FEE_PER_HOUR_IN_YEN: i32 = 10000;

pub(crate) const VALID_YEARS_OF_SERVICE_PERIOD_THREE: i32 = 3;
pub(crate) const VALID_YEARS_OF_SERVICE_PERIOD_FIVE: i32 = 5;
pub(crate) const VALID_YEARS_OF_SERVICE_PERIOD_TEN: i32 = 10;
pub(crate) const VALID_YEARS_OF_SERVICE_PERIOD_FIFTEEN: i32 = 15;
pub(crate) const VALID_YEARS_OF_SERVICE_PERIOD_TWENTY: i32 = 20;

pub(crate) const ROOT_PATH: &str = "/api";

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

pub(crate) const KEY_TO_MAX_ANNUAL_REWARDS_IN_YEN: &str = "MAX_ANNUAL_REWARDS_IN_YEN";
/// 年間で稼ぐことが可能な最大報酬額（単位：円）
///
/// 動作確認時の利便性のために環境変数をセットする選択肢を用意しているただけで、原則、環境変数をセットせず、デフォルト値を用いる。
pub(crate) static MAX_ANNUAL_REWARDS_IN_YEN: Lazy<i32> = Lazy::new(|| {
    let max_annual_rewards =
        var(KEY_TO_MAX_ANNUAL_REWARDS_IN_YEN).unwrap_or_else(|_| "470000".to_string());
    let max_annual_rewards = max_annual_rewards
        .parse()
        .expect("failed to parse MAX_ANNUAL_REWARDS_IN_YEN");
    if max_annual_rewards <= 0 {
        panic!(
            "MAX_ANNUAL_REWARDS_IN_YEN must be positive: {}",
            max_annual_rewards
        );
    }
    max_annual_rewards
});

pub(crate) const KEY_TO_MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS: &str =
    "MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS";
/// 相談者が相談依頼を行った日時を起点とし、相談開始日時までの秒単位での最小期間
///
/// 動作確認時に待機時間を減らすために環境変数をセットする選択肢を用意しているただけで、原則、環境変数をセットせず、デフォルト値を用いる。
pub(crate) static MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS: Lazy<i64> = Lazy::new(|| {
    let min_duration_in_seconds = var(KEY_TO_MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS)
        .unwrap_or_else(|_| {
            "259200".to_string() // 3 days
        });
    let min_duration_in_seconds = min_duration_in_seconds
        .parse()
        .expect("failed to parse MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS");
    if min_duration_in_seconds < 0 {
        panic!(
            "MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS must be 0 or positive ({})",
            min_duration_in_seconds
        );
    };
    min_duration_in_seconds
});

pub(crate) const KEY_TO_MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS: &str =
    "MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS";
/// 相談者が相談依頼を行った日時を起点とし、相談開始日時までの秒単位での最大期間
///
/// 動作確認時に待機時間を減らすために環境変数をセットする選択肢を用意しているただけで、原則、環境変数をセットせず、デフォルト値を用いる。
pub(crate) static MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS: Lazy<i64> = Lazy::new(|| {
    let max_duration_in_seconds = var(KEY_TO_MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS)
        .unwrap_or_else(|_| {
            "1814400".to_string() // 21 days
        });
    let max_duration_in_seconds = max_duration_in_seconds
        .parse()
        .expect("failed to parse MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS");
    if max_duration_in_seconds <= *MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS {
        panic!("MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS ({}) must be more than MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS ({})", 
            max_duration_in_seconds, *MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS);
    };
    max_duration_in_seconds
});

pub(crate) const KEY_TO_EXPIRY_DAYS_OF_CHARGE: &str = "EXPIRY_DAYS_OF_CHARGE";
/// 相談者が相談依頼を行った日時を起点とし、決済の認証が切れるまでの有効期限（単位：日）
///
/// 動作確認時の利便性のために環境変数をセットする選択肢を用意しているただけで、原則、環境変数をセットせず、デフォルト値を用いる。
pub(crate) static EXPIRY_DAYS_OF_CHARGE: Lazy<u32> = Lazy::new(|| {
    let expiry_days_of_charge =
        var(KEY_TO_EXPIRY_DAYS_OF_CHARGE).unwrap_or_else(|_| "59".to_string());
    let expiry_days_of_charge = expiry_days_of_charge
        .parse()
        .expect("failed to parse EXPIRY_DAYS_OF_CHARGE");
    // https://pay.jp/docs/api/#%E6%94%AF%E6%89%95%E3%81%84%E3%82%92%E4%BD%9C%E6%88%90
    // APIドキュメントでは60まで許容されているが、60を指定したときの挙動が奇妙なので59までしか使わないようにする
    if !(1..=59).contains(&expiry_days_of_charge) {
        panic!(
            "EXPIRY_DAYS_OF_CHARGE ({}) must be between 1 and 59",
            expiry_days_of_charge
        );
    };
    let expiry_days_of_charge_in_seconds = expiry_days_of_charge as i64 * 24 * 60 * 60;
    // TODO: 相談終了後、相談者が相談相手の評価をせず、自動決済の対象となる期間も考慮した制約として書き直す。
    if expiry_days_of_charge_in_seconds < *MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS {
        panic!(
            "EXPIRY_DAYS_OF_CHARGE in seconds ({}) must be MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS ({}) or more",
            expiry_days_of_charge, *MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS
        );
    };
    expiry_days_of_charge
});

pub(crate) const KEY_TO_MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE: &str =
    "MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE";
/// 受け付けた相談を承認する際、相談開始日時までに空いていなければならない最小期間（単位：時間）
///
/// 動作確認時に待機時間を減らすために環境変数をセットする選択肢を用意しているただけで、原則、環境変数をセットせず、デフォルト値を用いる。
pub(crate) static MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE: Lazy<u32> =
    Lazy::new(|| {
        let min_duration_in_hour = var(KEY_TO_MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE)
            .unwrap_or_else(|_| "6".to_string());
        min_duration_in_hour
            .parse()
            .expect("failed to parse MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE")
    });

pub(crate) const KEY_TO_FIRST_START_HOUR_OF_CONSULTATION: &str = "FIRST_START_HOUR_OF_CONSULTATION";
/// 1日の内、最も早い相談開始時刻
///
/// 動作確認時に待機時間を減らすために環境変数をセットする選択肢を用意しているただけで、原則、環境変数をセットせず、デフォルト値を用いる。
pub(crate) static FIRST_START_HOUR_OF_CONSULTATION: Lazy<u32> = Lazy::new(|| {
    let first_start_hour =
        var(KEY_TO_FIRST_START_HOUR_OF_CONSULTATION).unwrap_or_else(|_| "7".to_string());
    let first_start_hour = first_start_hour
        .parse()
        .expect("failed to parse FIRST_START_HOUR_OF_CONSULTATION");
    if !(0..=23).contains(&first_start_hour) {
        panic!(
            "FIRST_START_HOUR_OF_CONSULTATION must be between 0 to 23: {}",
            first_start_hour
        );
    };
    first_start_hour
});

pub(crate) const KEY_TO_LAST_START_HOUR_OF_CONSULTATION: &str = "LAST_START_HOUR_OF_CONSULTATION";
/// 1日の内、最も遅い相談開始時刻
///
/// 動作確認時に待機時間を減らすために環境変数をセットする選択肢を用意しているただけで、原則、環境変数をセットせず、デフォルト値を用いる。
pub(crate) static LAST_START_HOUR_OF_CONSULTATION: Lazy<u32> = Lazy::new(|| {
    let last_start_hour =
        var(KEY_TO_LAST_START_HOUR_OF_CONSULTATION).unwrap_or_else(|_| "23".to_string());
    let last_start_hour = last_start_hour
        .parse()
        .expect("failed to parse LAST_START_HOUR_OF_CONSULTATION");
    if !(0..=23).contains(&last_start_hour) {
        panic!(
            "LAST_START_HOUR_OF_CONSULTATION must be between 0 to 23: {}",
            last_start_hour
        );
    };
    if last_start_hour <= *FIRST_START_HOUR_OF_CONSULTATION {
        panic!("LAST_START_HOUR_OF_CONSULTATION ({}) must be more than FIRST_START_HOUR_OF_CONSULTATION ({})", last_start_hour, *FIRST_START_HOUR_OF_CONSULTATION);
    };
    last_start_hour
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

/// 渡された西暦に対して、その年の日本時間における1月1日0時0分0秒と12月31日23時59分59秒のタイムスタンプを返す
pub(crate) fn create_start_and_end_timestamps_of_current_year(current_year: i32) -> (i64, i64) {
    let start_timestamp = JAPANESE_TIME_ZONE
        .ymd(current_year, 1, 1)
        .and_hms(0, 0, 0)
        .timestamp();

    let end_timestamp = JAPANESE_TIME_ZONE
        .ymd(current_year, 12, 31)
        .and_hms(23, 59, 59)
        .timestamp();

    (start_timestamp, end_timestamp)
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

/// 通常のテストコードに加え、共通で使うモックをまとめる
#[cfg(test)]
pub(crate) mod tests {
    use std::io::Cursor;

    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use bytes::Bytes;
    use chrono::TimeZone;
    use common::JAPANESE_TIME_ZONE;
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
        create_start_and_end_timestamps_of_current_year,
    };

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

    #[test]
    fn test_case_normal_year_create_start_and_end_timestamps_of_current_year() {
        let (since_timestamp, until_timestamp) =
            create_start_and_end_timestamps_of_current_year(2022);
        assert_eq!(
            JAPANESE_TIME_ZONE
                .ymd(2022, 1, 1)
                .and_hms(0, 0, 0)
                .timestamp(),
            since_timestamp
        );
        assert_eq!(
            JAPANESE_TIME_ZONE
                .ymd(2022, 12, 31)
                .and_hms(23, 59, 59)
                .timestamp(),
            until_timestamp
        );
    }

    #[test]
    fn test_case_leap_year_create_start_and_end_timestamps_of_current_year() {
        let (since_timestamp, until_timestamp) =
            create_start_and_end_timestamps_of_current_year(2020);
        assert_eq!(
            JAPANESE_TIME_ZONE
                .ymd(2020, 1, 1)
                .and_hms(0, 0, 0)
                .timestamp(),
            since_timestamp
        );
        assert_eq!(
            JAPANESE_TIME_ZONE
                .ymd(2020, 12, 31)
                .and_hms(23, 59, 59)
                .timestamp(),
            until_timestamp
        );
    }

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: ErrResp,
    }

    #[derive(Debug)]
    struct Input {
        err: common::payment_platform::Error,
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        // ErrorDetailのメンバーは、実際に返ってくる値が不明なため使う値のみ正確に埋める。
        // pay.jpを使う中で実際に正確な値がわかった場合、随時更新していく。
        vec![
            TestCase {
                name: "status 402".to_string(),
                input: Input {
                    err: common::payment_platform::Error::ApiError(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 402,
                            r#type: "type".to_string(),
                            code: None,
                            param: None,
                            charge: None,
                        },
                    }),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::CardAuthPaymentError as u32,
                    }),
                ),
            },
            TestCase {
                name: "status 429".to_string(),
                input: Input {
                    err: common::payment_platform::Error::ApiError(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 429,
                            r#type: "type".to_string(),
                            code: None,
                            param: None,
                            charge: None,
                        },
                    }),
                },
                expected: (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ApiError {
                        code: Code::ReachPaymentPlatformRateLimit as u32,
                    }),
                ),
            },
            TestCase {
                name: "code incorrect_card_data".to_string(),
                input: Input {
                    err: common::payment_platform::Error::ApiError(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("incorrect_card_data".to_string()),
                            param: None,
                            charge: None,
                        },
                    }),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeIncorrectCardData as u32,
                    }),
                ),
            },
            TestCase {
                name: "code card_declined".to_string(),
                input: Input {
                    err: common::payment_platform::Error::ApiError(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("card_declined".to_string()),
                            param: None,
                            charge: None,
                        },
                    }),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeCardDeclined as u32,
                    }),
                ),
            },
            TestCase {
                name: "code card_flagged".to_string(),
                input: Input {
                    err: common::payment_platform::Error::ApiError(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("card_flagged".to_string()),
                            param: None,
                            charge: None,
                        },
                    }),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeCardFlagged as u32,
                    }),
                ),
            },
            TestCase {
                name: "code unacceptable_brand".to_string(),
                input: Input {
                    err: common::payment_platform::Error::ApiError(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("unacceptable_brand".to_string()),
                            param: None,
                            charge: None,
                        },
                    }),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeUnacceptableBrand as u32,
                    }),
                ),
            },
            TestCase {
                name: "code over_capacity".to_string(),
                input: Input {
                    err: common::payment_platform::Error::ApiError(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("over_capacity".to_string()),
                            param: None,
                            charge: None,
                        },
                    }),
                },
                expected: (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ApiError {
                        code: Code::ReachPaymentPlatformRateLimit as u32,
                    }),
                ),
            },
            TestCase {
                name: "code three_d_secure_incompleted".to_string(),
                input: Input {
                    err: common::payment_platform::Error::ApiError(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("three_d_secure_incompleted".to_string()),
                            param: None,
                            charge: None,
                        },
                    }),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeThreeDSecureIncompleted as u32,
                    }),
                ),
            },
            TestCase {
                name: "code three_d_secure_failed".to_string(),
                input: Input {
                    err: common::payment_platform::Error::ApiError(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("three_d_secure_failed".to_string()),
                            param: None,
                            charge: None,
                        },
                    }),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeThreeDSecureFailed as u32,
                    }),
                ),
            },
            TestCase {
                name: "code not_in_three_d_secure_flow".to_string(),
                input: Input {
                    err: common::payment_platform::Error::ApiError(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("not_in_three_d_secure_flow".to_string()),
                            param: None,
                            charge: None,
                        },
                    }),
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
        for test_case in TEST_CASE_SET.iter() {
            let err_resp = convert_payment_err_to_err_resp(&test_case.input.err);
            let message = format!("test case \"{}\" failed", test_case.name.clone());
            assert_eq!(test_case.expected.0, err_resp.0, "{}", message);
            assert_eq!(test_case.expected.1 .0, err_resp.1 .0, "{}", message);
        }
    }
}
