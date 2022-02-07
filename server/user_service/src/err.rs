// Copyright 2021 Ken Miura

//! エラーに関連する構造体、関数を集約するモジュール

use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};

/// API呼び出し時の処理の内、user_service crateのコード発生したエラーに対して付与するエラーコードの列挙<br>
/// user_service crateでのエラーコードには、20000-29999までの値を利用する。
pub(crate) enum Code {
    UnexpectedErr = 20000,
    AccountAlreadyExists = 20001,
    ReachTempAccountsLimit = 20002,
    InvalidUuid = 20003,
    TempAccountExpired = 20004,
    NoTempAccountFound = 20005,
    EmailOrPwdIncorrect = 20006,
    Unauthorized = 20007,
    NotTermsOfUseAgreedYet = 20008,
    AlreadyAgreedTermsOfUse = 20009,
    ReachNewPasswordsLimit = 20010,
    NoAccountFound = 20011,
    NoNewPasswordFound = 20012,
    NewPasswordExpired = 20013,
    ReachPaymentPlatformRateLimit = 20014,
    InvalidLastNameLength = 20015,
    IllegalCharInLastName = 20016,
    InvalidFirstNameLength = 20017,
    IllegalCharInFirstName = 20018,
    InvalidLastNameFuriganaLength = 20019,
    IllegalCharInLastNameFurigana = 20020,
    InvalidFirstNameFuriganaLength = 20021,
    IllegalCharInFirstNameFurigana = 20022,
    IllegalDate = 20023,
    IllegalAge = 20024,
    InvalidPrefecture = 20025,
    InvalidCityLength = 20026,
    IllegalCharInCity = 20027,
    InvalidAddressLine1Length = 20028,
    IllegalCharInAddressLine1 = 20029,
    InvalidAddressLine2Length = 20030,
    IllegalCharInAddressLine2 = 20031,
    InvalidTelNumFormat = 20032,
    NoNameFound = 20033,
    NoFileNameFound = 20034,
    DataParseFailure = 20035,
    InvalidNameInField = 20036,
    InvalidUtf8Sequence = 20037,
    InvalidIdentityJson = 20038,
    NotJpegExtension = 20039,
}

pub(crate) fn unexpected_err_resp() -> ErrResp {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            code: Code::UnexpectedErr as u32,
        }),
    )
}
