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
        admin::Admin, delete_awaiting_payment, find_awaiting_payment_with_exclusive_lock,
        find_identity_by_user_account_id_in_transaction, generate_sender_name,
        validate_consultation_id_is_positive, ConsultationIdBody,
    },
};

pub(crate) async fn post_awaiting_withdrawal(
    Admin { admin_info }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<ConsultationIdBody>,
) -> RespResult<PostAwaitingWithdrawalResult> {
    let consultation_id = req.consultation_id;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = AwaitingWithdrawalOperationImpl { pool };
    handle_awaiting_withdrawal(
        consultation_id,
        admin_info.email_address,
        current_date_time,
        op,
    )
    .await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct PostAwaitingWithdrawalResult {}

async fn handle_awaiting_withdrawal(
    consultation_id: i64,
    admin_email_address: String,
    current_date_time: DateTime<FixedOffset>,
    op: impl AwaitingWithdrawalOperation,
) -> RespResult<PostAwaitingWithdrawalResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    validate_email_address(&admin_email_address).map_err(|e| {
        error!("invalid email address ({}): {}", admin_email_address, e);
        unexpected_err_resp()
    })?;
    op.prepare_for_awaiting_withdrawal(consultation_id, admin_email_address, current_date_time)
        .await?;
    Ok((StatusCode::OK, Json(PostAwaitingWithdrawalResult {})))
}

#[async_trait]
trait AwaitingWithdrawalOperation {
    async fn prepare_for_awaiting_withdrawal(
        &self,
        consultation_id: i64,
        admin_email_address: String,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct AwaitingWithdrawalOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl AwaitingWithdrawalOperation for AwaitingWithdrawalOperationImpl {
    async fn prepare_for_awaiting_withdrawal(
        &self,
        consultation_id: i64,
        admin_email_address: String,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let ap_option =
                        find_awaiting_payment_with_exclusive_lock(consultation_id, txn).await?;
                    let ap = ap_option.ok_or_else(|| {
                        error!(
                            "no awaiting_payment (consultation_id: {}) found",
                            consultation_id
                        );
                        ErrRespStruct {
                            err_resp: (
                                StatusCode::BAD_REQUEST,
                                Json(ApiError {
                                    code: Code::NoAwaitingPaymentFound as u32,
                                }),
                            ),
                        }
                    })?;

                    let id =
                        find_identity_by_user_account_id_in_transaction(txn, ap.user_account_id)
                            .await?;
                    let id = id.ok_or_else(|| {
                        error!(
                            "no identity (user_account_id: {}) found",
                            ap.user_account_id
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;
                    let sender_name = generate_sender_name(id.last_name_furigana.to_string(), id.first_name_furigana.to_string(), ap.meeting_at)
                        .map_err(|e| {
                            error!("failed to generate_sender_name (last_name_furigana: {}, first_name_furigana: {}, meeting_at: {})",
                                id.last_name_furigana, id.first_name_furigana, ap.meeting_at);
                            ErrRespStruct {
                                err_resp: e,
                            }
                        })?;

                    insert_awaiting_withdrawal(ap, sender_name, admin_email_address, current_date_time, txn)
                        .await?;

                    delete_awaiting_payment(consultation_id, txn).await?;

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
                    error!(
                        "failed to prepare_for_awaiting_withdrawal: {}",
                        err_resp_struct
                    );
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

async fn insert_awaiting_withdrawal(
    ap: entity::awaiting_payment::Model,
    sender_name: String,
    payment_confirmed_by: String,
    created_at: DateTime<FixedOffset>,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    let aw = entity::awaiting_withdrawal::ActiveModel {
        consultation_id: Set(ap.consultation_id),
        user_account_id: Set(ap.user_account_id),
        consultant_id: Set(ap.consultant_id),
        meeting_at: Set(ap.meeting_at),
        fee_per_hour_in_yen: Set(ap.fee_per_hour_in_yen),
        sender_name: Set(sender_name),
        payment_confirmed_by: Set(payment_confirmed_by.clone()),
        created_at: Set(created_at),
    };
    let _ = aw.insert(txn).await.map_err(|e| {
        error!("failed to insert awaiting_withdrawal (awaiting_payment: {:?}, payment_confirmed_by: {}, created_at: {}): {}",
            ap, payment_confirmed_by, created_at, e);
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

    struct AwaitingWithdrawalOperationMock {
        consultation_id: i64,
        admin_email_address: String,
        current_date_time: DateTime<FixedOffset>,
        no_awaiting_payment_found: bool,
    }

    #[async_trait]
    impl AwaitingWithdrawalOperation for AwaitingWithdrawalOperationMock {
        async fn prepare_for_awaiting_withdrawal(
            &self,
            consultation_id: i64,
            admin_email_address: String,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            assert_eq!(consultation_id, self.consultation_id);
            assert_eq!(admin_email_address, self.admin_email_address);
            assert_eq!(current_date_time, self.current_date_time);
            if self.no_awaiting_payment_found {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoAwaitingPaymentFound as u32,
                    }),
                ));
            };
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_handle_awaiting_withdrawal_success() {
        let consultation_id = 512;
        let admin_email_address = "admin@test.com".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = AwaitingWithdrawalOperationMock {
            consultation_id,
            admin_email_address: admin_email_address.clone(),
            current_date_time,
            no_awaiting_payment_found: false,
        };

        let result =
            handle_awaiting_withdrawal(consultation_id, admin_email_address, current_date_time, op)
                .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(PostAwaitingWithdrawalResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn test_handle_awaiting_withdrawal_fail_non_positive_consultation_id() {
        let consultation_id = -1;
        let admin_email_address = "abc".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = AwaitingWithdrawalOperationMock {
            consultation_id,
            admin_email_address: admin_email_address.clone(),
            current_date_time,
            no_awaiting_payment_found: false,
        };

        let result =
            handle_awaiting_withdrawal(consultation_id, admin_email_address, current_date_time, op)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::ConsultationIdIsNotPositive as u32, resp.1 .0.code);
    }

    #[tokio::test]
    async fn test_handle_awaiting_withdrawal_fail_invalid_email_address() {
        let consultation_id = 512;
        let admin_email_address = "abc".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = AwaitingWithdrawalOperationMock {
            consultation_id,
            admin_email_address: admin_email_address.clone(),
            current_date_time,
            no_awaiting_payment_found: false,
        };

        let result =
            handle_awaiting_withdrawal(consultation_id, admin_email_address, current_date_time, op)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::INTERNAL_SERVER_ERROR, resp.0);
        assert_eq!(Code::UnexpectedErr as u32, resp.1 .0.code);
    }
}
