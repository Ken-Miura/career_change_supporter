// Copyright 2022 Ken Miura

use axum::http::StatusCode;
use axum::{async_trait, Json};
use axum::{extract::Query, Extension};
use common::{ApiError, ErrResp, RespResult};
use entity::prelude::{ConsultingFee, UserAccount};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::util::session::User;

pub(crate) async fn get_fee_per_hour_in_yen_for_application(
    User { account_id }: User,
    query: Query<FeePerHourInYenForApplicationQuery>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<FeePerHourInYenForApplication> {
    let query = query.0;
    let op = FeePerHourInYenForApplicationOperationImpl { pool };
    handle_fee_per_hour_in_yen_for_application(account_id, query.consultant_id, op).await
}

#[derive(Deserialize)]
pub(crate) struct FeePerHourInYenForApplicationQuery {
    pub consultant_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct FeePerHourInYenForApplication {
    pub fee_per_hour_in_yen: i32,
}

#[async_trait]
trait FeePerHourInYenForApplicationOperation {
    /// Identityが存在するか確認する。存在する場合、trueを返す。そうでない場合、falseを返す。
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
    /// コンサルタントのUserAccountが存在するか確認する。存在する場合、trueを返す。そうでない場合、falseを返す。
    async fn check_if_consultant_exists(&self, consultant_id: i64) -> Result<bool, ErrResp>;
    async fn find_fee_per_hour_in_yen_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Option<i32>, ErrResp>;
}

struct FeePerHourInYenForApplicationOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl FeePerHourInYenForApplicationOperation for FeePerHourInYenForApplicationOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        let model = entity::prelude::Identity::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find identity (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.is_some())
    }

    async fn check_if_consultant_exists(&self, consultant_id: i64) -> Result<bool, ErrResp> {
        let model = UserAccount::find_by_id(consultant_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find user_account (user_account_id): {}): {}",
                    consultant_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.is_some())
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
}

async fn handle_fee_per_hour_in_yen_for_application(
    account_id: i64,
    consultant_id: i64,
    op: impl FeePerHourInYenForApplicationOperation,
) -> RespResult<FeePerHourInYenForApplication> {
    if !consultant_id.is_positive() {
        error!("consultant_id ({}) is not positive", consultant_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonPositiveConsultantId as u32,
            }),
        ));
    }
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
    let consultant_exists = op.check_if_consultant_exists(consultant_id).await?;
    if !consultant_exists {
        error!(
            "consultant does not exist (consultant_id: {})",
            consultant_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultantDoesNotExist as u32,
            }),
        ));
    }
    let fee_per_hour_in_yen = op
        .find_fee_per_hour_in_yen_by_consultant_id(account_id)
        .await?;
    let fee_per_hour_in_yen = fee_per_hour_in_yen.ok_or_else(|| {
        error!(
            "fee_per_hour_in_yen does not exist (consultant_id: {})",
            consultant_id
        );
        unexpected_err_resp()
    })?;
    Ok((
        StatusCode::OK,
        Json(FeePerHourInYenForApplication {
            fee_per_hour_in_yen,
        }),
    ))
}
