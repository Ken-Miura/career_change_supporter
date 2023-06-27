// Copyright 2021 Ken Miura

//! エラーに関連する構造体、関数を集約するモジュール

use axum::{http::StatusCode, Json};
use common::{ApiError, ErrResp};
use tracing::error;

/// API呼び出し時の処理の内、admin_service crateのコード発生したエラーに対して付与するエラーコードの列挙<br>
/// admin_service crateでのエラーコードには、30000-39999までの値を利用する。
pub(crate) enum Code {
    UnexpectedErr = 30000,
    EmailOrPwdIncorrect = 30001,
    InvalidPassCodeFormat = 30002,
    MfaIsNotEnabled = 30003,
    PassCodeDoesNotMatch = 30004,
    Unauthorized = 30005,
    NoAccountFound = 30006,
    IllegalPageSize = 30007,
    NoCreateIdentityReqDetailFound = 30008,
    IllegalDate = 30009,
    InvalidFormatReason = 30010,
    NoUpdateIdentityReqDetailFound = 30011,
    NoIdentityFound = 30012,
    NoCreateCareerReqDetailFound = 30013,
    AccountIdIsNotPositive = 30014,
    ConsultationIdIsNotPositive = 30015,
    SettlementIdIsNotPositive = 30016,
    StoppedSettlementIdIsNotPositive = 30017,
    CreditFacilitiesAlreadyExpired = 30018,
    PaymentRelatedErr = 30019,
    ReceiptIdIsNotPositive = 30020,
    ExceedsRefundTimeLimit = 30021,
    IllegalDateTime = 30022,
    IllegalMaintenanceDateTime = 30023,
    MaintenanceAlreadyHasBeenSet = 30024,
    ExceedsMaxMaintenanceDurationLimit = 30025,
    InvalidTitleLength = 30026,
    IllegalTitle = 30027,
    InvalidBodyLength = 30028,
    IllegalBody = 30029,
    InvalidNewsId = 30030,
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
