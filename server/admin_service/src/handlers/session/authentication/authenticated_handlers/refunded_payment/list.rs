// Copyright 2023 Ken Miura

use async_session::async_trait;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use common::{ErrResp, RespResult};
use entity::sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder};
use serde::Serialize;
use tracing::error;

use crate::{
    err::unexpected_err_resp,
    handlers::session::authentication::authenticated_handlers::{
        admin::Admin, convert_date_time_to_rfc3339_string, pagination::Pagination,
    },
};

const VALID_PAGE_SIZE: u64 = 20;

pub(crate) async fn get_refunded_payments(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<Pagination>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<RefundedPaymentsResult> {
    let op = RefundedPaymentsOperationImpl { pool };
    handle_refunded_payments(query.page, query.per_page, op).await
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct RefundedPaymentsResult {
    refunded_payments: Vec<RefundedPayment>,
}

#[derive(Clone, Serialize, Debug, PartialEq)]
struct RefundedPayment {
    consultation_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    meeting_at: String, // RFC 3339形式の文字列,
    fee_per_hour_in_yen: i32,
    transfer_fee_in_yen: i32,
    sender_name: String,
    reason: String,
    refund_confirmed_by: String,
    created_at: String, // RFC 3339形式の文字列
}

#[async_trait]
trait RefundedPaymentsOperation {
    async fn get_refunded_payments(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<Vec<RefundedPayment>, ErrResp>;
}

struct RefundedPaymentsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl RefundedPaymentsOperation for RefundedPaymentsOperationImpl {
    async fn get_refunded_payments(
        &self,
        page: u64,
        per_page: u64,
    ) -> Result<Vec<RefundedPayment>, ErrResp> {
        let models = entity::refunded_payment::Entity::find()
            .order_by_desc(entity::refunded_payment::Column::CreatedAt)
            .paginate(&self.pool, per_page)
            .fetch_page(page)
            .await
            .map_err(|e| {
                error!(
                    "failed to find refunded_payment (page: {}, per_page: {}): {}",
                    page, per_page, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| RefundedPayment {
                consultation_id: m.consultation_id,
                user_account_id: m.user_account_id,
                consultant_id: m.consultant_id,
                meeting_at: convert_date_time_to_rfc3339_string(m.meeting_at),
                fee_per_hour_in_yen: m.fee_per_hour_in_yen,
                transfer_fee_in_yen: m.transfer_fee_in_yen,
                sender_name: m.sender_name,
                reason: m.reason,
                refund_confirmed_by: m.refund_confirmed_by,
                created_at: convert_date_time_to_rfc3339_string(m.created_at),
            })
            .collect())
    }
}

async fn handle_refunded_payments(
    page: u64,
    per_page: u64,
    op: impl RefundedPaymentsOperation,
) -> RespResult<RefundedPaymentsResult> {
    if per_page > VALID_PAGE_SIZE {
        error!("invalid per_page ({})", per_page);
        return Err(unexpected_err_resp());
    };

    let refunded_payments = op.get_refunded_payments(page, per_page).await?;

    Ok((
        StatusCode::OK,
        Json(RefundedPaymentsResult { refunded_payments }),
    ))
}
