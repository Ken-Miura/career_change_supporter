// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use common::{ApiError, ErrResp, RespResult};
use entity::consultation_req;
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::err::{unexpected_err_resp, Code};
use crate::util::session::User;
use crate::util::{self, consultation_req_exists, ConsultationRequest};

pub(crate) async fn post_consultation_request_rejection(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
    Json(param): Json<ConsultationRequestRejectionParam>,
) -> RespResult<ConsultationRequestRejectionResult> {
    let consultation_req_id = param.consultation_req_id;
    let op = ConsultationRequestRejectionImpl { pool };
    handle_consultation_request_rejection(account_id, consultation_req_id, op).await
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
) -> RespResult<ConsultationRequestRejectionResult> {
    validate_consultation_req_id_is_positive(consultation_req_id)?;
    validate_identity_exists(user_account_id, &op).await?;

    let req = op
        .find_consultation_req_by_consultation_req_id(consultation_req_id)
        .await?;
    let req = consultation_req_exists(req, consultation_req_id)?;
    validate_consultation_req_for_delete(&req, user_account_id)?;

    op.delete_consultation_req(req.consultation_req_id).await?;
    // TODO: Errの場合でも大きな問題にはならないので、先に進めるように修正
    op.invalidate_charge(req.charge_id.as_str()).await?;

    // TODO: メール送信

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
    async fn invalidate_charge(&self, charge_id: &str) -> Result<(), ErrResp>;
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

    async fn invalidate_charge(&self, charge_id: &str) -> Result<(), ErrResp> {
        todo!()
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
