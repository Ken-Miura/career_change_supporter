// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use common::{ErrResp, RespResult};
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::mfa::ensure_mfa_is_enabled;
use crate::util::session::user::User;

pub(crate) async fn post_disable_mfa_req(
    User { user_info }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<DisableMfaReqResult> {
    let account_id = user_info.account_id;
    let mfa_enabled = user_info.mfa_enabled_at.is_some();
    let op = DisableMfaReqOperationImpl { pool };
    handle_disable_mfa_req(account_id, mfa_enabled, op).await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct DisableMfaReqResult {}

async fn handle_disable_mfa_req(
    account_id: i64,
    mfa_enabled: bool,
    op: impl DisableMfaReqOperation,
) -> RespResult<DisableMfaReqResult> {
    ensure_mfa_is_enabled(mfa_enabled)?;

    op.disable_mfa(account_id).await?;

    Ok((StatusCode::OK, Json(DisableMfaReqResult {})))
}

#[async_trait]
trait DisableMfaReqOperation {
    async fn disable_mfa(&self, account_id: i64) -> Result<(), ErrResp>;
}

struct DisableMfaReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl DisableMfaReqOperation for DisableMfaReqOperationImpl {
    async fn disable_mfa(&self, account_id: i64) -> Result<(), ErrResp> {
        crate::mfa::disable_mfa(account_id, &self.pool).await
    }
}
