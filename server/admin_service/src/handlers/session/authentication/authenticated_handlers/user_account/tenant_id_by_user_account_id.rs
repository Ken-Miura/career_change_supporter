// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::{ErrResp, RespResult};
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use super::super::admin::Admin;
use super::{validate_account_id_is_positive, UserAccountIdQuery};

pub(crate) async fn get_tenant_id_by_user_account_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<UserAccountIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<TenantIdResult> {
    let query = query.0;
    let op = TenantIdOperationImpl { pool };
    get_tenant_id_by_user_account_id_internal(query.user_account_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct TenantIdResult {
    tenant_id: Option<String>,
}

async fn get_tenant_id_by_user_account_id_internal(
    user_account_id: i64,
    op: impl TenantIdOperation,
) -> RespResult<TenantIdResult> {
    validate_account_id_is_positive(user_account_id)?;
    let tenant_id = op.get_tenant_id_by_user_account_id(user_account_id).await?;
    Ok((StatusCode::OK, Json(TenantIdResult { tenant_id })))
}

#[async_trait]
trait TenantIdOperation {
    async fn get_tenant_id_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<String>, ErrResp>;
}

struct TenantIdOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl TenantIdOperation for TenantIdOperationImpl {
    async fn get_tenant_id_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<String>, ErrResp> {
        super::get_tenant_id(user_account_id, &self.pool).await
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::ErrResp;

    use crate::err::Code;

    use super::*;

    struct TenantIdOperationMock {
        user_account_id: i64,
        tenant_id: String,
    }

    #[async_trait]
    impl TenantIdOperation for TenantIdOperationMock {
        async fn get_tenant_id_by_user_account_id(
            &self,
            user_account_id: i64,
        ) -> Result<Option<String>, ErrResp> {
            if self.user_account_id != user_account_id {
                return Ok(None);
            }
            Ok(Some(self.tenant_id.clone()))
        }
    }

    #[tokio::test]

    async fn get_tenant_id_by_user_account_id_internal_success() {
        let user_account_id = 64431;
        let tenant_id = "97cf9e78f6c74f4bac1c9bf0cf4cffba";
        let op_mock = TenantIdOperationMock {
            user_account_id,
            tenant_id: tenant_id.to_string(),
        };

        let result = get_tenant_id_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            tenant_id,
            resp.1 .0.tenant_id.expect("failed to get tenant_id")
        );
    }

    #[tokio::test]

    async fn get_tenant_id_by_user_account_id_internal_success_no_tenant_id_found() {
        let user_account_id = 64431;
        let tenant_id = "97cf9e78f6c74f4bac1c9bf0cf4cffba";
        let op_mock = TenantIdOperationMock {
            user_account_id,
            tenant_id: tenant_id.to_string(),
        };
        let dummy_id = user_account_id + 451;

        let result = get_tenant_id_by_user_account_id_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(None, resp.1 .0.tenant_id);
    }

    #[tokio::test]
    async fn get_tenant_id_by_user_account_id_internal_fail_user_account_id_is_zero() {
        let user_account_id = 0;
        let tenant_id = "97cf9e78f6c74f4bac1c9bf0cf4cffba";
        let op_mock = TenantIdOperationMock {
            user_account_id,
            tenant_id: tenant_id.to_string(),
        };

        let result = get_tenant_id_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_tenant_id_by_user_account_id_internal_fail_user_account_id_is_negative() {
        let user_account_id = -1;
        let tenant_id = "97cf9e78f6c74f4bac1c9bf0cf4cffba";
        let op_mock = TenantIdOperationMock {
            user_account_id,
            tenant_id: tenant_id.to_string(),
        };

        let result = get_tenant_id_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }
}
