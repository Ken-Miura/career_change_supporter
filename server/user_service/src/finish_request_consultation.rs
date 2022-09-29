// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::{Extension, Json};
use chrono::{DateTime, FixedOffset};
use common::payment_platform::charge::{ChargeOperation, ChargeOperationImpl};
use common::smtp::{SendMail, SmtpClient, SMTP_HOST, SMTP_PASSWORD, SMTP_PORT, SMTP_USERNAME};
use common::{ErrResp, RespResult};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::util::session::User;
use crate::util::ACCESS_INFO;

pub(crate) async fn post_finish_request_consultation(
    User { account_id }: User,
    Json(param): Json<FinishRequestConsultationParam>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<FinishRequestConsultationResult> {
    let charge_id = param.charge_id;
    let op = FinishRequestConsultationOperationImpl { pool };
    let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
    let smtp_client = SmtpClient::new(
        SMTP_HOST.to_string(),
        *SMTP_PORT,
        SMTP_USERNAME.to_string(),
        SMTP_PASSWORD.to_string(),
    );
    handle_finish_request_consultation(account_id, charge_id, op, charge_op, smtp_client).await
}

#[derive(Deserialize)]
pub(crate) struct FinishRequestConsultationParam {
    pub charge_id: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct FinishRequestConsultationResult {}

async fn handle_finish_request_consultation(
    account_id: i64,
    charge_id: String,
    op: impl FinishRequestConsultationOperation,
    charge_op: impl ChargeOperation,
    send_mail: impl SendMail,
) -> RespResult<FinishRequestConsultationResult> {
    todo!()
}

#[async_trait]
trait FinishRequestConsultationOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp>;
    async fn create_request_consultation(
        &self,
        account_id: i64,
        consultant_id: i64,
        charge_id: String,
        expired_date_time: DateTime<FixedOffset>,
    ) -> Result<bool, ErrResp>;
}

struct FinishRequestConsultationOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl FinishRequestConsultationOperation for FinishRequestConsultationOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        todo!()
    }

    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp> {
        todo!()
    }

    async fn create_request_consultation(
        &self,
        account_id: i64,
        consultant_id: i64,
        charge_id: String,
        expired_date_time: DateTime<FixedOffset>,
    ) -> Result<bool, ErrResp> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
