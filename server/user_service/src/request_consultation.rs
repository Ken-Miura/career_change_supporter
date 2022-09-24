// Copyright 2022 Ken Miura

use async_session::async_trait;
use axum::{Extension, Json};
use common::{
    payment_platform::charge::{ChargeOperation, ChargeOperationImpl},
    ErrResp, RespResult,
};
use entity::prelude::UserAccount;
use entity::{
    prelude::ConsultingFee,
    sea_orm::{DatabaseConnection, EntityTrait},
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    err::unexpected_err_resp,
    util::{self, session::User, ACCESS_INFO},
};

pub(crate) async fn post_request_consultation(
    User { account_id }: User,
    Json(param): Json<RequestConsultationParam>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<RequestConsultationResult> {
    let request_consultation_op = RequestConsultationOperationImpl { pool };
    let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
    handle_request_consultation(account_id, param, request_consultation_op, charge_op).await
}

#[derive(Deserialize)]
pub(crate) struct RequestConsultationParam {
    pub consultant_id: i64,
    pub fee_per_hour_in_yen: i32,
    pub card_token: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct RequestConsultationResult {
    pub charge_id: String,
}

#[async_trait]
trait RequestConsultationOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;

    async fn check_if_consultant_exists(&self, consultant_id: i64) -> Result<bool, ErrResp>;

    async fn find_fee_per_hour_in_yen_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Option<i32>, ErrResp>;

    async fn find_tenant_id_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Option<String>, ErrResp>;
}

struct RequestConsultationOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl RequestConsultationOperation for RequestConsultationOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn check_if_consultant_exists(&self, consultant_id: i64) -> Result<bool, ErrResp> {
        util::check_if_consultant_exists(&self.pool, consultant_id).await
    }

    async fn find_fee_per_hour_in_yen_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Option<i32>, ErrResp> {
        let model = ConsultingFee::find_by_id(consultant_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find consulting_fee (user_account_id: {}): {}",
                    consultant_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.fee_per_hour_in_yen))
    }

    async fn find_tenant_id_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Option<String>, ErrResp> {
        util::find_tenant_id_by_account_id(&self.pool, consultant_id).await
    }
}

async fn handle_request_consultation(
    account_id: i64,
    request_consultation_param: RequestConsultationParam,
    request_consultation_op: impl RequestConsultationOperation,
    charge_op: impl ChargeOperation,
) -> RespResult<RequestConsultationResult> {
    todo!()
}
