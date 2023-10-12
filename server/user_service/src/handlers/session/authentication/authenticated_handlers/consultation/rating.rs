// Copyright 2023 Ken Miura

use axum::{http::StatusCode, Json};
use chrono::{DateTime, Duration, FixedOffset};
use common::{ApiError, ErrResp, JAPANESE_TIME_ZONE};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use tracing::error;

use crate::{
    err::{unexpected_err_resp, Code},
    handlers::session::authentication::LENGTH_OF_MEETING_IN_MINUTE,
};

pub(crate) mod consultant_rating;
pub(crate) mod unrated_items;
pub(crate) mod user_rating;

#[derive(Clone, Debug)]
struct ConsultationInfo {
    user_account_id: i64,
    consultant_id: i64,
    consultation_date_time_in_jst: DateTime<FixedOffset>,
}

async fn find_consultation_info(
    pool: &DatabaseConnection,
    consultation_id: i64,
) -> Result<Option<ConsultationInfo>, ErrResp> {
    let model_option = entity::consultation::Entity::find_by_id(consultation_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find consultation (consultation_id: {}): {}",
                consultation_id, e
            );
            unexpected_err_resp()
        })?;
    Ok(model_option.map(|m| ConsultationInfo {
        user_account_id: m.user_account_id,
        consultant_id: m.consultant_id,
        consultation_date_time_in_jst: m.meeting_at.with_timezone(&(*JAPANESE_TIME_ZONE)),
    }))
}

const MIN_RATING: i16 = 1;
const MAX_RATING: i16 = 5;

fn ensure_rating_is_in_valid_range(rating: i16) -> Result<(), ErrResp> {
    if !(MIN_RATING..=MAX_RATING).contains(&rating) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidRating as u32,
            }),
        ));
    }
    Ok(())
}

fn ensure_end_of_consultation_date_time_has_passed(
    consultation_date_time: &DateTime<FixedOffset>,
    current_date_time: &DateTime<FixedOffset>,
) -> Result<(), ErrResp> {
    let criteria = *consultation_date_time + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64);
    if *current_date_time <= criteria {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::EndOfConsultationDateTimeHasNotPassedYet as u32,
            }),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;
    use common::JAPANESE_TIME_ZONE;

    use super::*;

    #[test]
    fn test_succsess_lower_bound_ensure_rating_is_in_valid_range() {
        ensure_rating_is_in_valid_range(MIN_RATING).expect("failed to get Ok");
    }

    #[test]
    fn test_succsess_upper_bound_ensure_rating_is_in_valid_range() {
        ensure_rating_is_in_valid_range(MAX_RATING).expect("failed to get Ok");
    }

    #[test]
    fn test_fail_under_lower_bound_ensure_rating_is_in_valid_range() {
        let result =
            ensure_rating_is_in_valid_range(MIN_RATING - 1).expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(
            result.1 .0,
            ApiError {
                code: Code::InvalidRating as u32
            }
        );
    }

    #[test]
    fn test_fail_over_upper_bound_ensure_rating_is_in_valid_range() {
        let result =
            ensure_rating_is_in_valid_range(MAX_RATING + 1).expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(
            result.1 .0,
            ApiError {
                code: Code::InvalidRating as u32
            }
        );
    }

    #[test]
    fn test_succsess_ensure_end_of_consultation_date_time_has_passed() {
        let consultation_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 5, 15, 0, 0)
            .unwrap();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 5, 16, 0, 1)
            .unwrap();

        ensure_end_of_consultation_date_time_has_passed(
            &consultation_date_time,
            &current_date_time,
        )
        .expect("failed to get Ok");
    }

    #[test]
    fn test_fail_same_as_end_end_of_consultation_date_time_ensure_end_of_consultation_date_time_has_passed(
    ) {
        let consultation_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 5, 15, 0, 0)
            .unwrap();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 5, 16, 0, 0)
            .unwrap();

        let result = ensure_end_of_consultation_date_time_has_passed(
            &consultation_date_time,
            &current_date_time,
        )
        .expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(
            result.1 .0,
            ApiError {
                code: Code::EndOfConsultationDateTimeHasNotPassedYet as u32
            }
        );
    }

    #[test]
    fn test_fail_not_over_yet_ensure_end_of_consultation_date_time_has_passed() {
        let consultation_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 5, 15, 0, 0)
            .unwrap();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 3, 5, 15, 59, 59)
            .unwrap();

        let result = ensure_end_of_consultation_date_time_has_passed(
            &consultation_date_time,
            &current_date_time,
        )
        .expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(
            result.1 .0,
            ApiError {
                code: Code::EndOfConsultationDateTimeHasNotPassedYet as u32
            }
        );
    }
}
