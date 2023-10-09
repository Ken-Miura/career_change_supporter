// Copyright 2023 Ken Miura

use async_session::async_trait;
use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{
    util::validator::email_address_validator::validate_email_address, ApiError, ErrResp,
    ErrRespStruct, RespResult, JAPANESE_TIME_ZONE,
};
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, Set, TransactionError,
    TransactionTrait,
};
use serde::Serialize;
use tracing::error;

use crate::{
    err::{unexpected_err_resp, Code},
    handlers::session::authentication::authenticated_handlers::{
        admin::Admin, calculate_reward, delete_awaiting_withdrawal,
        find_awaiting_withdrawal_with_exclusive_lock, validate_consultation_id_is_positive,
        ConsultationIdBody, PLATFORM_FEE_RATE_IN_PERCENTAGE, TRANSFER_FEE_IN_YEN,
    },
};

pub(crate) async fn post_receipt_of_consultation(
    Admin { admin_info }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<ConsultationIdBody>,
) -> RespResult<ReceiptOfConsultationResult> {
    let consultation_id = req.consultation_id;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = ReceiptOfConsultationOperationImpl { pool };
    handle_receipt_of_consultation(
        consultation_id,
        admin_info.email_address,
        current_date_time,
        op,
    )
    .await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ReceiptOfConsultationResult {}

async fn handle_receipt_of_consultation(
    consultation_id: i64,
    admin_email_address: String,
    current_date_time: DateTime<FixedOffset>,
    op: impl ReceiptOfConsultationOperation,
) -> RespResult<ReceiptOfConsultationResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    validate_email_address(&admin_email_address).map_err(|e| {
        error!("invalid email address ({}): {}", admin_email_address, e);
        unexpected_err_resp()
    })?;
    // NOTE:
    // 現在時刻が出金可能時刻を超えていることもチェックすべきだが、
    // 一般公開するサービスではなく、管理者しかアクセスできないサービスなのでそこまで厳密にチェックしていない
    op.receipt_of_consultation(
        consultation_id,
        admin_email_address,
        current_date_time,
        PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string(),
        *TRANSFER_FEE_IN_YEN,
    )
    .await?;
    Ok((StatusCode::OK, Json(ReceiptOfConsultationResult {})))
}

#[async_trait]
trait ReceiptOfConsultationOperation {
    async fn receipt_of_consultation(
        &self,
        consultation_id: i64,
        admin_email_address: String,
        current_date_time: DateTime<FixedOffset>,
        platform_fee_rate_in_percentage: String,
        transfer_fee_in_yen: i32,
    ) -> Result<(), ErrResp>;
}

