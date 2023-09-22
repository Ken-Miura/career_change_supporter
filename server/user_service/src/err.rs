// Copyright 2021 Ken Miura

//! エラーに関連する構造体、関数を集約するモジュール

use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};
use tracing::error;

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
    AccountDisabled = 20007,
    NotTermsOfUseAgreedYet = 20008,
    AlreadyAgreedTermsOfUse = 20009,
    ReachPasswordChangeReqLimit = 20010,
    NoAccountFound = 20011,
    NoPwdChnageReqFound = 20012,
    PwdChnageReqExpired = 20013,
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
    ExceedMaxIdentityImageSizeLimit = 20040,
    InvalidJpegImage = 20041,
    NoIdentityFound = 20042,
    NoIdentityImage1Found = 20043,
    IdentityReqAlreadyExists = 20044,
    DateOfBirthIsNotMatch = 20045,
    NoIdentityUpdated = 20046,
    FirstNameIsNotMatch = 20047,
    InvalidMultiPartFormData = 20048,
    InvalidCareerJson = 20049,
    InvalidCompanyNameLength = 20050,
    IllegalCharInCompanyName = 20051,
    InvalidDepartmentNameLength = 20052,
    IllegalCharInDepartmentName = 20053,
    InvalidOfficeLength = 20054,
    IllegalCharInOffice = 20055,
    IllegalCareerStartDate = 20056,
    IllegalCareerEndDate = 20057,
    CareerStartDateExceedsCareerEndDate = 20058,
    IllegalContractType = 20059,
    InvalidProfessionLength = 20060,
    IllegalCharInProfession = 20061,
    IllegalAnnualIncomeInManYen = 20062,
    InvalidPositionNameLength = 20063,
    IllegalCharInPositionName = 20064,
    InvalidNoteLength = 20065,
    IllegalCharInNote = 20066,
    NoCareerFound = 20067,
    NoCareerImage1Found = 20068,
    ExceedMaxCareerImageSizeLimit = 20069,
    ReachCareerNumLimit = 20070,
    NoIdentityRegistered = 20071,
    ReachCreateCareerReqNumLimit = 20072,
    NoCareerToHandleFound = 20073,
    IllegalFeePerHourInYen = 20074,
    InvalidBankCodeFormat = 20075,
    InvalidBranchCodeFormat = 20076,
    InvalidAccountType = 20077,
    InvalidAccountNumberFormat = 20078,
    InvalidAccountHolderNameLength = 20079,
    IllegalCharInAccountHolderName = 20080,
    AccountHolderNameDoesNotMatchFullName = 20081,
    IllegalYearsOfService = 20085,
    EqualOrMoreExceedsEqualOrLessInAnnualIncomeInManYen = 20086,
    EqualOrMoreExceedsEqualOrLessInFeePerHourInYen = 20087,
    InvalidSortKey = 20088,
    InvalidSortOrder = 20089,
    InvalidConsultantSearchParamFrom = 20090,
    InvalidConsultantSearchParamSize = 20091,
    NonPositiveConsultantId = 20092,
    ConsultantDoesNotExist = 20093,
    EqualOrMoreIsLessThanOrMoreYearsOfService = 20094,
    NoCareersFound = 20095,
    NoFeePerHourInYenFound = 20096,
    FeePerHourInYenWasUpdated = 20097,
    ConsultantIsNotAvailable = 20098,
    ProfitObjectiveUseIsNotAllowd = 20099,
    IllegalConsultationDateTime = 20100,
    IllegalConsultationHour = 20101,
    InvalidConsultationDateTime = 20102,
    DuplicateDateTimeCandidates = 20103,
    ThreeDSecureError = 20104,
    ExceedMaxAnnualRewards = 20105,
    CardAuthPaymentError = 20106,
    PayJpCodeIncorrectCardData = 20107,
    PayJpCodeCardDeclined = 20108,
    PayJpCodeCardFlagged = 20109,
    PayJpCodeUnacceptableBrand = 20110,
    PayJpCodeThreeDSecureIncompleted = 20111,
    PayJpCodeThreeDSecureFailed = 20112,
    PayJpCodeNotInThreeDSecureFlow = 20113,
    NonPositiveConsultationReqId = 20114,
    NoConsultationReqFound = 20115,
    InvalidCandidate = 20116,
    UserDoesNotCheckConfirmationItems = 20117,
    TheOtherPersonAccountIsNotAvailable = 20118,
    UserHasSameMeetingDateTime = 20119,
    ConsultantHasSameMeetingDateTime = 20120,
    MeetingDateTimeOverlapsMaintenance = 20121,
    NonPositiveConsultationId = 20122,
    NoConsultationFound = 20123,
    ConsultationRoomHasNotOpenedYet = 20124,
    AudioTestIsNotDone = 20125,
    ConsultationRoomHasAlreadyClosed = 20126,
    RatingIdIsNotPositive = 20127,
    InvalidRating = 20128,
    EndOfConsultationDateTimeHasNotPassedYet = 20129,
    NoUserRatingFound = 20130,
    UserAccountHasAlreadyBeenRated = 20131,
    NoConsultantRatingFound = 20132,
    ConsultantHasAlreadyBeenRated = 20133,
    MfaHasAlreadyBeenEnabled = 20134,
    ReachTempMfaSecretLimit = 20135,
    NoTempMfaSecretFound = 20136,
    InvalidPassCode = 20137,
    PassCodeDoesNotMatch = 20138,
    MfaIsNotEnabled = 20139,
    InvalidRecoveryCode = 20140,
    RecoveryCodeDoesNotMatch = 20141,
    AccountDeleteIsNotConfirmed = 20142,
    NoEnoughSpareTimeBeforeMeeting = 20143,
    ConsultationHasNotBeenFinished = 20144,
}

pub(crate) fn unexpected_err_resp() -> ErrResp {
    error!("unexpected error");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiError {
            code: Code::UnexpectedErr as u32,
        }),
    )
}
