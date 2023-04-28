// Copyright 2023 Ken Miura

use axum::{http::StatusCode, Json};
use chrono::{DateTime, Duration, FixedOffset};
use common::{ApiError, ErrResp};

use crate::{err::Code, handlers::session::LENGTH_OF_MEETING_IN_MINUTE};

pub(crate) mod consultant_rating;
pub(crate) mod unrated_items;
pub(crate) mod user_rating;

#[derive(Clone, Debug)]
struct ConsultationInfo {
    consultation_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    consultation_date_time_in_jst: DateTime<FixedOffset>,
}

fn ensure_rating_id_is_positive(rating_id: i64) -> Result<(), ErrResp> {
    if !rating_id.is_positive() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::RatingIdIsNotPositive as u32,
            }),
        ));
    }
    Ok(())
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
    fn test_succsess_ensure_rating_id_is_positive() {
        ensure_rating_id_is_positive(1).expect("failed to get Ok");
    }

    #[test]
    fn test_fail_zero_ensure_rating_id_is_positive() {
        let result = ensure_rating_id_is_positive(0).expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(
            result.1 .0,
            ApiError {
                code: Code::RatingIdIsNotPositive as u32
            }
        );
    }

    #[test]
    fn test_fail_negative_value_ensure_rating_id_is_positive() {
        let result = ensure_rating_id_is_positive(-1).expect_err("failed to get Err");

        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(
            result.1 .0,
            ApiError {
                code: Code::RatingIdIsNotPositive as u32
            }
        );
    }

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
