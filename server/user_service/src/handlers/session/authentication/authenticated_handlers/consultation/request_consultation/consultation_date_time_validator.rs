// Copyright 2022 Ken Miura

use std::{error::Error, fmt::Display};

use chrono::{DateTime, FixedOffset, NaiveDate};

use crate::{
    handlers::session::authentication::authenticated_handlers::consultation::ConsultationDateTime,
    optional_env_var::{
        FIRST_START_HOUR_OF_CONSULTATION, LAST_START_HOUR_OF_CONSULTATION,
        MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS, MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS,
    },
};

pub(super) fn validate_consultation_date_time(
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

    if !(*FIRST_START_HOUR_OF_CONSULTATION..=*LAST_START_HOUR_OF_CONSULTATION).contains(&hour) {
        return Err(ConsultationDateTimeValidationError::IllegalConsultationHour { hour });
    }

    let timezone = current_date_time.offset();
    let consultation_date_time = DateTime::<FixedOffset>::from_local(date_time, *timezone);
    let duration = consultation_date_time - *current_date_time;
    if !(*MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS
        ..=*MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS)
        .contains(&duration.num_seconds())
    {
        return Err(
            ConsultationDateTimeValidationError::InvalidConsultationDateTime {
                consultation_date_time,
                current_date_time: *current_date_time,
            },
        );
    }
    Ok(())
}

/// Error related to [validate_consultation_date_time()]
#[derive(Debug, PartialEq)]
pub(super) enum ConsultationDateTimeValidationError {
    IllegalDateTime {
        year: i32,
        month: u32,
        day: u32,
        hour: u32,
    },
    IllegalConsultationHour {
        hour: u32,
    },
    InvalidConsultationDateTime {
        consultation_date_time: DateTime<FixedOffset>,
        current_date_time: DateTime<FixedOffset>,
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
            ConsultationDateTimeValidationError::IllegalConsultationHour { hour } => write!(
              f,
              "illegal consultation hour (hour: {}, FIRST_START_HOUR_OF_CONSULTATION: {}, LAST_START_HOUR_OF_CONSULTATION: {}",
              hour, *FIRST_START_HOUR_OF_CONSULTATION, *LAST_START_HOUR_OF_CONSULTATION),
            ConsultationDateTimeValidationError::InvalidConsultationDateTime {
                consultation_date_time,
                current_date_time,
            } => write!(
              f,
              "illegal consultation date time (consultation_date_time: {}, current_date_time: {}, MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS: {}, MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS: {})",
              consultation_date_time, current_date_time, *MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS, *MAX_DURATION_BEFORE_CONSULTATION_IN_SECONDS
          ),
        }
    }
}

impl Error for ConsultationDateTimeValidationError {}

#[cfg(test)]
mod tests {

    use common::JAPANESE_TIME_ZONE;
    use once_cell::sync::Lazy;

    use super::*;

    struct TestCase {
        name: String,
        input: Input,
        expected: Result<(), ConsultationDateTimeValidationError>,
    }

