// Copyright 2023 Ken Miura

use async_session::async_trait;
use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{
    util::validator::email_address_validator::validate_email_address, ApiError, ErrResp,
    ErrRespStruct, RespResult, JAPANESE_TIME_ZONE,
};
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, Set, TransactionError,
    TransactionTrait,
};
use serde::Serialize;
use tracing::error;

use crate::{
    err::{unexpected_err_resp, Code},
    handlers::session::authentication::authenticated_handlers::{
        admin::Admin, delete_awaiting_withdrawal, find_awaiting_withdrawal_with_exclusive_lock,
        validate_consultation_id_is_positive, ConsultationIdBody,
    },
};

pub(crate) async fn post_left_awaiting_withdrawal(
    Admin { admin_info }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<ConsultationIdBody>,
) -> RespResult<LeftAwaitingWithdrawalResult> {
    let consultation_id = req.consultation_id;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = LeftAwaitingWithdrawalOperationImpl { pool };
    handle_left_awaiting_withdrawal(
        consultation_id,
        admin_info.email_address,
        current_date_time,
        op,
    )
    .await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct LeftAwaitingWithdrawalResult {}

async fn handle_left_awaiting_withdrawal(
    consultation_id: i64,
    admin_email_address: String,
    current_date_time: DateTime<FixedOffset>,
    op: impl LeftAwaitingWithdrawalOperation,
) -> RespResult<LeftAwaitingWithdrawalResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    validate_email_address(&admin_email_address).map_err(|e| {
        error!("invalid email address ({}): {}", admin_email_address, e);
        unexpected_err_resp()
    })?;
    // NOTE:
    // 現在時刻が出金可能時刻を超えていることもチェックすべきだが、
    // 一般公開するサービスではなく、管理者しかアクセスできないサービスなのでそこまで厳密にチェックしていない
    op.leave_awaiting_withdrawal(consultation_id, admin_email_address, current_date_time)
        .await?;
    Ok((StatusCode::OK, Json(LeftAwaitingWithdrawalResult {})))
}

#[async_trait]
trait LeftAwaitingWithdrawalOperation {
    async fn leave_awaiting_withdrawal(
        &self,
        consultation_id: i64,
        admin_email_address: String,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct LeftAwaitingWithdrawalOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl LeftAwaitingWithdrawalOperation for LeftAwaitingWithdrawalOperationImpl {
    async fn leave_awaiting_withdrawal(
        &self,
        consultation_id: i64,
        admin_email_address: String,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let aw_option =
                        find_awaiting_withdrawal_with_exclusive_lock(consultation_id, txn).await?;
                    let aw = aw_option.ok_or_else(|| {
                        error!(
                            "no awaiting_withdrawal (consultation_id: {}) found",
                            consultation_id
                        );
                        ErrRespStruct {
                            err_resp: (
                                StatusCode::BAD_REQUEST,
                                Json(ApiError {
                                    code: Code::NoAwaitingWithdrawalFound as u32,
                                }),
                            ),
                        }
                    })?;

                    insert_left_awaiting_withdrawal(
                        aw,
                        admin_email_address,
                        current_date_time,
                        txn,
                    )
                    .await?;

                    delete_awaiting_withdrawal(consultation_id, txn).await?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(db_err) => {
                    error!("connection error: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    error!("failed to left_awaiting_withdrawal: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

async fn insert_left_awaiting_withdrawal(
    aw: entity::awaiting_withdrawal::Model,
    confirmed_by: String,
    created_at: DateTime<FixedOffset>,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    let law = entity::left_awaiting_withdrawal::ActiveModel {
        consultation_id: Set(aw.consultation_id),
        user_account_id: Set(aw.user_account_id),
        consultant_id: Set(aw.consultant_id),
        meeting_at: Set(aw.meeting_at),
        fee_per_hour_in_yen: Set(aw.fee_per_hour_in_yen),
        sender_name: Set(aw.sender_name.clone()),
        confirmed_by: Set(confirmed_by.clone()),
        created_at: Set(created_at),
    };
    let _ = law.insert(txn).await.map_err(|e| {
        error!("failed to insert left_awaiting_withdrawal (awaiting_withdrawal: {:?}, confirmed_by: {}, created_at: {}): {}",
            aw, confirmed_by, created_at, e);
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;

    use crate::err::Code;

    use super::*;

    struct LeftAwaitingWithdrawalOperationMock {
        consultation_id: i64,
        admin_email_address: String,
        current_date_time: DateTime<FixedOffset>,
        no_awaiting_withdrawal_found: bool,
    }

    #[async_trait]
    impl LeftAwaitingWithdrawalOperation for LeftAwaitingWithdrawalOperationMock {
        async fn leave_awaiting_withdrawal(
            &self,
            consultation_id: i64,
            admin_email_address: String,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            assert_eq!(consultation_id, self.consultation_id);
            assert_eq!(admin_email_address, self.admin_email_address);
            assert_eq!(current_date_time, self.current_date_time);
            if self.no_awaiting_withdrawal_found {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoAwaitingWithdrawalFound as u32,
                    }),
                ));
            };
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_handle_left_awaiting_withdrawal_success() {
        let consultation_id = 512;
        let admin_email_address = "admin@test.com".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = LeftAwaitingWithdrawalOperationMock {
            consultation_id,
            admin_email_address: admin_email_address.clone(),
            current_date_time,
            no_awaiting_withdrawal_found: false,
        };

        let result = handle_left_awaiting_withdrawal(
            consultation_id,
            admin_email_address,
            current_date_time,
            op,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(LeftAwaitingWithdrawalResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn test_handle_left_awaiting_withdrawal_fail_non_positive_consultation_id() {
        let consultation_id = -1;
        let admin_email_address = "abc".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = LeftAwaitingWithdrawalOperationMock {
            consultation_id,
            admin_email_address: admin_email_address.clone(),
            current_date_time,
            no_awaiting_withdrawal_found: false,
        };

        let result = handle_left_awaiting_withdrawal(
            consultation_id,
            admin_email_address,
            current_date_time,
            op,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::ConsultationIdIsNotPositive as u32, resp.1 .0.code);
    }

    #[tokio::test]
    async fn test_handle_left_awaiting_withdrawal_fail_invalid_email_address() {
        let consultation_id = 512;
        let admin_email_address = "abc".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = LeftAwaitingWithdrawalOperationMock {
            consultation_id,
            admin_email_address: admin_email_address.clone(),
            current_date_time,
            no_awaiting_withdrawal_found: false,
        };

        let result = handle_left_awaiting_withdrawal(
            consultation_id,
            admin_email_address,
            current_date_time,
            op,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.0);
        assert_eq!(Code::UnexpectedErr as u32, resp.1 .0.code);
    }

    #[tokio::test]
    async fn test_handle_left_awaiting_withdrawal_fail_no_awaiting_withdrawal_found() {
        let consultation_id = 512;
        let admin_email_address = "admin@test.com".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = LeftAwaitingWithdrawalOperationMock {
            consultation_id,
            admin_email_address: admin_email_address.clone(),
            current_date_time,
            no_awaiting_withdrawal_found: true,
        };

        let result = handle_left_awaiting_withdrawal(
            consultation_id,
            admin_email_address,
            current_date_time,
            op,
        )
        .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::NoAwaitingWithdrawalFound as u32, resp.1 .0.code);
    }
}
