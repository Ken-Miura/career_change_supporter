// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::mfa::generate_base32_encoded_secret;
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, Set,
};
use serde::Serialize;
use tracing::error;

use crate::err::Code;
use crate::mfa::{ensure_mfa_is_not_enabled, MAX_NUM_OF_TEMP_MFA_SECRETS};
use crate::{err::unexpected_err_resp, util::session::user::User};

const VALID_PERIOD_IN_MINUTE: i64 = 15;

pub(crate) async fn post_temp_mfa_secret(
    User { user_info }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<PostTempMfaSecretResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let base32_encoded_secret = generate_base32_encoded_secret()?;
    let op = TempMfaSecretResultOperationImpl { pool };
    handle_temp_mfp_secret(
        user_info.account_id,
        user_info.mfa_enabled_at.is_some(),
        base32_encoded_secret,
        current_date_time,
        op,
    )
    .await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct PostTempMfaSecretResult {}

async fn handle_temp_mfp_secret(
    account_id: i64,
    mfa_enabled: bool,
    base32_encoded_secret: String,
    current_date_time: DateTime<FixedOffset>,
    op: impl TempMfaSecretResultOperation,
) -> RespResult<PostTempMfaSecretResult> {
    ensure_mfa_is_not_enabled(mfa_enabled)?;
    ensure_temp_mfa_secre_does_not_reach_max_count(account_id, MAX_NUM_OF_TEMP_MFA_SECRETS, &op)
        .await?;

    let expiry_date_time = current_date_time + chrono::Duration::minutes(VALID_PERIOD_IN_MINUTE);
    op.create_temp_mfa_secret(account_id, base32_encoded_secret.clone(), expiry_date_time)
        .await?;

    Ok((StatusCode::OK, Json(PostTempMfaSecretResult {})))
}

async fn ensure_temp_mfa_secre_does_not_reach_max_count(
    account_id: i64,
    max_count: u64,
    op: &impl TempMfaSecretResultOperation,
) -> Result<(), ErrResp> {
    let count = op.count_temp_mfa_secret(account_id).await?;
    if count >= max_count {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ReachTempMfaSecretLimit as u32,
            }),
        ));
    }
    Ok(())
}

#[async_trait]
trait TempMfaSecretResultOperation {
    async fn count_temp_mfa_secret(&self, account_id: i64) -> Result<u64, ErrResp>;

    async fn create_temp_mfa_secret(
        &self,
        account_id: i64,
        base32_encoded_secret: String,
        expiry_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct TempMfaSecretResultOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl TempMfaSecretResultOperation for TempMfaSecretResultOperationImpl {
    async fn count_temp_mfa_secret(&self, account_id: i64) -> Result<u64, ErrResp> {
        let result = entity::temp_mfa_secret::Entity::find()
            .filter(entity::temp_mfa_secret::Column::UserAccountId.eq(account_id))
            .count(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to count temp_mfa_secret (account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(result)
    }

    async fn create_temp_mfa_secret(
        &self,
        account_id: i64,
        base32_encoded_secret: String,
        expiry_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        let active_model = entity::temp_mfa_secret::ActiveModel {
            temp_mfa_secret_id: NotSet,
            user_account_id: Set(account_id),
            base32_encoded_secret: Set(base32_encoded_secret),
            expired_at: Set(expiry_date_time),
        };
        let _ = active_model.insert(&self.pool).await.map_err(|e| {
            error!(
                "failed to insert temp_mfa_secret (user_account_id: {}, expired_at: {}): {}",
                account_id, expiry_date_time, e
            );
            unexpected_err_resp()
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use chrono::{DateTime, FixedOffset, TimeZone};
    use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
    use once_cell::sync::Lazy;

    use crate::err::Code;
    use crate::mfa::temp_secret::post::VALID_PERIOD_IN_MINUTE;

    use super::{handle_temp_mfp_secret, PostTempMfaSecretResult, TempMfaSecretResultOperation};

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<PostTempMfaSecretResult>,
    }

    #[derive(Debug)]
    struct Input {
        account_id: i64,
        mfa_enabled: bool,
        base32_encoded_secret: String,
        current_date_time: DateTime<FixedOffset>,
        op: TempMfaSecretResultOperationMock,
    }

    impl Input {
        fn new(
            account_id: i64,
            mfa_enabled: bool,
            base32_encoded_secret: String,
            current_date_time: DateTime<FixedOffset>,
            count: u64,
        ) -> Self {
            Input {
                account_id,
                mfa_enabled,
                base32_encoded_secret: base32_encoded_secret.clone(),
                current_date_time,
                op: TempMfaSecretResultOperationMock {
                    account_id,
                    base32_encoded_secret,
                    current_date_time,
                    count,
                },
            }
        }
    }

    #[derive(Clone, Debug)]
    struct TempMfaSecretResultOperationMock {
        account_id: i64,
        base32_encoded_secret: String,
        current_date_time: DateTime<FixedOffset>,
        count: u64,
    }

    #[async_trait]
    impl TempMfaSecretResultOperation for TempMfaSecretResultOperationMock {
        async fn count_temp_mfa_secret(&self, account_id: i64) -> Result<u64, ErrResp> {
            assert_eq!(self.account_id, account_id);
            Ok(self.count)
        }

        async fn create_temp_mfa_secret(
            &self,
            account_id: i64,
            base32_encoded_secret: String,
            expiry_date_time: DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.account_id, account_id);
            assert_eq!(self.base32_encoded_secret, base32_encoded_secret);
            assert_eq!(
                self.current_date_time + chrono::Duration::minutes(VALID_PERIOD_IN_MINUTE),
                expiry_date_time
            );
            Ok(())
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        let account_id = 1616;
        let mfa_enabled = false;
        let base32_encoded_secret = "7GRCVBFZ73L6NM5VTBKN7SBS4652NTIK".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 4, 5, 0, 1, 7)
            .unwrap();
        let count = 0;
        vec![
            TestCase {
                name: "success".to_string(),
                input: Input::new(
                    account_id,
                    mfa_enabled,
                    base32_encoded_secret.clone(),
                    current_date_time,
                    count,
                ),
                expected: Ok((StatusCode::OK, Json(PostTempMfaSecretResult {}))),
            },
            TestCase {
                name: "fail MfaHasAlreadyBeenEnabled".to_string(),
                input: Input::new(
                    account_id,
                    true,
                    base32_encoded_secret.clone(),
                    current_date_time,
                    count,
                ),
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::MfaHasAlreadyBeenEnabled as u32,
                    }),
                )),
            },
        ]
    });

    #[tokio::test]
    async fn handle_temp_mfp_secret_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let account_id = test_case.input.account_id;
            let mfa_enabled = test_case.input.mfa_enabled;
            let base32_encoded_secret = test_case.input.base32_encoded_secret.clone();
            let current_date_time = test_case.input.current_date_time;
            let op = test_case.input.op.clone();

            let result = handle_temp_mfp_secret(
                account_id,
                mfa_enabled,
                base32_encoded_secret,
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
