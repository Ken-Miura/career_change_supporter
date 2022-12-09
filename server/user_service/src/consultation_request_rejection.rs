// Copyright 2022 Ken Miura

use async_session::log::warn;
use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use common::payment_platform::charge::{ChargeOperation, ChargeOperationImpl, RefundQuery};
use common::smtp::{
    SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SMTP_HOST, SMTP_PASSWORD, SMTP_PORT,
    SMTP_USERNAME, SYSTEM_EMAIL_ADDRESS,
};
use common::{ApiError, ErrResp, RespResult, WEB_SITE_NAME};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use entity::{consultation_req, user_account};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::err::{unexpected_err_resp, Code};
use crate::util::session::User;
use crate::util::{self, consultation_req_exists, ConsultationRequest, ACCESS_INFO};

static CONSULTATION_REQ_REJECTION_MAIL_SUBJECT: Lazy<String> =
    Lazy::new(|| format!("[{}] 相談申し込み拒否通知", WEB_SITE_NAME));

pub(crate) async fn post_consultation_request_rejection(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
    Json(param): Json<ConsultationRequestRejectionParam>,
) -> RespResult<ConsultationRequestRejectionResult> {
    let consultation_req_id = param.consultation_req_id;
    let op = ConsultationRequestRejectionImpl { pool };
    let smtp_client = SmtpClient::new(
        SMTP_HOST.to_string(),
        *SMTP_PORT,
        SMTP_USERNAME.to_string(),
        SMTP_PASSWORD.to_string(),
    );
    handle_consultation_request_rejection(account_id, consultation_req_id, op, smtp_client).await
}

#[derive(Deserialize)]
pub(crate) struct ConsultationRequestRejectionParam {
    pub(crate) consultation_req_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationRequestRejectionResult {}

async fn handle_consultation_request_rejection(
    user_account_id: i64,
    consultation_req_id: i64,
    op: impl ConsultationRequestRejection,
    send_mail: impl SendMail,
) -> RespResult<ConsultationRequestRejectionResult> {
    validate_consultation_req_id_is_positive(consultation_req_id)?;
    validate_identity_exists(user_account_id, &op).await?;

    let req = op
        .find_consultation_req_by_consultation_req_id(consultation_req_id)
        .await?;
    let req = consultation_req_exists(req, consultation_req_id)?;
    validate_consultation_req_for_delete(&req, user_account_id)?;

    op.delete_consultation_req(req.consultation_req_id).await?;

    let result = op.release_credit_facility(req.charge_id.as_str()).await;
    // 与信枠は[EXPIRY_DAYS_OF_CHARGE]日後に自動的に開放されるので、失敗しても大きな問題にはならない
    // 従って失敗した場合でもログに記録するだけで処理は先に進める
    if result.is_err() {
        warn!(
            "failed to release credit facility (charge_id: {}, result: {:?})",
            req.charge_id.as_str(),
            result
        );
    };

    send_consultation_req_rejection_mail_if_user_exists(
        req.user_account_id,
        req.consultation_req_id,
        &op,
        &send_mail,
    )
    .await?;

    info!("rejected consultation request ({:?})", req);
    Ok((StatusCode::OK, Json(ConsultationRequestRejectionResult {})))
}

#[async_trait]
trait ConsultationRequestRejection {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp>;
    async fn delete_consultation_req(&self, consultation_req_id: i64) -> Result<(), ErrResp>;
    /// 与信枠を開放する（＋支払いの確定を出来なくする）
    async fn release_credit_facility(
        &self,
        charge_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
    async fn find_user_email_address_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<String>, ErrResp>;
}

struct ConsultationRequestRejectionImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationRequestRejection for ConsultationRequestRejectionImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp> {
        util::find_consultation_req_by_consultation_req_id(&self.pool, consultation_req_id).await
    }

    async fn delete_consultation_req(&self, consultation_req_id: i64) -> Result<(), ErrResp> {
        consultation_req::Entity::delete_by_id(consultation_req_id)
            .exec(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to delete consultation_req (consultation_req_id: {}): {}",
                    consultation_req_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(())
    }

    async fn release_credit_facility(
        &self,
        charge_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
        let refund_reason = "refunded_by_consultation_request_rejection".to_string();
        let query = RefundQuery::new(refund_reason).map_err(Box::new)?;
        let _ = charge_op.refund(charge_id, query).await.map_err(Box::new)?;
        Ok(())
    }

    async fn find_user_email_address_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<String>, ErrResp> {
        let model_option = user_account::Entity::find_by_id(user_account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find user_account (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model_option.map(|m| m.email_address))
    }
}

fn validate_consultation_req_id_is_positive(consultation_req_id: i64) -> Result<(), ErrResp> {
    if !consultation_req_id.is_positive() {
        error!(
            "consultation_req_id ({}) is not positive",
            consultation_req_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonPositiveConsultationReqId as u32,
            }),
        ));
    }
    Ok(())
}

async fn validate_identity_exists(
    account_id: i64,
    op: &impl ConsultationRequestRejection,
) -> Result<(), ErrResp> {
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
    Ok(())
}

fn validate_consultation_req_for_delete(
    consultation_req: &ConsultationRequest,
    consultant_id: i64,
) -> Result<(), ErrResp> {
    if consultation_req.consultant_id != consultant_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonConsultationReqFound as u32,
            }),
        ));
    }
    Ok(())
}

async fn send_consultation_req_rejection_mail_if_user_exists(
    user_account_id: i64,
    consultation_req_id: i64,
    op: &impl ConsultationRequestRejection,
    send_mail: &impl SendMail,
) -> Result<(), ErrResp> {
    let user_email_address = op
        .find_user_email_address_by_user_account_id(user_account_id)
        .await?;
    // メールアドレスが取得出来ない = アカウント削除済みを意味するのでそのケースは通知の必要なし
    if let Some(user_email_address) = user_email_address {
        info!(
            "send consultation request rejection mail (consultation_req_id: {}) to {}",
            consultation_req_id, user_email_address
        );
        send_mail
            .send_mail(
                user_email_address.as_str(),
                SYSTEM_EMAIL_ADDRESS,
                CONSULTATION_REQ_REJECTION_MAIL_SUBJECT.as_str(),
                create_text(consultation_req_id).as_str(),
            )
            .await?;
    }
    Ok(())
}

fn create_text(consultation_req_id: i64) -> String {
    // TODO: 文面の調整
    format!(
        r"相談申し込み（相談申し込み番号: {}）が拒否されました（相談申し込みが拒否されたため、相談料金の支払いは発生しません）
        
【お問い合わせ先】
Email: {}",
        consultation_req_id, INQUIRY_EMAIL_ADDRESS
    )
}

#[cfg(test)]
mod tests {}
