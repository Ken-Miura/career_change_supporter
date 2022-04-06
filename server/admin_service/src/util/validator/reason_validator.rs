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
mod tests {
    use std::collections::HashSet;

    use once_cell::sync::Lazy;

    use crate::util::validator::reason_validator::{
        ReasonValidationError, REASON_MAX_LENGTH, REASON_MIN_LENGTH,
    };

    use super::validate_reason;

    static SYMBOL_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(32);
        set.insert("!".to_string());
        set.insert("\"".to_string());
        set.insert("#".to_string());
        set.insert("$".to_string());
        set.insert("%".to_string());
        set.insert("&".to_string());
        set.insert("'".to_string());
        set.insert("(".to_string());
        set.insert(")".to_string());
        set.insert("*".to_string());
        set.insert("+".to_string());
        set.insert(",".to_string());
        set.insert("-".to_string());
        set.insert(".".to_string());
        set.insert("/".to_string());
        set.insert(":".to_string());
        set.insert(";".to_string());
        set.insert("<".to_string());
        set.insert("=".to_string());
        set.insert(">".to_string());
        set.insert("?".to_string());
        set.insert("@".to_string());
        set.insert("[".to_string());
        set.insert("\\".to_string());
        set.insert("]".to_string());
        set.insert("^".to_string());
        set.insert("_".to_string());
        set.insert("`".to_string());
        set.insert("{".to_string());
        set.insert("|".to_string());
        set.insert("}".to_string());
        set.insert("~".to_string());
        set
    });

    static CONTROL_CHAR_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(32 + 1 + 32);
        // C0制御コード
        set.insert('\u{0000}'.to_string());
        set.insert('\u{0001}'.to_string());
        set.insert('\u{0002}'.to_string());
        set.insert('\u{0003}'.to_string());
        set.insert('\u{0004}'.to_string());
        set.insert('\u{0005}'.to_string());
        set.insert('\u{0006}'.to_string());
        set.insert('\u{0007}'.to_string());
        set.insert('\u{0008}'.to_string());
        set.insert('\u{0009}'.to_string());
        set.insert('\u{000A}'.to_string());
        set.insert('\u{000B}'.to_string());
        set.insert('\u{000C}'.to_string());
        set.insert('\u{000D}'.to_string());
        set.insert('\u{000E}'.to_string());
        set.insert('\u{000F}'.to_string());
        set.insert('\u{0010}'.to_string());
        set.insert('\u{0011}'.to_string());
        set.insert('\u{0012}'.to_string());
        set.insert('\u{0013}'.to_string());
        set.insert('\u{0014}'.to_string());
        set.insert('\u{0015}'.to_string());
        set.insert('\u{0016}'.to_string());
        set.insert('\u{0017}'.to_string());
        set.insert('\u{0018}'.to_string());
        set.insert('\u{0019}'.to_string());
        set.insert('\u{001A}'.to_string());
        set.insert('\u{001B}'.to_string());
        set.insert('\u{001C}'.to_string());
        set.insert('\u{001D}'.to_string());
        set.insert('\u{001E}'.to_string());
        set.insert('\u{001F}'.to_string());
        // 削除文字
        set.insert('\u{007F}'.to_string());
        // C1制御コード
        set.insert('\u{0080}'.to_string());
        set.insert('\u{0081}'.to_string());
        set.insert('\u{0082}'.to_string());
        set.insert('\u{0083}'.to_string());
        set.insert('\u{0084}'.to_string());
        set.insert('\u{0085}'.to_string());
        set.insert('\u{0086}'.to_string());
        set.insert('\u{0087}'.to_string());
        set.insert('\u{0088}'.to_string());
        set.insert('\u{0089}'.to_string());
        set.insert('\u{008A}'.to_string());
        set.insert('\u{008B}'.to_string());
        set.insert('\u{008C}'.to_string());
        set.insert('\u{008D}'.to_string());
        set.insert('\u{008E}'.to_string());
        set.insert('\u{008F}'.to_string());
        set.insert('\u{0090}'.to_string());
        set.insert('\u{0091}'.to_string());
        set.insert('\u{0092}'.to_string());
        set.insert('\u{0093}'.to_string());
        set.insert('\u{0094}'.to_string());
        set.insert('\u{0095}'.to_string());
        set.insert('\u{0096}'.to_string());
        set.insert('\u{0097}'.to_string());
        set.insert('\u{0098}'.to_string());
        set.insert('\u{0099}'.to_string());
        set.insert('\u{009A}'.to_string());
        set.insert('\u{009B}'.to_string());
        set.insert('\u{009C}'.to_string());
        set.insert('\u{009D}'.to_string());
        set.insert('\u{009E}'.to_string());
        set.insert('\u{009F}'.to_string());
        set
    });

    static SPACE_SET: Lazy<HashSet<String>> = Lazy::new(|| {
        let mut set: HashSet<String> = HashSet::with_capacity(2);
        // 半角スペース
        set.insert(" ".to_string());
        // 全角スペース
        set.insert("　".to_string());
        set
    });

    #[test]
    fn validate_reason_returns_ok_if_valid_str_is_passed() {
        let reason = "画像が不鮮明なため";
        let result = validate_reason(reason);
        let _ = result.expect("failed to get Ok");
    }

    #[test]
    fn validate_reason_returns_err_if_empty_str_is_passed() {
        let reason = "";
        let result = validate_reason(reason);
        let err = result.expect_err("failed to get Err");
        assert_eq!(
            ReasonValidationError::InvalidReasonLength {
                length: reason.len(),
                min_length: REASON_MIN_LENGTH,
                max_length: REASON_MAX_LENGTH
            },
            err
        );
    }

    #[test]
    fn validate_reason_returns_err_if_257_length_str_is_passed() {
        let reason = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        let result = validate_reason(reason);
        let err = result.expect_err("failed to get Err");
        assert_eq!(
            ReasonValidationError::InvalidReasonLength {
                length: reason.len(),
                min_length: REASON_MIN_LENGTH,
                max_length: REASON_MAX_LENGTH
            },
            err
        );
    }

    #[test]
    fn validate_reason_returns_err_if_space_is_included() {
        for s in SPACE_SET.iter() {
            let result = validate_reason(s);
            let err = result.expect_err("failed to get Err");
            assert_eq!(
                ReasonValidationError::IllegalCharInReason(s.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_reason_returns_err_if_symbol_is_included() {
        for s in SYMBOL_SET.iter() {
            let result = validate_reason(s);
            let err = result.expect_err("failed to get Err");
            assert_eq!(
                ReasonValidationError::IllegalCharInReason(s.to_string()),
                err
            );
        }
    }

    #[test]
    fn validate_reason_returns_err_if_control_char_is_included() {
        for s in CONTROL_CHAR_SET.iter() {
            let result = validate_reason(s);
            let err = result.expect_err("failed to get Err");
            assert_eq!(
                ReasonValidationError::IllegalCharInReason(s.to_string()),
                err
            );
        }
    }
}
