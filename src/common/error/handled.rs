// Copyright 2021 Ken Miura

use derive_more::Display;

// TODO: どのようにグループ分けするか検討する
#[derive(Display, Debug)]
pub(crate) enum Error {
    // ----credential.rs----
    #[display(fmt = "{}", _0)]
    InvalidEmailAddressLength(InvalidEmailAddressLength),

    #[display(fmt = "{}", _0)]
    InvalidEmailAddressFormat(InvalidEmailAddressFormat),

    #[display(fmt = "{}", _0)]
    InvalidPasswordLength(InvalidPasswordLength),

    #[display(fmt = "{}", _0)]
    InvalidPasswordFormat(InvalidPasswordFormat),

    #[display(fmt = "{}", _0)]
    PasswordConstraintsViolation(PasswordConstraintsViolation),

    #[display(fmt = "{}", _0)]
    PasswordNotMatch(PasswordNotMatch),
    // --------------------

    // ----account.rs------
    #[display(fmt = "{}", _0)]
    AccountAlreadyExists(AccountAlreadyExists),

    #[display(fmt = "{}", _0)]
    ReachLimitOfTemporaryAccount(ReachLimitOfTemporaryAccount),

    #[display(fmt = "{}", _0)]
    NoTemporaryAccountFound(NoTemporaryAccountFound),

    #[display(fmt = "{}", _0)]
    TemporaryAccountExpired(TemporaryAccountExpired),

    #[display(fmt = "{}", _0)]
    InvalidTemporaryAccountId(InvalidTemporaryAccountId),
    // --------------------

    // ----authentication.rs------
    #[display(fmt = "{}", _0)]
    NoAccountFound(NoAccountFound),

    #[display(fmt = "{}", _0)]
    NoSessionFound(NoSessionFound),
    // --------------------
}

// TODO: 番号の順番を整理する
// NOTE: Use positive value because negative value is used for unexpected error
const EMAIL_FORMAT_INVALID_LENGTH: i32 = 1;
const EMAIL_FORMAT_INVALID_FORMAT: i32 = 2;
const PASSWORD_FORMAT_INVALID_LENGTH: i32 = 3;
const PASSWORD_FORMAT_INVALID_FORMAT: i32 = 4;
const PASSWORD_CONSTRAINTS_VIOLATION: i32 = 5;
const AUTHENTICATION_FAILED: i32 = 6;
const ACCOUNT_ALREADY_EXISTS: i32 = 7;
const REACH_LIMIT_OF_TEMPORARY_ACCOUNT: i32 = 8;
const NO_TEMPORARY_ACCOUNT_FOUND: i32 = 9;
const TEMPORARY_ACCOUNT_EXPIRED: i32 = 10;
const INVALID_TEMPORARY_ACCOUNT_ID: i32 = 11;
const NO_SESSION_FOUND: i32 = 12;

#[derive(Display, Debug)]
#[display(
    fmt = "invalid email address length (code: {}, length: {}, min_length: {}, max_length: {})",
    code,
    length,
    min_length,
    max_length
)]
pub(crate) struct InvalidEmailAddressLength {
    pub(super) code: i32,
    pub(super) length: usize,
    pub(super) min_length: usize,
    pub(super) max_length: usize,
}

impl InvalidEmailAddressLength {
    pub(crate) fn new(length: usize, min_length: usize, max_length: usize) -> Self {
        InvalidEmailAddressLength {
            code: EMAIL_FORMAT_INVALID_LENGTH,
            length,
            min_length,
            max_length,
        }
    }
}

#[derive(Display, Debug)]
#[display(
    fmt = "invalid email address format (code: {}, email_address: {})",
    code,
    email_address
)]
pub(crate) struct InvalidEmailAddressFormat {
    pub(super) code: i32,
    pub(super) email_address: String,
}

impl InvalidEmailAddressFormat {
    pub(crate) fn new(email_address: String) -> Self {
        InvalidEmailAddressFormat {
            code: EMAIL_FORMAT_INVALID_FORMAT,
            email_address,
        }
    }
}

#[derive(Display, Debug)]
#[display(
    fmt = "invalid password length (code: {}, min_length: {}, max_length: {})",
    code,
    min_length,
    max_length
)]
pub(crate) struct InvalidPasswordLength {
    pub(super) code: i32,
    pub(super) min_length: usize,
    pub(super) max_length: usize,
}

impl InvalidPasswordLength {
    pub(crate) fn new(min_length: usize, max_length: usize) -> Self {
        InvalidPasswordLength {
            code: PASSWORD_FORMAT_INVALID_LENGTH,
            min_length,
            max_length,
        }
    }
}

#[derive(Display, Debug)]
#[display(fmt = "invalid password format (code: {})", code)]
pub(crate) struct InvalidPasswordFormat {
    pub(super) code: i32,
}

