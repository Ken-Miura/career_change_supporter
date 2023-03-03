// Copyright 2023 Ken Miura

use axum::{http::StatusCode, Json};
use chrono::{DateTime, Duration, FixedOffset};
use common::{ApiError, ErrResp};

use crate::{err::Code, util::request_consultation::LENGTH_OF_MEETING_IN_MINUTE};

pub(crate) mod consultant_rating;
pub(crate) mod unrated_items;
pub(crate) mod user_rating;

fn ensure_rating_id_is_positive(rating_id: i64) -> Result<(), ErrResp> {
    if !rating_id.is_positive() {
        return Err((
            StatusCode::OK,
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
            StatusCode::OK,
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
            StatusCode::OK,
            Json(ApiError {
                code: Code::EndOfConsultationDateTimeHasNotPassedYet as u32,
            }),
        ));
    }
    Ok(())
}
