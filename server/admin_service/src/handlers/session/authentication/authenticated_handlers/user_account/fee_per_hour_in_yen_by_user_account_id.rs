// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::{ErrResp, RespResult};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;

use super::super::admin::Admin;
use super::UserAccountIdQuery;

pub(crate) async fn get_fee_per_hour_in_yen_by_user_account_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<UserAccountIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<FeePerHourInYenResult> {
    let query = query.0;
    let op = FeePerHourInYenOperationImpl { pool };
    get_fee_per_hour_in_yen_by_user_account_id_internal(query.user_account_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct FeePerHourInYenResult {
    fee_per_hour_in_yen: Option<i32>,
}

async fn get_fee_per_hour_in_yen_by_user_account_id_internal(
    user_account_id: i64,
    op: impl FeePerHourInYenOperation,
) -> RespResult<FeePerHourInYenResult> {
    let fee_per_hour_in_yen = op
        .get_fee_per_hour_in_yen_by_user_account_id(user_account_id)
        .await?;
    Ok((
        StatusCode::OK,
        Json(FeePerHourInYenResult {
            fee_per_hour_in_yen,
        }),
    ))
}

#[async_trait]
trait FeePerHourInYenOperation {
    async fn get_fee_per_hour_in_yen_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<i32>, ErrResp>;
}

struct FeePerHourInYenOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl FeePerHourInYenOperation for FeePerHourInYenOperationImpl {
    async fn get_fee_per_hour_in_yen_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<i32>, ErrResp> {
        let result = entity::consulting_fee::Entity::find_by_id(user_account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find consulting_fee (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(result.map(|m| m.fee_per_hour_in_yen))
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::ErrResp;

    use super::*;

    struct FeePerHourInYenOperationMock {
        user_account_id: i64,
        fee_per_hour_in_yen: i32,
    }

    #[async_trait]
    impl FeePerHourInYenOperation for FeePerHourInYenOperationMock {
        async fn get_fee_per_hour_in_yen_by_user_account_id(
            &self,
            user_account_id: i64,
        ) -> Result<Option<i32>, ErrResp> {
            if self.user_account_id != user_account_id {
                return Ok(None);
            }
            Ok(Some(self.fee_per_hour_in_yen))
        }
    }

    #[tokio::test]

    async fn get_fee_per_hour_in_yen_by_user_account_id_internal_success() {
        let user_account_id = 64431;
        let fee_per_hour_in_yen = 5000;
        let op_mock = FeePerHourInYenOperationMock {
            user_account_id,
            fee_per_hour_in_yen,
        };

        let result =
            get_fee_per_hour_in_yen_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            fee_per_hour_in_yen,
            resp.1
                 .0
                .fee_per_hour_in_yen
                .expect("failed to get fee_per_hour_in_yen")
        );
    }

    #[tokio::test]

    async fn get_fee_per_hour_in_yen_by_user_account_id_internal_success_no_fee_per_hour_in_yen_found(
    ) {
        let user_account_id = 64431;
        let fee_per_hour_in_yen = 7000;
        let op_mock = FeePerHourInYenOperationMock {
            user_account_id,
            fee_per_hour_in_yen,
        };
        let dummy_id = user_account_id + 451;

        let result = get_fee_per_hour_in_yen_by_user_account_id_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(None, resp.1 .0.fee_per_hour_in_yen);
    }
}
