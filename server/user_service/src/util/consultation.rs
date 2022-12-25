// Copyright 2022 Ken Miura

use axum::http::StatusCode;
use axum::Json;
use common::{ApiError, ErrResp};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::err::{unexpected_err_resp, Code};

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct ConsultationDateTime {
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) day: u32,
    pub(crate) hour: u32,
}

pub(crate) fn convert_payment_err_to_err_resp(e: &common::payment_platform::Error) -> ErrResp {
    match e {
        common::payment_platform::Error::RequestProcessingError(_) => unexpected_err_resp(),
        common::payment_platform::Error::ApiError(err_info) => {
            let err_detail = &err_info.error;
            // https://pay.jp/docs/api/#error
            // status、typeとcodeがエラーハンドリングに使用可能に見える。
            // そのうち、typeはどのような場合に発生するエラーなのか説明が抽象的すぎてわからない。そのため、エラーハンドリングにはcodeとstatusを用いる。
            // codeの方がより詳細なエラーを示している。そのため、まずはcodeがあるか確認し、存在する場合はそちらをもとにエラーハンドリングし、なければstatusを用いる。
            if let Some(code) = err_detail.code.clone() {
                create_err_resp_from_code(code.as_str())
            } else {
                create_err_resp_from_status(err_detail.status)
            }
        }
    }
}

fn create_err_resp_from_code(code: &str) -> ErrResp {
    info!("code: {}", code);
    if code == "incorrect_card_data" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PayJpCodeIncorrectCardData as u32,
            }),
        )
    } else if code == "card_declined" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PayJpCodeCardDeclined as u32,
            }),
        )
    } else if code == "card_flagged" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PayJpCodeCardFlagged as u32,
            }),
        )
    } else if code == "unacceptable_brand" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PayJpCodeUnacceptableBrand as u32,
            }),
        )
    } else if code == "over_capacity" {
        (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ApiError {
                code: Code::ReachPaymentPlatformRateLimit as u32,
            }),
        )
    } else if code == "three_d_secure_incompleted" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PayJpCodeThreeDSecureIncompleted as u32,
            }),
        )
    } else if code == "three_d_secure_failed" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PayJpCodeThreeDSecureFailed as u32,
            }),
        )
    } else if code == "not_in_three_d_secure_flow" {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::PayJpCodeNotInThreeDSecureFlow as u32,
            }),
        )
    } else {
        // 上記で記載のcode以外は、ユーザーが利用するサービスでは想定していないもののため、unexpected_err_resp() で丸めて返却する
        unexpected_err_resp()
    }
}

fn create_err_resp_from_status(status: u32) -> ErrResp {
    info!("status: {}", status);
    if status == 402 {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::CardAuthPaymentError as u32,
            }),
        )
    } else if status == 429 {
        (
            StatusCode::TOO_MANY_REQUESTS,
            Json(ApiError {
                code: Code::ReachPaymentPlatformRateLimit as u32,
            }),
        )
    } else {
        unexpected_err_resp()
    }
}

#[cfg(test)]
mod tests {

    use axum::http::StatusCode;
    use axum::Json;
    use common::{
        payment_platform::{ErrorDetail, ErrorInfo},
        ApiError, ErrResp,
    };
    use once_cell::sync::Lazy;

    use crate::{err::Code, util::consultation::convert_payment_err_to_err_resp};

    #[derive(Debug)]
    struct ConvertPaymentErrToErrRespTestCase {
        name: String,
        input: ConvertPaymentErrToErrRespInput,
        expected: ErrResp,
    }

    #[derive(Debug)]
    struct ConvertPaymentErrToErrRespInput {
        err: common::payment_platform::Error,
    }

    static CONVERT_PAYMENT_ERR_TO_ERR_RESP_TEST_CASE_SET: Lazy<
        Vec<ConvertPaymentErrToErrRespTestCase>,
    > = Lazy::new(|| {
        // ErrorDetailのメンバーは、実際に返ってくる値が不明なため使う値のみ正確に埋める。
        // pay.jpを使う中で実際に正確な値がわかった場合、随時更新していく。
        vec![
            ConvertPaymentErrToErrRespTestCase {
                name: "status 402".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 402,
                            r#type: "type".to_string(),
                            code: None,
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::CardAuthPaymentError as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "status 429".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 429,
                            r#type: "type".to_string(),
                            code: None,
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ApiError {
                        code: Code::ReachPaymentPlatformRateLimit as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code incorrect_card_data".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("incorrect_card_data".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeIncorrectCardData as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code card_declined".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("card_declined".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeCardDeclined as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code card_flagged".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("card_flagged".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeCardFlagged as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code unacceptable_brand".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("unacceptable_brand".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeUnacceptableBrand as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code over_capacity".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("over_capacity".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::TOO_MANY_REQUESTS,
                    Json(ApiError {
                        code: Code::ReachPaymentPlatformRateLimit as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code three_d_secure_incompleted".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("three_d_secure_incompleted".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeThreeDSecureIncompleted as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code three_d_secure_failed".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("three_d_secure_failed".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeThreeDSecureFailed as u32,
                    }),
                ),
            },
            ConvertPaymentErrToErrRespTestCase {
                name: "code not_in_three_d_secure_flow".to_string(),
                input: ConvertPaymentErrToErrRespInput {
                    err: common::payment_platform::Error::ApiError(Box::new(ErrorInfo {
                        error: ErrorDetail {
                            message: "message".to_string(),
                            status: 400,
                            r#type: "type".to_string(),
                            code: Some("not_in_three_d_secure_flow".to_string()),
                            param: None,
                            charge: None,
                        },
                    })),
                },
                expected: (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::PayJpCodeNotInThreeDSecureFlow as u32,
                    }),
                ),
            },
        ]
    });

    #[test]
    fn test_convert_payment_err_to_err_resp() {
        for test_case in CONVERT_PAYMENT_ERR_TO_ERR_RESP_TEST_CASE_SET.iter() {
            let err_resp = convert_payment_err_to_err_resp(&test_case.input.err);
            let message = format!("test case \"{}\" failed", test_case.name.clone());
            assert_eq!(test_case.expected.0, err_resp.0, "{}", message);
            assert_eq!(test_case.expected.1 .0, err_resp.1 .0, "{}", message);
        }
    }
}