    struct Input {
        consultation_date_time: ConsultationDateTime,
        current_date_time: DateTime<FixedOffset>,
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        vec![
            TestCase {
                name: "valid consultation date time 1".to_string(),
                input: Input {
                    consultation_date_time: ConsultationDateTime {
                        year: 2022,
                        month: 9,
                        day: 29,
                        hour: 7,
                    },
                    current_date_time: DateTime::<FixedOffset>::from_local(
                        NaiveDate::from_ymd_opt(2022, 9, 19)
                            .expect("failed to get NaiveDate")
                            .and_hms_opt(7, 0, 0)
                            .expect("failed to get NaiveDate"),
                        *JAPANESE_TIME_ZONE,
                    ),
                },
                expected: Ok(()),
            },
            TestCase {
                name: "valid consultation date time 2".to_string(),
                input: Input {
                    consultation_date_time: ConsultationDateTime {
                        year: 2022,
                        month: 9,
                        day: 22,
                        hour: 23,
                    },
                    current_date_time: DateTime::<FixedOffset>::from_local(
                        NaiveDate::from_ymd_opt(2022, 9, 1)
                            .expect("failed to get NaiveDate")
                            .and_hms_opt(23, 0, 0)
                            .expect("failed to get NaiveDate"),
                        *JAPANESE_TIME_ZONE,
                    ),
                },
                expected: Ok(()),
            },
            TestCase {
                name: "illegal date".to_string(),
                input: Input {
                    consultation_date_time: ConsultationDateTime {
                        year: 2022,
                        month: 9,
                        day: 31,
                        hour: 23,
                    },
                    current_date_time: DateTime::<FixedOffset>::from_local(
                        NaiveDate::from_ymd_opt(2022, 9, 19)
                            .expect("failed to get NaiveDate")
                            .and_hms_opt(23, 0, 0)
                            .expect("failed to get NaiveDate"),
                        *JAPANESE_TIME_ZONE,
                    ),
                },
                expected: Err(ConsultationDateTimeValidationError::IllegalDateTime {
                    year: 2022,
                    month: 9,
                    day: 31,
                    hour: 23,
                }),
            },
            TestCase {
                name: "illegal date time".to_string(),
                input: Input {
                    consultation_date_time: ConsultationDateTime {
                        year: 2022,
                        month: 9,
                        day: 30,
                        hour: 24,
                    },
                    current_date_time: DateTime::<FixedOffset>::from_local(
                        NaiveDate::from_ymd_opt(2022, 9, 19)
                            .expect("failed to get NaiveDate")
                            .and_hms_opt(23, 0, 0)
                            .expect("failed to get NaiveDate"),
                        *JAPANESE_TIME_ZONE,
                    ),
                },
                expected: Err(ConsultationDateTimeValidationError::IllegalDateTime {
                    year: 2022,
                    month: 9,
                    day: 30,
                    hour: 24,
                }),
            },
            TestCase {
                name: "illegal consultation hour 1".to_string(),
                input: Input {
                    consultation_date_time: ConsultationDateTime {
                        year: 2022,
                        month: 9,
                        day: 30,
                        hour: 6,
                    },
                    current_date_time: DateTime::<FixedOffset>::from_local(
                        NaiveDate::from_ymd_opt(2022, 9, 19)
                            .expect("failed to get NaiveDate")
                            .and_hms_opt(23, 0, 0)
                            .expect("failed to get NaiveDate"),
                        *JAPANESE_TIME_ZONE,
                    ),
                },
                expected: Err(
                    ConsultationDateTimeValidationError::IllegalConsultationHour { hour: 6 },
                ),
            },
            TestCase {
                name: "illegal consultation hour 2".to_string(),
                input: Input {
                    consultation_date_time: ConsultationDateTime {
                        year: 2022,
                        month: 9,
                        day: 30,
                        hour: 0,
                    },
                    current_date_time: DateTime::<FixedOffset>::from_local(
                        NaiveDate::from_ymd_opt(2022, 9, 19)
                            .expect("failed to get NaiveDate")
                            .and_hms_opt(23, 0, 0)
                            .expect("failed to get NaiveDate"),
                        *JAPANESE_TIME_ZONE,
                    ),
                },
                expected: Err(
                    ConsultationDateTimeValidationError::IllegalConsultationHour { hour: 0 },
                ),
            },
            TestCase {
                name: "illegal consultation date time 1".to_string(),
                input: Input {
                    consultation_date_time: ConsultationDateTime {
                        year: 2022,
                        month: 9,
                        day: 29,
                        hour: 7,
                    },
                    current_date_time: DateTime::<FixedOffset>::from_local(
                        NaiveDate::from_ymd_opt(2022, 9, 19)
                            .expect("failed to get NaiveDate")
                            .and_hms_opt(7, 0, 1)
                            .expect("failed to get NaiveDate"),
                        *JAPANESE_TIME_ZONE,
                    ),
                },
                expected: Err(
                    ConsultationDateTimeValidationError::InvalidConsultationDateTime {
                        consultation_date_time: DateTime::<FixedOffset>::from_local(
                            NaiveDate::from_ymd_opt(2022, 9, 29)
                                .expect("failed to get NaiveDate")
                                .and_hms_opt(7, 0, 0)
                                .expect("failed to get NaiveDate"),
                            *JAPANESE_TIME_ZONE,
                        ),
                        current_date_time: DateTime::<FixedOffset>::from_local(
                            NaiveDate::from_ymd_opt(2022, 9, 19)
                                .expect("failed to get NaiveDate")
                                .and_hms_opt(7, 0, 1)
                                .expect("failed to get NaiveDate"),
                            *JAPANESE_TIME_ZONE,
                        ),
                    },
                ),
            },
            TestCase {
                name: "illegal consultation date time 2".to_string(),
                input: Input {
                    consultation_date_time: ConsultationDateTime {
                        year: 2022,
                        month: 9,
                        day: 22,
                        hour: 23,
                    },
                    current_date_time: DateTime::<FixedOffset>::from_local(
                        NaiveDate::from_ymd_opt(2022, 9, 1)
                            .expect("failed to get NaiveDate")
                            .and_hms_opt(22, 59, 59)
                            .expect("failed to get NaiveDate"),
                        *JAPANESE_TIME_ZONE,
                    ),
                },
                expected: Err(
                    ConsultationDateTimeValidationError::InvalidConsultationDateTime {
                        consultation_date_time: DateTime::<FixedOffset>::from_local(
                            NaiveDate::from_ymd_opt(2022, 9, 22)
                                .expect("failed to get NaiveDate")
                                .and_hms_opt(23, 0, 0)
                                .expect("failed to get NaiveDate"),
                            *JAPANESE_TIME_ZONE,
                        ),
                        current_date_time: DateTime::<FixedOffset>::from_local(
                            NaiveDate::from_ymd_opt(2022, 9, 1)
                                .expect("failed to get NaiveDate")
                                .and_hms_opt(22, 59, 59)
                                .expect("failed to get NaiveDate"),
                            *JAPANESE_TIME_ZONE,
                        ),
                    },
                ),
            },
        ]
    });

    #[test]
    fn test_validate_consultation_date_time() {
        for test_case in TEST_CASE_SET.iter() {
            let result = validate_consultation_date_time(
                &test_case.input.consultation_date_time,
                &test_case.input.current_date_time,
            );
            let message = format!("test case \"{}\" failed", test_case.name.clone());
            assert_eq!(test_case.expected, result, "{}", message);
        }
    }
}