impl InvalidPasswordFormat {
    pub(crate) fn new() -> Self {
        InvalidPasswordFormat {
            code: PASSWORD_FORMAT_INVALID_FORMAT,
        }
    }
}

#[derive(Display, Debug)]
#[display(fmt = "password constraints violation (code: {})", code)]
pub(crate) struct PasswordConstraintsViolation {
    pub(super) code: i32,
}

impl PasswordConstraintsViolation {
    pub(crate) fn new() -> Self {
        PasswordConstraintsViolation {
            code: PASSWORD_CONSTRAINTS_VIOLATION,
        }
    }
}

#[derive(Display, Debug)]
#[display(fmt = "password doesn't match (code: {}, error: {})", code, error)]
pub(crate) struct PasswordNotMatch {
    pub(super) code: i32,
    pub(super) error: ring::error::Unspecified,
}

impl PasswordNotMatch {
    pub(crate) fn new(error: ring::error::Unspecified) -> Self {
        PasswordNotMatch {
            code: AUTHENTICATION_FAILED,
            error,
        }
    }
}

#[derive(Display, Debug)]
#[display(
    fmt = "account already exists (code: {}, email_address: {})",
    code,
    email_address
)]
pub(crate) struct AccountAlreadyExists {
    pub(super) code: i32,
    pub(super) email_address: String,
}

impl AccountAlreadyExists {
    pub(crate) fn new(email_address: String) -> Self {
        AccountAlreadyExists {
            code: ACCOUNT_ALREADY_EXISTS,
            email_address,
        }
    }
}

#[derive(Display, Debug)]
#[display(
    fmt = "reach limit of temporary account (code: {}, email_address: {}, count: {})",
    code,
    email_address,
    count
)]
pub(crate) struct ReachLimitOfTemporaryAccount {
    pub(super) code: i32,
    pub(super) email_address: String,
    pub(super) count: i64,
}

impl ReachLimitOfTemporaryAccount {
    pub(crate) fn new(email_address: String, count: i64) -> Self {
        ReachLimitOfTemporaryAccount {
            code: REACH_LIMIT_OF_TEMPORARY_ACCOUNT,
            email_address,
            count,
        }
    }
}

#[derive(Display, Debug)]
#[display(fmt = "no temporary account found (code: {}, id: {})", code, id)]
pub(crate) struct NoTemporaryAccountFound {
    pub(super) code: i32,
    pub(super) id: String,
}

impl NoTemporaryAccountFound {
    pub(crate) fn new(id: String) -> Self {
        NoTemporaryAccountFound {
            code: NO_TEMPORARY_ACCOUNT_FOUND,
            id,
        }
    }
}

#[derive(Display, Debug)]
#[display(
    fmt = "temporary account expired (code: {}, id: {}, created_at: {}, activated_at: {})",
    code,
    id,
    created_at,
    activated_at
)]
pub(crate) struct TemporaryAccountExpired {
    pub(super) code: i32,
    pub(super) id: String,
    pub(super) created_at: chrono::DateTime<chrono::Utc>,
    pub(super) activated_at: chrono::DateTime<chrono::Utc>,
}

impl TemporaryAccountExpired {
    pub(crate) fn new(
        id: String,
        created_at: chrono::DateTime<chrono::Utc>,
        activated_at: chrono::DateTime<chrono::Utc>,
    ) -> Self {
        TemporaryAccountExpired {
            code: TEMPORARY_ACCOUNT_EXPIRED,
            id,
            created_at,
            activated_at,
        }
    }
}

#[derive(Display, Debug)]
#[display(fmt = "invalid temporary account id (code: {}, id: {})", code, id)]
pub(crate) struct InvalidTemporaryAccountId {
    pub(super) code: i32,
    pub(super) id: String,
}

impl InvalidTemporaryAccountId {
    pub(crate) fn new(id: String) -> Self {
        InvalidTemporaryAccountId {
            code: INVALID_TEMPORARY_ACCOUNT_ID,
            id,
        }
    }
}

#[derive(Display, Debug)]
#[display(
    fmt = "no account found (code: {}, email_address: {})",
    code,
    email_address
)]
pub(crate) struct NoAccountFound {
    pub(super) code: i32,
    pub(super) email_address: String,
}

impl NoAccountFound {
    pub(crate) fn new(email_address: String) -> Self {
        NoAccountFound {
            // NOTE: セキュリティ上の観点からPasswordNotMatchと同じ値を返し、メールアドレスが見つからないことと、パスワードが一致しないことを区別しない
            code: AUTHENTICATION_FAILED,
            email_address,
        }
    }
}

#[derive(Display, Debug)]
#[display(fmt = "no session found (code: {})", code)]
pub(crate) struct NoSessionFound {
    pub(super) code: i32,
}

impl NoSessionFound {
    pub(crate) fn new() -> Self {
        NoSessionFound {
            code: NO_SESSION_FOUND,
        }
    }
}
