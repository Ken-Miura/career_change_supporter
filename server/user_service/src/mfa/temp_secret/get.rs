// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::mfa::generate_base64_encoded_qr_code;
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::mfa::{ensure_mfa_is_not_enabled, extract_first_temp_mfa_secret, USER_TOTP_ISSUER};
use crate::mfa::{filter_temp_mfa_secret_order_by_dsc, TempMfaSecret};
use crate::util::session::user::User;

pub(crate) async fn get_temp_mfa_secret(
    User { user_info }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<GetTempMfaSecretResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = TempMfaSecretResultOperationImpl { pool };
    handle_temp_mfp_secret(
        user_info.account_id,
        user_info.mfa_enabled_at.is_some(),
        USER_TOTP_ISSUER.as_str(),
        current_date_time,
        op,
    )
    .await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct GetTempMfaSecretResult {
    // QRコード
    base64_encoded_image: String,
    // QRコードを読み込めない場合に使うシークレットキー
    base32_encoded_secret: String,
}

async fn handle_temp_mfp_secret(
    account_id: i64,
    mfa_enabled: bool,
    issuer: &str,
    current_date_time: DateTime<FixedOffset>,
    op: impl TempMfaSecretResultOperation,
) -> RespResult<GetTempMfaSecretResult> {
    ensure_mfa_is_not_enabled(mfa_enabled)?;

    let temp_mfa_secrets = op
        .filter_temp_mfa_secret_order_by_dsc(account_id, current_date_time)
        .await?;
    // temp_mfa_secretsが登録された日付に降順でソートされているため、1つ目のエントリが最新
    let temp_mfa_secret = extract_first_temp_mfa_secret(temp_mfa_secrets)?;

    let qr_code = generate_base64_encoded_qr_code(
        account_id,
        temp_mfa_secret.base32_encoded_secret.as_str(),
        issuer,
    )?;

    Ok((
        StatusCode::OK,
        Json(GetTempMfaSecretResult {
            base64_encoded_image: qr_code,
            base32_encoded_secret: temp_mfa_secret.base32_encoded_secret,
        }),
    ))
}

#[async_trait]
trait TempMfaSecretResultOperation {
    async fn filter_temp_mfa_secret_order_by_dsc(
        &self,
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<TempMfaSecret>, ErrResp>;
}

struct TempMfaSecretResultOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl TempMfaSecretResultOperation for TempMfaSecretResultOperationImpl {
    async fn filter_temp_mfa_secret_order_by_dsc(
        &self,
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<TempMfaSecret>, ErrResp> {
        filter_temp_mfa_secret_order_by_dsc(account_id, current_date_time, &self.pool).await
    }
}

#[cfg(test)]
mod tests {
    use axum::{async_trait, Json};
    use chrono::{DateTime, FixedOffset, TimeZone};
    use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
    use hyper::StatusCode;
    use once_cell::sync::Lazy;

    use crate::{err::Code, mfa::TempMfaSecret};

    use super::{handle_temp_mfp_secret, GetTempMfaSecretResult, TempMfaSecretResultOperation};

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<GetTempMfaSecretResult>,
    }

    #[derive(Debug)]
    struct Input {
        account_id: i64,
        mfa_enabled: bool,
        issuer: String,
        current_date_time: DateTime<FixedOffset>,
        op: TempMfaSecretResultOperationMock,
    }

    impl Input {
        fn new(
            account_id: i64,
            mfa_enabled: bool,
            issuer: String,
            current_date_time: DateTime<FixedOffset>,
            temp_mfa_secrets: Vec<TempMfaSecret>,
        ) -> Self {
            Input {
                account_id,
                mfa_enabled,
                issuer,
                current_date_time,
                op: TempMfaSecretResultOperationMock {
                    account_id,
                    current_date_time,
                    temp_mfa_secrets,
                },
            }
        }
    }

    #[derive(Clone, Debug)]
    struct TempMfaSecretResultOperationMock {
        account_id: i64,
        current_date_time: DateTime<FixedOffset>,
        temp_mfa_secrets: Vec<TempMfaSecret>,
    }

    #[async_trait]
    impl TempMfaSecretResultOperation for TempMfaSecretResultOperationMock {
        async fn filter_temp_mfa_secret_order_by_dsc(
            &self,
            account_id: i64,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<Vec<TempMfaSecret>, ErrResp> {
            assert_eq!(self.account_id, account_id);
            assert_eq!(self.current_date_time, current_date_time);
            Ok(self.temp_mfa_secrets.clone())
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        let account_id = 413;
        let mfa_enabled = false;
        let issuer = "Issuer".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 4, 5, 0, 1, 7)
            .unwrap();
        vec![TestCase {
            name: "success".to_string(),
            input: Input::new(
                account_id,
                mfa_enabled,
                issuer.clone(),
                current_date_time,
                vec![TempMfaSecret {
                    temp_mfa_secret_id: 1,
                    base32_encoded_secret: "HU7YU2643SZJMWFW5MUOMWNMHSGLA3S6".to_string(),
                }],
            ),
            expected: Ok((StatusCode::OK, Json(GetTempMfaSecretResult{
                // account_id, issuer, base32_encoded_secretから実際に生成されたものを利用 
                base64_encoded_image: "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".to_string(), 
                base32_encoded_secret: "HU7YU2643SZJMWFW5MUOMWNMHSGLA3S6".to_string() }))),
        },
        TestCase {
            name: "fail MfaHasAlreadyBeenEnabled".to_string(),
            input: Input::new(
                account_id,
                true,
                issuer.clone(),
                current_date_time,
                vec![TempMfaSecret {
                    temp_mfa_secret_id: 1,
                    base32_encoded_secret: "HU7YU2643SZJMWFW5MUOMWNMHSGLA3S6".to_string(),
                }],
            ),
            expected: Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::MfaHasAlreadyBeenEnabled as u32,
                }),
            )),
        },
        TestCase {
            name: "fail NoTempMfaSecretFound".to_string(),
            input: Input::new(
                account_id,
                mfa_enabled,
                issuer,
                current_date_time,
                vec![],
            ),
            expected: Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::NoTempMfaSecretFound as u32,
                }),
            )),
        }]
    });

    #[tokio::test]
    async fn handle_temp_mfp_secret_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let account_id = test_case.input.account_id;
            let mfa_enabled = test_case.input.mfa_enabled;
            let issuer = test_case.input.issuer.clone();
            let current_date_time = test_case.input.current_date_time;
            let op = test_case.input.op.clone();

            let result = handle_temp_mfp_secret(
                account_id,
                mfa_enabled,
                issuer.as_str(),
                current_date_time,
                op,
            )
            .await;

            let message = format!("test case \"{}\" failed", test_case.name.clone());
            if test_case.expected.is_ok() {
                let resp = result.expect("failed to get Ok");
                let expected = test_case.expected.as_ref().expect("failed to get Ok");
                assert_eq!(expected.0, resp.0, "{}", message);
                assert_eq!(expected.1 .0, resp.1 .0, "{}", message);
            } else {
                let resp = result.expect_err("failed to get Err");
                let expected = test_case.expected.as_ref().expect_err("failed to get Err");
                assert_eq!(expected.0, resp.0, "{}", message);
                assert_eq!(expected.1 .0, resp.1 .0, "{}", message);
            }
        }
    }
}
