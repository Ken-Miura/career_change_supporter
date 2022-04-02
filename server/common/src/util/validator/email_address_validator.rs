// Copyright 2021 Ken Miura

use std::{error::Error, fmt::Display};

use once_cell::sync::Lazy;
use regex::Regex;

const EMAIL_ADDRESS_MIN_LENGTH: usize = 1;
const EMAIL_ADDRESS_MAX_LENGTH: usize = 254;
const EMAIL_ADDRESS_REGEXP: &str = r"^[a-zA-Z0-9.!#$%&''*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$";

static EMAIL_ADDR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(EMAIL_ADDRESS_REGEXP).expect("failed to compile email address regexp"));

/// Validate email address.
pub fn validate_email_address(email_address: &str) -> Result<(), EmailAddressValidationError> {
    let mail_addr_length = email_address.len();
    if !(EMAIL_ADDRESS_MIN_LENGTH..=EMAIL_ADDRESS_MAX_LENGTH).contains(&mail_addr_length) {
        return Err(EmailAddressValidationError::InvalidLength {
            length: mail_addr_length,
            min_length: EMAIL_ADDRESS_MIN_LENGTH,
            max_length: EMAIL_ADDRESS_MAX_LENGTH,
        });
    }
    if !EMAIL_ADDR_RE.is_match(email_address) {
        return Err(EmailAddressValidationError::InvalidFormat {
            email_address: email_address.to_string(),
        });
    }
    Ok(())
}

/// Error related to [validate_email_address()]
#[derive(Debug)]
pub enum EmailAddressValidationError {
    InvalidLength {
        length: usize,
        min_length: usize,
        max_length: usize,
    },
    InvalidFormat {
        email_address: String,
    },
}

impl Display for EmailAddressValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailAddressValidationError::InvalidLength {
                length,
                min_length,
                max_length,
            } => write!(
                f,
                "invalid email address length: {} (length must be {} or more, and {} or less)",
                length, min_length, max_length
            ),
            EmailAddressValidationError::InvalidFormat { email_address } => {
                write!(f, "invalid email address format: {}", email_address)
            }
        }
    }
}

impl Error for EmailAddressValidationError {}

#[cfg(test)]
mod tests {
    use super::{validate_email_address, EmailAddressValidationError};

    #[test]
    fn validate_email_address_returns_ok_if_given_valid_email_address_with_various_letters() {
        let valid_email_address =
            "The.quick.brown.fox.jumps.over.the.lazy.dog_0123456789@example.com";

        let result = validate_email_address(valid_email_address);

        assert!(
            result.is_ok(),
            "valid_email_address: {}",
            valid_email_address
        );
    }

    #[test]
    fn validate_email_address_returns_ok_if_given_valid_email_address_of_254_characters() {
        let valid_email_address =
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa@example.com";

        let result = validate_email_address(valid_email_address);

        assert!(
            result.is_ok(),
            "valid_email_address: {}",
            valid_email_address
        );
    }

    #[test]
    fn validate_email_address_returns_invalid_length_if_given_empty_string() {
        let empty_str = "";

        let result = validate_email_address(empty_str);

        let err = result.expect_err("failed to get Err");
        match err {
            EmailAddressValidationError::InvalidLength {
                length,
                min_length: _,
                max_length: _,
            } => assert!(length == 0, "length: {}", length),
            EmailAddressValidationError::InvalidFormat { email_address } => panic!(
                "EmailAddressValidationError::InvalidFormat {{ email_address: \"{}\" }}",
                email_address
            ),
        }
    }

    #[test]
    fn validate_email_address_returns_invalid_length_if_given_255_characters() {
        // valid format email address of 255 characters
        let too_long_email_address = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa@example.com";

        let result = validate_email_address(too_long_email_address);

        let err = result.expect_err("failed to get Err");
        match err {
            EmailAddressValidationError::InvalidLength {
                length,
                min_length: _,
                max_length: _,
            } => assert!(length == 255, "length: {}", length),
            EmailAddressValidationError::InvalidFormat { email_address } => panic!(
                "EmailAddressValidationError::InvalidFormat {{ email_address: \"{}\" }}",
                email_address
            ),
        }
    }

    #[test]
    fn validate_email_address_returns_invalid_format_if_given_one_letter() {
        let one_letter = "a";

        let result = validate_email_address(one_letter);

        let err = result.expect_err("failed to get Err");
        match err {
            EmailAddressValidationError::InvalidLength {
                length,
                min_length: _,
                max_length: _,
            } => panic!(
                "EmailAddressValidationError::InvalidLength: length: {}",
                length
            ),
            EmailAddressValidationError::InvalidFormat { email_address } => {
                assert_eq!(
                    one_letter, &email_address,
                    "email_address: {}",
                    email_address
                )
            }
        }
    }

    #[test]
    fn validate_email_address_returns_invalid_format_if_given_email_address_without_domain() {
        let email_address_without_domain = "test@";

        let result = validate_email_address(email_address_without_domain);

        let err = result.expect_err("failed to get Err");
        match err {
            EmailAddressValidationError::InvalidLength {
                length,
                min_length: _,
                max_length: _,
            } => panic!(
                "EmailAddressValidationError::InvalidLength: length: {}",
                length
            ),
            EmailAddressValidationError::InvalidFormat { email_address } => assert_eq!(
                email_address_without_domain, &email_address,
                "email_address: {}",
                email_address
            ),
        }
    }

    #[test]
    fn validate_email_address_returns_invalid_format_if_given_email_address_without_local_part() {
        let email_address_without_local_part = "@example.com";

        let result = validate_email_address(email_address_without_local_part);

        let err = result.expect_err("failed to get Err");
        match err {
            EmailAddressValidationError::InvalidLength {
                length,
                min_length: _,
                max_length: _,
            } => panic!(
                "EmailAddressValidationError::InvalidLength: length: {}",
                length
            ),
            EmailAddressValidationError::InvalidFormat { email_address } => assert_eq!(
                email_address_without_local_part, &email_address,
                "email_address: {}",
                email_address
            ),
        }
    }
}
