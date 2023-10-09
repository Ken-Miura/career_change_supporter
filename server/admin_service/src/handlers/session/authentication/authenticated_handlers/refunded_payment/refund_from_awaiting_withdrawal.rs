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
        find_identity_by_user_account_id_in_transaction, generate_sender_name,
        validate_consultation_id_is_positive, ConsultationIdBody, TRANSFER_FEE_IN_YEN,
    },
};

const REASON: &str = "ユーザーからのクレームのため返金";

pub(crate) async fn post_refund_from_awaiting_withdrawal(
    Admin { admin_info }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<ConsultationIdBody>,
) -> RespResult<RefundFromAwaitingWithdrawalResult> {
    let consultation_id = req.consultation_id;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = RefundFromAwaitingWithdrawalOperationImpl { pool };
    handle_refund_from_awaiting_withdrawal(
        consultation_id,
        admin_info.email_address,
        current_date_time,
        op,
    )
    .await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct RefundFromAwaitingWithdrawalResult {}

async fn handle_refund_from_awaiting_withdrawal(
    consultation_id: i64,
    admin_email_address: String,
    current_date_time: DateTime<FixedOffset>,
    op: impl RefundFromAwaitingWithdrawalOperation,
) -> RespResult<RefundFromAwaitingWithdrawalResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    validate_email_address(&admin_email_address).map_err(|e| {
        error!("invalid email address ({}): {}", admin_email_address, e);
        unexpected_err_resp()
    })?;
    // NOTE:
    // 現在時刻が相談日時を超えていることもチェックすべきだが、
    // 一般公開するサービスではなく、管理者しかアクセスできないサービスなのでそこまで厳密にチェックしていない
    op.refund_from_awaiting_withdrawal(
        consultation_id,
        admin_email_address,
        current_date_time,
        REASON.to_string(),
        *TRANSFER_FEE_IN_YEN,
    )
    .await?;
    Ok((StatusCode::OK, Json(RefundFromAwaitingWithdrawalResult {})))
}

#[async_trait]
trait RefundFromAwaitingWithdrawalOperation {
    async fn refund_from_awaiting_withdrawal(
        &self,
        consultation_id: i64,
        admin_email_address: String,
        current_date_time: DateTime<FixedOffset>,
        reason: String,
        transfer_fee_in_yen: i32,
    ) -> Result<(), ErrResp>;
}

struct RefundFromAwaitingWithdrawalOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl RefundFromAwaitingWithdrawalOperation for RefundFromAwaitingWithdrawalOperationImpl {
    async fn refund_from_awaiting_withdrawal(
        &self,
        consultation_id: i64,
        admin_email_address: String,
        current_date_time: DateTime<FixedOffset>,
        reason: String,
        transfer_fee_in_yen: i32,
    ) -> Result<(), ErrResp> {
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let ap_option =
                        find_awaiting_withdrawal_with_exclusive_lock(consultation_id, txn).await?;
                    let ap = ap_option.ok_or_else(|| {
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

                    insert_refunded_payment(ap, sender_name, admin_email_address, current_date_time, reason, transfer_fee_in_yen, txn)
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
                    error!(
                        "failed to refund_from_awaiting_withdrawal: {}",
                        err_resp_struct
                    );
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

async fn insert_refunded_payment(
    ap: entity::awaiting_withdrawal::Model,
    sender_name: String,
    refund_confirmed_by: String,
    created_at: DateTime<FixedOffset>,
    reason: String,
    transfer_fee_in_yen: i32,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    let rp = entity::refunded_payment::ActiveModel {
        consultation_id: Set(ap.consultation_id),
        user_account_id: Set(ap.user_account_id),
        consultant_id: Set(ap.consultant_id),
        meeting_at: Set(ap.meeting_at),
        fee_per_hour_in_yen: Set(ap.fee_per_hour_in_yen),
        transfer_fee_in_yen: Set(transfer_fee_in_yen),
        sender_name: Set(sender_name),
        reason: Set(reason.clone()),
        refund_confirmed_by: Set(refund_confirmed_by.clone()),
        created_at: Set(created_at),
    };
    let _ = rp.insert(txn).await.map_err(|e| {
        error!("failed to insert refunded_payment (awaiting_withdrawal: {:?}, transfer_fee_in_yen: {}, reason: {}, refund_confirmed_by: {}, created_at: {}): {}",
            ap, transfer_fee_in_yen, reason, refund_confirmed_by, created_at, e);
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

    struct RefundFromAwaitingWithdrawalOperationMock {
        consultation_id: i64,
        admin_email_address: String,
        current_date_time: DateTime<FixedOffset>,
        reason: String,
        transfer_fee_in_yen: i32,
        no_awaiting_withdrawal_found: bool,
    }

    #[async_trait]
    impl RefundFromAwaitingWithdrawalOperation for RefundFromAwaitingWithdrawalOperationMock {
        async fn refund_from_awaiting_withdrawal(
            &self,
            consultation_id: i64,
            admin_email_address: String,
            current_date_time: DateTime<FixedOffset>,
            reason: String,
            transfer_fee_in_yen: i32,
        ) -> Result<(), ErrResp> {
            assert_eq!(consultation_id, self.consultation_id);
            assert_eq!(admin_email_address, self.admin_email_address);
            assert_eq!(current_date_time, self.current_date_time);
            assert_eq!(reason, self.reason);
            assert_eq!(transfer_fee_in_yen, self.transfer_fee_in_yen);
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
    async fn test_handle_refund_from_awaiting_withdrawal_success() {
        let consultation_id = 512;
        let admin_email_address = "admin@test.com".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = RefundFromAwaitingWithdrawalOperationMock {
            consultation_id,
            admin_email_address: admin_email_address.clone(),
            current_date_time,
            reason: REASON.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            no_awaiting_withdrawal_found: false,
        };

        let result = handle_refund_from_awaiting_withdrawal(
            consultation_id,
            admin_email_address,
            current_date_time,
            op,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(RefundFromAwaitingWithdrawalResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn test_handle_refund_from_awaiting_withdrawal_fail_non_positive_consultation_id() {
        let consultation_id = -1;
        let admin_email_address = "abc".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = RefundFromAwaitingWithdrawalOperationMock {
            consultation_id,
            admin_email_address: admin_email_address.clone(),
            current_date_time,
            reason: REASON.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            no_awaiting_withdrawal_found: false,
        };

        let result = handle_refund_from_awaiting_withdrawal(
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
    async fn test_handle_refund_from_awaiting_withdrawal_fail_invalid_email_address() {
        let consultation_id = 512;
        let admin_email_address = "abc".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = RefundFromAwaitingWithdrawalOperationMock {
            consultation_id,
            admin_email_address: admin_email_address.clone(),
            current_date_time,
            reason: REASON.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            no_awaiting_withdrawal_found: false,
        };

        let result = handle_refund_from_awaiting_withdrawal(
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
    async fn test_handle_refund_from_awaiting_withdrawal_fail_no_awaiting_withdrawal_found() {
        let consultation_id = 512;
        let admin_email_address = "admin@test.com".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = RefundFromAwaitingWithdrawalOperationMock {
            consultation_id,
            admin_email_address: admin_email_address.clone(),
            current_date_time,
            reason: REASON.to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            no_awaiting_withdrawal_found: true,
        };

        let result = handle_refund_from_awaiting_withdrawal(
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
