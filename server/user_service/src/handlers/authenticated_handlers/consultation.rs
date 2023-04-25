// Copyright 2023 Ken Miura

pub(crate) mod consultant;
pub(crate) mod consultation_request;
pub(crate) mod consultation_room;
pub(crate) mod consultations;
pub(crate) mod rating;
pub(crate) mod request_consultation;

use crate::util::user_info::FindUserInfoOperation;

use axum::{http::StatusCode, Json};
use chrono::{DateTime, FixedOffset};
use common::{ApiError, ErrResp, JAPANESE_TIME_ZONE};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
struct ConsultationDateTime {
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
}

/// コンサルタントのアカウントが利用可能か確認する。
/// アカウントが存在し、かつ無効化されていない（=利用可能な）場合、trueを返す。そうでない場合、falseを返す。
async fn check_if_consultant_is_available(
    account_id: i64,
    op: &impl FindUserInfoOperation,
) -> Result<bool, ErrResp> {
    let user_info = op.find_user_info_by_account_id(account_id).await?;
    if let Some(u) = user_info {
        if u.disabled_at.is_some() {
            Ok(false)
        } else {
            Ok(true)
        }
    } else {
        Ok(false)
    }
}

/// 相談申し込み
#[derive(Clone, Debug)]
struct ConsultationRequest {
    consultation_req_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    fee_per_hour_in_yen: i32,
    first_candidate_date_time_in_jst: DateTime<FixedOffset>,
    second_candidate_date_time_in_jst: DateTime<FixedOffset>,
    third_candidate_date_time_in_jst: DateTime<FixedOffset>,
    charge_id: String,
    latest_candidate_date_time_in_jst: DateTime<FixedOffset>,
}

/// 相談申し込みを取得する
///
/// 取得した相談申し込みは、consultant_idがリクエスト送信元のユーザーIDと一致するか（操作可能なユーザーか）必ずチェックする
async fn find_consultation_req_by_consultation_req_id(
    pool: &DatabaseConnection,
    consultation_req_id: i64,
) -> Result<Option<ConsultationRequest>, ErrResp> {
    let model = entity::prelude::ConsultationReq::find_by_id(consultation_req_id)
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
fn consultation_req_exists(
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

/// 小数点以下2桁目を四捨五入し、小数点以下1桁目までを示す少数を文字列表現として返す。
fn round_to_one_decimal_places(rating: f64) -> String {
    let result = (rating * 10.0).round() / 10.0;
    // format!("{:.1}", rating) のみで少数点以下2桁目を四捨五入し、小数点以下1桁まで求める動作となる。
    // しかし、下記のドキュメントに、その動作（四捨五入）に関して正式な仕様として記載がないため、四捨五入の箇所は自身で実装する。
    // https://doc.rust-lang.org/std/fmt/
    format!("{:.1}", result)
}

#[cfg(test)]
mod tests {

    use axum::async_trait;
    use chrono::TimeZone;
    use common::{ErrResp, JAPANESE_TIME_ZONE};
    use once_cell::sync::Lazy;

    use crate::{
        handlers::authenticated_handlers::consultation::{
            check_if_consultant_is_available, round_to_one_decimal_places,
        },
        util::user_info::{FindUserInfoOperation, UserInfo},
    };

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
    async fn test_check_if_user_account_is_available_returns_false_when_no_user_is_found() {
        let user_info = UserInfo {
            account_id: 6051,
            email_address: "test@test.com".to_string(),
            mfa_enabled_at: None,
            disabled_at: None,
        };
        let op = FindUserInfoOperationMock {
            user_info: &user_info,
        };
        let other_account_id = user_info.account_id + 6501;

        let ret = check_if_consultant_is_available(other_account_id, &op)
            .await
            .expect("failed to get Ok");

        assert!(!ret);
    }

    #[tokio::test]
    async fn test_check_if_user_account_is_available_returns_false_when_user_is_disabled() {
        let user_info = UserInfo {
            account_id: 6051,
            email_address: "test@test.com".to_string(),
            mfa_enabled_at: None,
            disabled_at: Some(
                JAPANESE_TIME_ZONE
                    .with_ymd_and_hms(2022, 1, 3, 23, 59, 59)
                    .unwrap(),
            ),
        };
        let op = FindUserInfoOperationMock {
            user_info: &user_info,
        };

        let ret = check_if_consultant_is_available(user_info.account_id, &op)
            .await
            .expect("failed to get Ok");

        assert!(!ret);
    }

    #[tokio::test]
    async fn test_check_if_user_account_is_available_returns_true_when_user_is_not_disabled() {
        let user_info = UserInfo {
            account_id: 6051,
            email_address: "test@test.com".to_string(),
            mfa_enabled_at: None,
            disabled_at: None,
        };
        let op = FindUserInfoOperationMock {
            user_info: &user_info,
        };

        let ret = check_if_consultant_is_available(user_info.account_id, &op)
            .await
            .expect("failed to get Ok");

        assert!(ret);
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
