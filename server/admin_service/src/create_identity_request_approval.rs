// Copyright 2021 Ken Miura

use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};

use axum::extract::Extension;
use entity::{
    create_identity_req,
    sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, QueryOrder},
};
use serde::{Deserialize, Serialize};

use crate::{err::unexpected_err_resp, util::session::Admin};

pub(crate) async fn post_create_identity_request_approval(
    Admin { account_id }: Admin, // 認証されていることを保証するために必須のパラメータ
    Json(create_identity_req_approval): Json<CreateIdentityReqApproval>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<CreateIdentityReqApprovalResult> {
    let op = CreateIdentityReqApprovalOperationImpl { pool };
    let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
    handle_create_identity_request_approval(
        account_id,
        create_identity_req_approval.user_account_id,
        current_date_time,
        op,
    )
    .await
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateIdentityReqApproval {
    pub(crate) user_account_id: i64,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateIdentityReqApprovalResult {}

async fn handle_create_identity_request_approval(
    admin_account_id: i64,
    user_account_id: i64,
    approved_time: DateTime<FixedOffset>,
    op: impl CreateIdentityReqApprovalOperation,
) -> RespResult<CreateIdentityReqApprovalResult> {
    todo!()
}

#[async_trait]
trait CreateIdentityReqApprovalOperation {}

struct CreateIdentityReqApprovalOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl CreateIdentityReqApprovalOperation for CreateIdentityReqApprovalOperationImpl {}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test() {}
}
