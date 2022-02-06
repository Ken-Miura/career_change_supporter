// Copyright 2021 Ken Miura

//! API呼び出し時の処理の内、user_service crateのコード発生したエラーに対して付与するエラーコードを列挙する。
//! user_service crateでのエラーコードには、20000-29999までの値を利用する。

pub(crate) const UNEXPECTED_ERR: u32 = 20000;
pub(crate) const ACCOUNT_ALREADY_EXISTS: u32 = 20001;
pub(crate) const REACH_TEMP_ACCOUNTS_LIMIT: u32 = 20002;
pub(crate) const INVALID_UUID: u32 = 20003;
pub(crate) const TEMP_ACCOUNT_EXPIRED: u32 = 20004;
pub(crate) const NO_TEMP_ACCOUNT_FOUND: u32 = 20005;
pub(crate) const EMAIL_OR_PWD_INCORRECT: u32 = 20006;
pub(crate) const UNAUTHORIZED: u32 = 20007;
pub(crate) const NOT_TERMS_OF_USE_AGREED_YET: u32 = 20008;
pub(crate) const ALREADY_AGREED_TERMS_OF_USE: u32 = 20009;
pub(crate) const REACH_NEW_PASSWORDS_LIMIT: u32 = 20010;
pub(crate) const NO_ACCOUNT_FOUND: u32 = 20011;
pub(crate) const NO_NEW_PASSWORD_FOUND: u32 = 20012;
pub(crate) const NEW_PASSWORD_EXPIRED: u32 = 20013;
pub(crate) const REACH_PAYMENT_PLATFORM_RATE_LIMIT: u32 = 20014;
pub(crate) const INVALID_LAST_NAME_LENGTH: u32 = 20015;
pub(crate) const ILLEGAL_CHAR_IN_LAST_NAME: u32 = 20016;
pub(crate) const INVALID_FIRST_NAME_LENGTH: u32 = 20017;
pub(crate) const ILLEGAL_CHAR_IN_FIRST_NAME: u32 = 20018;
pub(crate) const INVALID_LAST_NAME_FURIGANA_LENGTH: u32 = 20019;
pub(crate) const ILLEGAL_CHAR_IN_LAST_NAME_FURIGANA: u32 = 20020;
pub(crate) const INVALID_FIRST_NAME_FURIGANA_LENGTH: u32 = 20021;
pub(crate) const ILLEGAL_CHAR_IN_FIRST_NAME_FURIGANA: u32 = 20022;
pub(crate) const ILLEGAL_DATE: u32 = 20023;
pub(crate) const ILLEGAL_AGE: u32 = 20024;
pub(crate) const INVALID_PREFECTURE: u32 = 20025;
pub(crate) const INVALID_CITY_LENGTH: u32 = 20026;
pub(crate) const ILLEGAL_CHAR_IN_CITY: u32 = 20027;
pub(crate) const INVALID_ADDRESS_LINE1_LENGTH: u32 = 20028;
pub(crate) const ILLEGAL_CHAR_IN_ADDRESS_LINE1: u32 = 20029;
pub(crate) const INVALID_ADDRESS_LINE2_LENGTH: u32 = 20030;
pub(crate) const ILLEGAL_CHAR_IN_ADDRESS_LINE2: u32 = 20031;
pub(crate) const INVALID_TEL_NUM_FORMAT: u32 = 20032;
