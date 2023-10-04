// Copyright 2023 Ken Miura

use async_session::async_trait;
use axum::{extract::State, http::StatusCode, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::handlers::session::authentication::authenticated_handlers::{
    admin::Admin, ConsultationIdBody,
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
        todo!()
    }
}
