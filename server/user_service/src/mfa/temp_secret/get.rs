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
        USER_TOTP_ISSUER.as_str(),
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
    use axum::async_trait;
    use chrono::{DateTime, FixedOffset};
    use common::{ErrResp, RespResult};
    use once_cell::sync::Lazy;

    use crate::mfa::TempMfaSecret;

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
        current_date_time: DateTime<FixedOffset>,
        op: TempMfaSecretResultOperationMock,
    }

    impl Input {
        fn new(
            account_id: i64,
            mfa_enabled: bool,
            current_date_time: DateTime<FixedOffset>,
            temp_mfa_secrets: Vec<TempMfaSecret>,
        ) -> Self {
            Input {
                account_id,
                mfa_enabled,
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

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| vec![]);

    #[tokio::test]
    async fn handle_temp_mfp_secret_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let account_id = test_case.input.account_id;
            let mfa_enabled = test_case.input.mfa_enabled;
            let current_date_time = test_case.input.current_date_time;
            let op = test_case.input.op.clone();

            let result =
                handle_temp_mfp_secret(account_id, mfa_enabled, current_date_time, op).await;

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
