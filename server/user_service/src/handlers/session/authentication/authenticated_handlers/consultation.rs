// Copyright 2023 Ken Miura

pub(crate) mod consultant;
pub(crate) mod consultation_request;
pub(crate) mod consultation_room;
pub(crate) mod consultations;
mod convert_payment_err;
pub(crate) mod rating;
pub(crate) mod request_consultation;

use axum::{http::StatusCode, Json};
use chrono::{DateTime, FixedOffset};
use common::{ApiError, ErrResp, JAPANESE_TIME_ZONE};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    err::{unexpected_err_resp, Code},
    handlers::session::authentication::user_operation::{FindUserInfoOperation, UserInfo},
};

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
    let user_info_opion = find_user_info_if_available(account_id, op).await?;
    if user_info_opion.is_some() {
        return Ok(true);
    }
    Ok(false)
}

/// アカウントが存在し、かつ無効化されていない場合、UserInfoを返す
async fn find_user_info_if_available(
    account_id: i64,
    op: &impl FindUserInfoOperation,
) -> Result<Option<UserInfo>, ErrResp> {
    let user_info = op.find_user_info_by_account_id(account_id).await?;
    if let Some(u) = user_info {
        if u.disabled_at.is_some() {
            Ok(None)
        } else {
            Ok(Some(u))
        }
    } else {
        Ok(None)
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

fn validate_consultation_id_is_positive(consultation_id: i64) -> Result<(), ErrResp> {
    if !consultation_id.is_positive() {
        error!("consultation_id ({}) is not positive", consultation_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonPositiveConsultationId as u32,
            }),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use axum::async_trait;
    use chrono::TimeZone;

    use super::*;

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

    #[tokio::test]
    async fn test_find_user_info_if_available_returns_none_when_no_account_is_found() {
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

        let ret = find_user_info_if_available(other_account_id, &op)
            .await
            .expect("failed to get Ok");

        assert_eq!(ret, None);
    }

    #[tokio::test]
    async fn test_find_user_info_if_available_returns_none_when_user_is_disabled() {
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

        let ret = find_user_info_if_available(user_info.account_id, &op)
            .await
            .expect("failed to get Ok");

        assert_eq!(ret, None);
    }

    #[tokio::test]
    async fn test_find_user_info_if_available_returns_user_info_when_user_is_not_disabled() {
        let user_info = UserInfo {
            account_id: 6051,
            email_address: "test@test.com".to_string(),
            mfa_enabled_at: None,
            disabled_at: None,
        };
        let op = FindUserInfoOperationMock {
            user_info: &user_info,
        };

        let ret = find_user_info_if_available(user_info.account_id, &op)
            .await
            .expect("failed to get Ok");

        assert_eq!(ret, Some(user_info));
    }

    #[test]
    fn test_validate_consultation_id_is_positive_success() {
        let consultation_id = 1;

        let result = validate_consultation_id_is_positive(consultation_id);

        result.expect("failed to get Ok");
    }

    #[test]
    fn test_validate_consultation_id_is_positive_fail1() {
        let consultation_id = 0;

        let result = validate_consultation_id_is_positive(consultation_id);

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::NonPositiveConsultationId as u32, resp.1.code);
    }

    #[test]
    fn test_validate_consultation_id_is_positive_fail2() {
        let consultation_id = -1;

        let result = validate_consultation_id_is_positive(consultation_id);

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::NonPositiveConsultationId as u32, resp.1.code);
    }
}
