// Copyright 2022 Ken Miura

use axum::http::StatusCode;
use axum::{async_trait, Json};
use axum::{extract::Query, Extension};
use common::{ApiError, ErrResp, RespResult};
use entity::prelude::ConsultingFee;
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::util;
use crate::util::session::User;

pub(crate) async fn get_fee_per_hour_in_yen_for_application(
    User { account_id }: User,
    query: Query<FeePerHourInYenForApplicationQuery>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<FeePerHourInYenForApplication> {
    let query = query.0;
    let op = FeePerHourInYenForApplicationOperationImpl { pool };
    handle_fee_per_hour_in_yen_for_application(account_id, query.consultant_id, op).await
}

#[derive(Deserialize)]
pub(crate) struct FeePerHourInYenForApplicationQuery {
    pub consultant_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct FeePerHourInYenForApplication {
    pub fee_per_hour_in_yen: i32,
}

#[async_trait]
trait FeePerHourInYenForApplicationOperation {
    /// Identityが存在するか確認する。存在する場合、trueを返す。そうでない場合、falseを返す。
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
    /// コンサルタントのUserAccountが存在するか確認する。存在する場合、trueを返す。そうでない場合、falseを返す。
    async fn check_if_consultant_exists(&self, consultant_id: i64) -> Result<bool, ErrResp>;
    async fn find_fee_per_hour_in_yen_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Option<i32>, ErrResp>;
}

struct FeePerHourInYenForApplicationOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl FeePerHourInYenForApplicationOperation for FeePerHourInYenForApplicationOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn check_if_consultant_exists(&self, consultant_id: i64) -> Result<bool, ErrResp> {
        util::check_if_consultant_exists(&self.pool, consultant_id).await
    }

    async fn find_fee_per_hour_in_yen_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Option<i32>, ErrResp> {
        let model = ConsultingFee::find_by_id(consultant_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find consulting_fee (consultant_id: {}): {}",
                    consultant_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.fee_per_hour_in_yen))
    }
}

async fn handle_fee_per_hour_in_yen_for_application(
    account_id: i64,
    consultant_id: i64,
    op: impl FeePerHourInYenForApplicationOperation,
) -> RespResult<FeePerHourInYenForApplication> {
    if !consultant_id.is_positive() {
        error!("consultant_id ({}) is not positive", consultant_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonPositiveConsultantId as u32,
            }),
        ));
    }
    let identity_exists = op.check_if_identity_exists(account_id).await?;
    if !identity_exists {
        error!("identity is not registered (account_id: {})", account_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoIdentityRegistered as u32,
            }),
        ));
    }
    let consultant_exists = op.check_if_consultant_exists(consultant_id).await?;
    if !consultant_exists {
        error!(
            "consultant does not exist (consultant_id: {})",
            consultant_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultantDoesNotExist as u32,
            }),
        ));
    }
    let fee_per_hour_in_yen = op
        .find_fee_per_hour_in_yen_by_consultant_id(consultant_id)
        .await?;
    let fee_per_hour_in_yen = fee_per_hour_in_yen.ok_or_else(|| {
        error!(
            "fee_per_hour_in_yen does not exist (consultant_id: {})",
            consultant_id
        );
        unexpected_err_resp()
    })?;
    Ok((
        StatusCode::OK,
        Json(FeePerHourInYenForApplication {
            fee_per_hour_in_yen,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use common::{ApiError, ErrResp, RespResult};
    use once_cell::sync::Lazy;

    use crate::err::Code;

    use super::{
        handle_fee_per_hour_in_yen_for_application, FeePerHourInYenForApplication,
        FeePerHourInYenForApplicationOperation,
    };

    #[derive(Clone, Debug)]
    struct FeePerHourInYenForApplicationOperationMock {
        account_id: i64,
        consultant_id: i64,
        fee_per_hour_in_yen: i32,
    }

    #[async_trait]
    impl FeePerHourInYenForApplicationOperation for FeePerHourInYenForApplicationOperationMock {
        async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
            if self.account_id != account_id {
                return Ok(false);
            };
            Ok(true)
        }

        async fn check_if_consultant_exists(&self, consultant_id: i64) -> Result<bool, ErrResp> {
            if self.consultant_id != consultant_id {
                return Ok(false);
            };
            Ok(true)
        }

        async fn find_fee_per_hour_in_yen_by_consultant_id(
            &self,
            _consultant_id: i64,
        ) -> Result<Option<i32>, ErrResp> {
            Ok(Some(self.fee_per_hour_in_yen))
        }
    }

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<FeePerHourInYenForApplication>,
    }

    #[derive(Debug)]
    struct Input {
        account_id: i64,
        consultant_id: i64,
        op: FeePerHourInYenForApplicationOperationMock,
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        vec![
            TestCase {
                name: "consultant id is not positive".to_string(),
                input: Input {
                    account_id: 1,
                    consultant_id: 0,
                    op: FeePerHourInYenForApplicationOperationMock {
                        account_id: 1,
                        consultant_id: 0,
                        fee_per_hour_in_yen: 3000,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NonPositiveConsultantId as u32,
                    }),
                )),
            },
            TestCase {
                name: "no identity found".to_string(),
                input: Input {
                    account_id: 1,
                    consultant_id: 3,
                    op: FeePerHourInYenForApplicationOperationMock {
                        account_id: 2,
                        consultant_id: 3,
                        fee_per_hour_in_yen: 3000,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoIdentityRegistered as u32,
                    }),
                )),
            },
            TestCase {
                name: "no counsultant found".to_string(),
                input: Input {
                    account_id: 1,
                    consultant_id: 2,
                    op: FeePerHourInYenForApplicationOperationMock {
                        account_id: 1,
                        consultant_id: 3,
                        fee_per_hour_in_yen: 50000,
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::ConsultantDoesNotExist as u32,
                    }),
                )),
            },
            TestCase {
                name: "succeed in getting fee_per_hour_in_yen".to_string(),
                input: Input {
                    account_id: 1,
                    consultant_id: 2,
                    op: FeePerHourInYenForApplicationOperationMock {
                        account_id: 1,
                        consultant_id: 2,
                        fee_per_hour_in_yen: 25000,
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(FeePerHourInYenForApplication {
                        fee_per_hour_in_yen: 25000,
                    }),
                )),
            },
        ]
    });

    #[tokio::test]
    async fn test_handle_fee_per_hour_in_yen_for_application() {
        for test_case in TEST_CASE_SET.iter() {
            let result = handle_fee_per_hour_in_yen_for_application(
                test_case.input.account_id,
                test_case.input.consultant_id,
                test_case.input.op.clone(),
            )
            .await;

            let message = format!("test case \"{}\" failed", test_case.name.clone());
            if let Ok(actual) = result {
                let expected = test_case.expected.as_ref().expect("failed to get Ok");
                assert_eq!(actual.0, expected.0, "{}", message);
                assert_eq!(actual.1 .0, expected.1 .0, "{}", message);
            } else {
                let actual = result.expect_err("failed to get Err");
                let expected = test_case.expected.as_ref().expect_err("failed to get Err");
                assert_eq!(actual.0, expected.0, "{}", message);
                assert_eq!(actual.1 .0, expected.1 .0, "{}", message);
            }
        }
    }
}
