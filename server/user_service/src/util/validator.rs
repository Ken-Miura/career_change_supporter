// Copyright 2021 Ken Miura

use std::{error::Error, fmt::Display};

use chrono::{Datelike, NaiveDate};

use super::{Identity, Ymd};

const LAST_NAME_MIN_LENGTH: usize = 1;
const LAST_NAME_MAX_LENGTH: usize = 128;
const FIRST_NAME_MIN_LENGTH: usize = 1;
const FIRST_NAME_MAX_LENGTH: usize = 128;
const LAST_NAME_FURIGANA_MIN_LENGTH: usize = 1;
const LAST_NAME_FURIGANA_MAX_LENGTH: usize = 128;
const FIRST_NAME_FURIGANA_MIN_LENGTH: usize = 1;
const FIRST_NAME_FURIGANA_MAX_LENGTH: usize = 128;
const MIN_AGE_REQUIREMENT: i32 = 18;

pub(crate) fn validate_identity(
    identity: &Identity,
    current_date: &NaiveDate,
) -> Result<(), IdentityValidationError> {
    let _ = validate_last_name(&identity.last_name)?;
    let _ = validate_first_name(&identity.first_name)?;
    let _ = validate_last_name_furigana(&identity.last_name_furigana)?;
    let _ = validate_first_name_furigana(&identity.first_name_furigana)?;
    let _ = validate_date_of_birth(&identity.date_of_birth, current_date)?;
    Ok(())
}

fn validate_last_name(last_name: &str) -> Result<(), IdentityValidationError> {
    let last_name_length = last_name.len();
    if !(LAST_NAME_MIN_LENGTH..=LAST_NAME_MAX_LENGTH).contains(&last_name_length) {
        return Err(IdentityValidationError::InvalidLastNameLength {
            length: last_name_length,
            min_length: LAST_NAME_MIN_LENGTH,
            max_length: LAST_NAME_MAX_LENGTH,
        });
    }
    if has_control_char(last_name) {
        return Err(IdentityValidationError::IllegalCharInLastName(
            last_name.to_string(),
        ));
    }
    Ok(())
}

fn validate_first_name(first_name: &str) -> Result<(), IdentityValidationError> {
    let first_name_length = first_name.len();
    if !(FIRST_NAME_MIN_LENGTH..=FIRST_NAME_MAX_LENGTH).contains(&first_name_length) {
        return Err(IdentityValidationError::InvalidFirstNameLength {
            length: first_name_length,
            min_length: LAST_NAME_MIN_LENGTH,
            max_length: LAST_NAME_MAX_LENGTH,
        });
    }
    if has_control_char(first_name) {
        return Err(IdentityValidationError::IllegalCharInFirstName(
            first_name.to_string(),
        ));
    }
    Ok(())
}

fn validate_last_name_furigana(last_name_furigana: &str) -> Result<(), IdentityValidationError> {
    let last_name_furigana_length = last_name_furigana.len();
    if !(LAST_NAME_FURIGANA_MIN_LENGTH..=LAST_NAME_FURIGANA_MAX_LENGTH)
        .contains(&last_name_furigana_length)
    {
        return Err(IdentityValidationError::InvalidLastNameFuriganaLength {
            length: last_name_furigana_length,
            min_length: LAST_NAME_FURIGANA_MIN_LENGTH,
            max_length: LAST_NAME_FURIGANA_MAX_LENGTH,
        });
    }
    if has_control_char(last_name_furigana) {
        return Err(IdentityValidationError::IllegalCharInLastNameFurigana(
            last_name_furigana.to_string(),
        ));
    }
    Ok(())
}

fn validate_first_name_furigana(first_name_furigana: &str) -> Result<(), IdentityValidationError> {
    let first_name_furigana_length = first_name_furigana.len();
    if !(FIRST_NAME_FURIGANA_MIN_LENGTH..=FIRST_NAME_FURIGANA_MAX_LENGTH)
        .contains(&first_name_furigana_length)
    {
        return Err(IdentityValidationError::InvalidFirstNameFuriganaLength {
            length: first_name_furigana_length,
            min_length: LAST_NAME_MIN_LENGTH,
            max_length: LAST_NAME_MAX_LENGTH,
        });
    }
    if has_control_char(first_name_furigana) {
        return Err(IdentityValidationError::IllegalCharInFirstNameFurigana(
            first_name_furigana.to_string(),
        ));
    }
    Ok(())
}

