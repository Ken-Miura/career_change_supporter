// Copyright 2023 Ken Miura

use async_session::async_trait;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{
    ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use serde::Serialize;
use tracing::error;

use crate::{
    err::unexpected_err_resp,
    handlers::session::authentication::authenticated_handlers::{
        admin::Admin, pagination::Pagination,
    },
};

// DBテーブルの設計上、この回数分だけクエリを呼ぶようになるため、他より少なめな一方で運用上閲覧するのに十分な値を設定する
const VALID_PAGE_SIZE: u64 = 20;

pub(crate) async fn get_awaiting_payments(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<Pagination>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<AwaitingPaymentResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = AwaitingPaymentsOperationImpl { pool };
    handle_awaiting_payments(query.page, query.per_page, current_date_time, op).await
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct AwaitingPaymentResult {
    awaiting_payments: Vec<AwaitingPayment>,
}

#[derive(Clone, Serialize, Debug, PartialEq)]
pub(crate) struct AwaitingPayment {
    consultation_id: i64,
    consultant_id: i64,
    user_account_id: i64,
    meeting_at: String, // RFC 3339形式の文字列
    fee_per_hour_in_yen: i32,
    sender_name: String,
}

async fn handle_awaiting_payments(
    page: u64,
    per_page: u64,
    current_date_time: DateTime<FixedOffset>,
    op: impl AwaitingPaymentsOperation,
) -> RespResult<AwaitingPaymentResult> {
    if per_page != VALID_PAGE_SIZE {
        error!("invalid per_page ({})", per_page);
        return Err(unexpected_err_resp());
    };

    let results = op
        .get_awaiting_payment_and_consultation(page, per_page, current_date_time)
        .await?;
    let mut awaiting_payments = Vec::with_capacity(results.len());
    for result in results {
        // resultsの個数回分だけDBアクセスが発生してしまうが、per_page回以下であることが保証されるため、許容する
        let name = op
            .find_name_by_user_account_id(result.user_account_id)
            .await?;
        let sender_name = create_sender_name(name, result.meeting_at);
        awaiting_payments.push(AwaitingPayment {
            consultation_id: result.consultation_id,
            consultant_id: result.consultant_id,
            user_account_id: result.user_account_id,
            meeting_at: result.meeting_at.to_rfc3339(),
            fee_per_hour_in_yen: result.fee_per_hour_in_yen,
            sender_name,
        })
    }

    Ok((
        StatusCode::OK,
        Json(AwaitingPaymentResult { awaiting_payments }),
    ))
}

struct AwaitingPaymentAndConsultation {
    consultation_id: i64,
    consultant_id: i64,
    user_account_id: i64,
    meeting_at: DateTime<FixedOffset>,
    fee_per_hour_in_yen: i32,
}

struct Name {
    first_name: String,
    last_name: String,
}

#[async_trait]
trait AwaitingPaymentsOperation {
    async fn get_awaiting_payment_and_consultation(
        &self,
        page: u64,
        per_page: u64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<AwaitingPaymentAndConsultation>, ErrResp>;

    async fn find_name_by_user_account_id(&self, user_account_id: i64) -> Result<Name, ErrResp>;
}

struct AwaitingPaymentsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl AwaitingPaymentsOperation for AwaitingPaymentsOperationImpl {
    async fn get_awaiting_payment_and_consultation(
        &self,
        page: u64,
        per_page: u64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<AwaitingPaymentAndConsultation>, ErrResp> {
        let models = entity::awaiting_payment::Entity::find()
            .find_also_related(entity::consultation::Entity)
            .filter(entity::consultation::Column::MeetingAt.lt(current_date_time))
            .order_by_asc(entity::consultation::Column::MeetingAt)
            .paginate(&self.pool, per_page)
            .fetch_page(page)
            .await
            .map_err(|e| {
                error!(
                    "failed to find awaiting_payment and consultation (page: {}, per_page: {}, current_date_time: {}): {}",
                    page, per_page, current_date_time, e
                );
                unexpected_err_resp()
            })?;
        models
            .into_iter()
            .map(|m| {
                let a = m.0;
                let c = m.1.ok_or_else(|| {
                    error!("failed to get consultation");
                    unexpected_err_resp()
                })?;
                Ok(AwaitingPaymentAndConsultation {
                    consultation_id: a.consultation_id,
                    consultant_id: c.consultant_id,
                    user_account_id: c.user_account_id,
                    meeting_at: c.meeting_at,
                    fee_per_hour_in_yen: a.fee_per_hour_in_yen,
                })
            })
            .collect::<Result<Vec<AwaitingPaymentAndConsultation>, ErrResp>>()
    }

    async fn find_name_by_user_account_id(&self, user_account_id: i64) -> Result<Name, ErrResp> {
        todo!()
    }
}

fn create_sender_name(name: Name, meeting_at: DateTime<FixedOffset>) -> String {
    todo!()
}
