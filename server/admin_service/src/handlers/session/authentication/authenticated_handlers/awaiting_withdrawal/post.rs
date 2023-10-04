// Copyright 2023 Ken Miura

use async_session::async_trait;
use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
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
        ConsultationIdBody,
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

                    insert_awaiting_withdrawal(ap, admin_email_address, current_date_time, txn)
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