struct ReceiptOfConsultationOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ReceiptOfConsultationOperation for ReceiptOfConsultationOperationImpl {
    async fn receipt_of_consultation(
        &self,
        consultation_id: i64,
        admin_email_address: String,
        current_date_time: DateTime<FixedOffset>,
        platform_fee_rate_in_percentage: String,
        transfer_fee_in_yen: i32,
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

                    // 欲しいのはコンサルタントの口座情報なのでconsultant_idを渡す
                    let ba_option = find_bank_account(aw.consultant_id, txn).await?;
                    let ba = ba_option.ok_or_else(|| {
                        error!(
                            "no bank_account (consultant_id: {}) found",
                            aw.consultant_id
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    let reward = calculate_reward(
                        aw.fee_per_hour_in_yen,
                        &platform_fee_rate_in_percentage,
                        transfer_fee_in_yen,
                    )
                    .map_err(|e| {
                        error!(
                            "failed calculate_reward ({}, {}, {})",
                            aw.fee_per_hour_in_yen,
                            &platform_fee_rate_in_percentage,
                            transfer_fee_in_yen
                        );
                        ErrRespStruct { err_resp: e }
                    })?;

                    let fee_related_info = FeeRelatedInfo {
                        transfer_fee_in_yen,
                        platform_fee_rate_in_percentage,
                    };

                    insert_receipt_of_consultation(
                        aw,
                        ba,
                        admin_email_address,
                        current_date_time,
                        fee_related_info,
                        reward,
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
                    error!("failed to receipt_of_consultation: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

async fn find_bank_account(
    user_account_id: i64,
    txn: &DatabaseTransaction,
) -> Result<Option<entity::bank_account::Model>, ErrRespStruct> {
    let model = entity::bank_account::Entity::find_by_id(user_account_id)
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find bank_account (user_account_id: {}): {}",
                user_account_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(model)
}

struct FeeRelatedInfo {
    transfer_fee_in_yen: i32,
    platform_fee_rate_in_percentage: String,
}

async fn insert_receipt_of_consultation(
    aw: entity::awaiting_withdrawal::Model,
    ba: entity::bank_account::Model,
    withdrawal_confirmed_by: String,
    created_at: DateTime<FixedOffset>,
    fee_related_info: FeeRelatedInfo,
    reward: i32,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    let rp = entity::receipt_of_consultation::ActiveModel {
        consultation_id: Set(aw.consultation_id),
        user_account_id: Set(aw.user_account_id),
        consultant_id: Set(aw.consultant_id),
        meeting_at: Set(aw.meeting_at),
        fee_per_hour_in_yen: Set(aw.fee_per_hour_in_yen),
        platform_fee_rate_in_percentage: Set(fee_related_info
            .platform_fee_rate_in_percentage
            .clone()),
        transfer_fee_in_yen: Set(fee_related_info.transfer_fee_in_yen),
        reward: Set(reward),
        sender_name: Set(aw.sender_name.clone()),
        bank_code: Set(ba.bank_code.clone()),
        branch_code: Set(ba.branch_code.clone()),
        account_type: Set(ba.account_type.clone()),
        account_number: Set(ba.account_number.clone()),
        account_holder_name: Set(ba.account_holder_name.clone()),
        withdrawal_confirmed_by: Set(withdrawal_confirmed_by.clone()),
        created_at: Set(created_at),
    };
    let _ = rp.insert(txn).await.map_err(|e| {
        error!("failed to insert receipt_of_consultation (awaiting_withdrawal: {:?}, bank_account: {:?}, platform_fee_rage_in_percentage: {}, transfer_fee_in_yen: {}, reward: {}, withdrawal_confirmed_by: {}, created_at: {}): {}",
            aw, ba, fee_related_info.platform_fee_rate_in_percentage, fee_related_info.transfer_fee_in_yen, reward, withdrawal_confirmed_by, created_at, e);
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

    struct ReceiptOfConsultationOperationMock {
        consultation_id: i64,
        admin_email_address: String,
        current_date_time: DateTime<FixedOffset>,
        reason: String,
        transfer_fee_in_yen: i32,
        no_awaiting_withdrawal_found: bool,
    }

    #[async_trait]
    impl ReceiptOfConsultationOperation for ReceiptOfConsultationOperationMock {
        async fn receipt_of_consultation(
            &self,
            consultation_id: i64,
            admin_email_address: String,
            current_date_time: DateTime<FixedOffset>,
            platform_fee_rate_in_percentage: String,
            transfer_fee_in_yen: i32,
        ) -> Result<(), ErrResp> {
            assert_eq!(consultation_id, self.consultation_id);
            assert_eq!(admin_email_address, self.admin_email_address);
            assert_eq!(current_date_time, self.current_date_time);
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
    async fn test_handle_receipt_of_consultation_success() {
        let consultation_id = 512;
        let admin_email_address = "admin@test.com".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = ReceiptOfConsultationOperationMock {
            consultation_id,
            admin_email_address: admin_email_address.clone(),
            current_date_time,
            reason: "REASON".to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            no_awaiting_withdrawal_found: false,
        };

        let result = handle_receipt_of_consultation(
            consultation_id,
            admin_email_address,
            current_date_time,
            op,
        )
        .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(ReceiptOfConsultationResult {}, resp.1 .0);
    }

    #[tokio::test]
    async fn test_handle_receipt_of_consultation_fail_non_positive_consultation_id() {
        let consultation_id = -1;
        let admin_email_address = "abc".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = ReceiptOfConsultationOperationMock {
            consultation_id,
            admin_email_address: admin_email_address.clone(),
            current_date_time,
            reason: "REASON".to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            no_awaiting_withdrawal_found: false,
        };

        let result = handle_receipt_of_consultation(
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
    async fn test_handle_receipt_of_consultation_fail_invalid_email_address() {
        let consultation_id = 512;
        let admin_email_address = "abc".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = ReceiptOfConsultationOperationMock {
            consultation_id,
            admin_email_address: admin_email_address.clone(),
            current_date_time,
            reason: "REASON".to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            no_awaiting_withdrawal_found: false,
        };

        let result = handle_receipt_of_consultation(
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
    async fn test_handle_receipt_of_consultation_fail_no_awaiting_withdrawal_found() {
        let consultation_id = 512;
        let admin_email_address = "admin@test.com".to_string();
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 9, 5, 21, 0, 40)
            .unwrap();
        let op = ReceiptOfConsultationOperationMock {
            consultation_id,
            admin_email_address: admin_email_address.clone(),
            current_date_time,
            reason: "REASON".to_string(),
            transfer_fee_in_yen: *TRANSFER_FEE_IN_YEN,
            no_awaiting_withdrawal_found: true,
        };

        let result = handle_receipt_of_consultation(
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