fn has_control_char(s: &str) -> bool {
    let characters = s.chars().collect::<Vec<char>>();
    for c in characters {
        if c.is_control() {
            return true;
        }
    }
    return false;
}

fn validate_date_of_birth(
    date_of_birth: &Ymd,
    current_date: &NaiveDate,
) -> Result<(), IdentityValidationError> {
    match NaiveDate::from_ymd_opt(date_of_birth.year, date_of_birth.month, date_of_birth.day) {
        Some(ymd) => {
            let _ = validate_age_satisfies_min_age_requirement(&ymd, current_date)?;
        }
        None => {
            return Err(IdentityValidationError::IllegalDate {
                year: date_of_birth.year,
                month: date_of_birth.month,
                day: date_of_birth.day,
            })
        }
    };
    Ok(())
}

fn validate_age_satisfies_min_age_requirement(
    date_of_birth: &NaiveDate,
    current_date: &NaiveDate,
) -> Result<(), IdentityValidationError> {
    let year_diff = current_date.year() - date_of_birth.year();
    if year_diff > MIN_AGE_REQUIREMENT {
        return Ok(());
    } else if year_diff == MIN_AGE_REQUIREMENT {
        return validate_current_day_passes_birthday(date_of_birth, current_date);
    } else {
        return Err(IdentityValidationError::IllegalAge {
            birth_year: date_of_birth.year(),
            birth_month: date_of_birth.month(),
            birth_day: date_of_birth.day(),
            current_year: current_date.year(),
            current_month: current_date.month(),
            current_day: current_date.day(),
        });
    };
}

fn validate_current_day_passes_birthday(
    date_of_birth: &NaiveDate,
    current_date: &NaiveDate,
) -> Result<(), IdentityValidationError> {
    if current_date.month() > date_of_birth.month() {
        return Ok(());
    } else if current_date.month() == date_of_birth.month() {
        if current_date.day() >= date_of_birth.day() {
            return Ok(());
        } else {
            return Err(IdentityValidationError::IllegalAge {
                birth_year: date_of_birth.year(),
                birth_month: date_of_birth.month(),
                birth_day: date_of_birth.day(),
                current_year: current_date.year(),
                current_month: current_date.month(),
                current_day: current_date.day(),
            });
        }
    } else {
        return Err(IdentityValidationError::IllegalAge {
            birth_year: date_of_birth.year(),
            birth_month: date_of_birth.month(),
            birth_day: date_of_birth.day(),
            current_year: current_date.year(),
            current_month: current_date.month(),
            current_day: current_date.day(),
        });
    }
}

/// Error related to [validate_identity()]
#[derive(Debug)]
pub(crate) enum IdentityValidationError {
    InvalidLastNameLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInLastName(String),
    InvalidFirstNameLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInFirstName(String),
    InvalidLastNameFuriganaLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInLastNameFurigana(String),
    InvalidFirstNameFuriganaLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInFirstNameFurigana(String),
    IllegalDate {
        year: i32,
        month: u32,
        day: u32,
    },
    IllegalAge {
        birth_year: i32,
        birth_month: u32,
        birth_day: u32,
        current_year: i32,
        current_month: u32,
        current_day: u32,
    },
}

impl Display for IdentityValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
        // match self {
        //     IdentityValidationError::InvalidLastNameLength {
        //         length,
        //         min_length,
        //         max_length,
        //     } => todo!(),
        //     IdentityValidationError::IllegalCharInLastName(last_name) => {
        //         write!(f, "illegal charcter included: {:X?}", last_name.as_bytes().to_vec())
        //     }
        // }
    }
}

impl Error for IdentityValidationError {}
