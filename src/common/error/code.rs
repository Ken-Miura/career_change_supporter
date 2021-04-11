// Copyright 2021 Ken Miura

// TODO: 番号の順番を整理する
pub(crate) const EMAIL_FORMAT_INVALID_LENGTH: u32 = 1;
pub(crate) const EMAIL_FORMAT_INVALID_FORMAT: u32 = 2;
pub(crate) const PASSWORD_FORMAT_INVALID_LENGTH: u32 = 3;
pub(crate) const PASSWORD_FORMAT_INVALID_FORMAT: u32 = 4;
pub(crate) const PASSWORD_FORMAT_CONSTRAINTS_VIOLATION: u32 = 5;
pub(crate) const AUTHENTICATION_FAILED: u32 = 6;
pub(crate) const INTERNAL_SERVER_ERROR: u32 = 7;
pub(crate) const ACCOUNT_ALREADY_EXISTS: u32 = 9;
pub(crate) const REACH_TEMPORARY_ACCOUNT_LIMIT: u32 = 12;
pub(crate) const NO_TEMPORARY_ACCOUNT: u32 = 13;
pub(crate) const TEMPORARY_ACCOUNT_EXPIRED: u32 = 14;
