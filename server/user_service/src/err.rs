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
    TempAccountExpired = 20003,
    NoTempAccountFound = 20004,
    EmailOrPwdIncorrect = 20005,
    Unauthorized = 20006,
    NotTermsOfUseAgreedYet = 20007,
    AlreadyAgreedTermsOfUse = 20008,
    ReachPasswordChangeReqLimit = 20009,
    NoAccountFound = 20010,
    NoPwdChnageReqFound = 20011,
    PwdChnageReqExpired = 20012,
    ReachPaymentPlatformRateLimit = 20013,
    InvalidLastNameLength = 20014,
    IllegalCharInLastName = 20015,
    InvalidFirstNameLength = 20016,
    IllegalCharInFirstName = 20017,
    InvalidLastNameFuriganaLength = 20018,
    IllegalCharInLastNameFurigana = 20019,
    InvalidFirstNameFuriganaLength = 20020,
    IllegalCharInFirstNameFurigana = 20021,
    IllegalDate = 20022,
    IllegalAge = 20023,
    InvalidPrefecture = 20024,
    InvalidCityLength = 20025,
    IllegalCharInCity = 20026,
    InvalidAddressLine1Length = 20027,
    IllegalCharInAddressLine1 = 20028,
    InvalidAddressLine2Length = 20029,
    IllegalCharInAddressLine2 = 20030,
    InvalidTelNumFormat = 20031,
    NoNameFound = 20032,
    NoFileNameFound = 20033,
    DataParseFailure = 20034,
    InvalidNameInField = 20035,
    InvalidUtf8Sequence = 20036,
    InvalidIdentityJson = 20037,
    NotJpegExtension = 20038,
    ExceedMaxIdentityImageSizeLimit = 20039,
    InvalidJpegImage = 20040,
    NoIdentityFound = 20041,
    NoIdentityImage1Found = 20042,
    CreateIdentityInfoReqAlreadyExists = 20043,
}

pub(crate) fn unexpected_err_resp() -> ErrResp {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            code: Code::UnexpectedErr as u32,
        }),
    )
}
