// Copyright 2021 Ken Miura

use std::{error::Error, fmt::Display};

use common::util::validator::{has_control_char, SPACE_RE, SYMBOL_CHAR_RE};

pub(crate) const REASON_MIN_LENGTH: usize = 1;
pub(crate) const REASON_MAX_LENGTH: usize = 256;

pub(crate) fn validate_reason(reason: &str) -> Result<(), ReasonValidationError> {
    let reason_length = reason.chars().count();
    if !(REASON_MIN_LENGTH..=REASON_MAX_LENGTH).contains(&reason_length) {
        return Err(ReasonValidationError::InvalidReasonLength {
            length: reason_length,
            min_length: REASON_MIN_LENGTH,
            max_length: REASON_MAX_LENGTH,
        });
    }
    if has_control_char(reason) {
        return Err(ReasonValidationError::IllegalCharInReason(
            reason.to_string(),
        ));
    }
    if SYMBOL_CHAR_RE.is_match(reason) || SPACE_RE.is_match(reason) {
        return Err(ReasonValidationError::IllegalCharInReason(
            reason.to_string(),
        ));
    }
    Ok(())
}

/// Error related to [validate_reason()]
#[derive(Debug, PartialEq)]
pub(crate) enum ReasonValidationError {
    InvalidReasonLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    IllegalCharInReason(String),
}

impl Display for ReasonValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReasonValidationError::InvalidReasonLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid reason length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            ReasonValidationError::IllegalCharInReason(last_name) => {
                write!(
                    f,
                    "reason: illegal charcter included: {} (binary: {:X?})",
                    last_name,
                    last_name.as_bytes().to_vec()
                )
            }
        }
    }
}

impl Error for ReasonValidationError {}

#[cfg(test)]
mod tests {}
