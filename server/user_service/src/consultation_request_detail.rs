// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::{extract::Query, Extension};
use common::{ErrResp, RespResult};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::util;
use crate::util::session::User;

pub(crate) async fn get_consultation_request_detail(
    User { account_id }: User,
    query: Query<ConsultationRequestDetailQuery>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<ConsultationRequestDetail> {
    todo!()
}

#[derive(Deserialize)]
pub(crate) struct ConsultationRequestDetailQuery {
    pub(crate) consultation_req_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationRequestDetail {}

#[async_trait]
trait ConsultationRequestDetailOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
}

struct ConsultationRequestDetailOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationRequestDetailOperation for ConsultationRequestDetailOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::check_if_identity_exists(&self.pool, account_id).await
    }
}
