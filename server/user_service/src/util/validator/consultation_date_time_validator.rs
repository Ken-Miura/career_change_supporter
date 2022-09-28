// Copyright 2022 Ken Miura

use std::{error::Error, fmt::Display};

use chrono::{DateTime, FixedOffset, NaiveDate};

use crate::request_consultation::ConsultationDateTime;

pub(crate) fn validate_consultation_date_time(
    consultation_date_time: &ConsultationDateTime,
    current_date_time: &DateTime<FixedOffset>,
) -> Result<(), ConsultationDateTimeValidationError> {
    let year = consultation_date_time.year;
    let month = consultation_date_time.month;
    let day = consultation_date_time.day;
    let hour = consultation_date_time.hour;
    let date = match NaiveDate::from_ymd_opt(year, month, day) {
        Some(date) => date,
        None => {
            return Err(ConsultationDateTimeValidationError::IllegalDateTime {
                year,
                month,
                day,
                hour,
            })
        }
    };
    let date_time = match date.and_hms_opt(hour, 0, 0) {
        Some(date_time) => date_time,
        None => {
            return Err(ConsultationDateTimeValidationError::IllegalDateTime {
                year,
                month,
                day,
                hour,
            })
        }
    };

    let timezone = current_date_time.offset();
    let consultation_date_time = DateTime::<FixedOffset>::from_local(date_time, *timezone);
    todo!()
}

/// Error related to [validate_consultation_date_time()]
#[derive(Debug, PartialEq)]
pub(crate) enum ConsultationDateTimeValidationError {
    IllegalDateTime {
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
    },
}

impl Display for ConsultationDateTimeValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsultationDateTimeValidationError::IllegalDateTime {
                year,
                month,
                day,
                hour,
            } => write!(
                f,
                "illegal date time (year: {}, month: {}, day: {}, hour: {})",
                year, month, day, hour
            ),
        }
    }
}

impl Error for ConsultationDateTimeValidationError {}

#[cfg(test)]
mod tests {}
