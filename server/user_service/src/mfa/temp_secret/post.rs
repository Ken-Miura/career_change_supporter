// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::extract::State;
use chrono::{DateTime, FixedOffset, Utc};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{
    ActiveValue::NotSet, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set, ActiveModelTrait,
};
use serde::Serialize;
use tracing::error;

use crate::{err::unexpected_err_resp, util::session::user::User};

pub(crate) async fn post_temp_mfa_secret(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<PostTempMfaSecretResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let base32_encoded_secret = "".to_string();
    let op = TempMfaSecretResultOperationImpl { pool };
    handle_temp_mfp_secret(account_id, base32_encoded_secret, current_date_time, op).await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct PostTempMfaSecretResult {
    // QRコード
    base64_encoded_image: String,
    // QRコードを読み込めない場合に使うシークレットキー
    base32_encoded_secret: String,
}

async fn handle_temp_mfp_secret(
    _account_id: i64,
    _base32_encoded_secret: String,
    _current_date_time: DateTime<FixedOffset>,
    _op: impl TempMfaSecretResultOperation,
) -> RespResult<PostTempMfaSecretResult> {
    // 既にMFAが有効かどうか確認
    // temp_mfa_secretが最大数作られていないか確認
    // temp_mfa_secretを作成
    todo!()
}

#[async_trait]
trait TempMfaSecretResultOperation {
    async fn check_if_mfa_is_enabled(&self, account_id: i64) -> Result<bool, ErrResp>;

    async fn count_temp_mfa_secret(&self, account_id: i64) -> Result<u64, ErrResp>;

    async fn create_temp_mfa_secret(
        &self,
        account_id: i64,
        base32_encoded_secret: String,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct TempMfaSecretResultOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl TempMfaSecretResultOperation for TempMfaSecretResultOperationImpl {
    async fn check_if_mfa_is_enabled(&self, account_id: i64) -> Result<bool, ErrResp> {
        let result = entity::user_account::Entity::find_by_id(account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find user_account by user_account_id ({}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        let result = result.ok_or_else(|| {
            error!("no user_account found (user_account_id: {})", account_id);
            unexpected_err_resp()
        })?;
        Ok(result.mfa_enabled_at.is_some())
    }

    async fn count_temp_mfa_secret(&self, account_id: i64) -> Result<u64, ErrResp> {
        let result = entity::temp_mfa_secret::Entity::find()
            .filter(entity::temp_mfa_secret::Column::UserAccountId.eq(account_id))
            .count(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to count temp_mfa_secret (account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(result)
    }

    async fn create_temp_mfa_secret(
        &self,
        account_id: i64,
        base32_encoded_secret: String,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        let active_model = entity::temp_mfa_secret::ActiveModel {
            temp_mfa_secret_id: NotSet,
            user_account_id: Set(account_id),
            base32_encoded_secret: Set(base32_encoded_secret),
            expired_at: Set(current_date_time),
        };
        let _ = active_model.insert(&self.pool).await.map_err(|e| {
            error!(
                "failed to insert temp_mfa_secret (user_account_id: {}, expired_at: {}): {}",
                account_id, current_date_time, e
            );
            unexpected_err_resp()
        })?;
        Ok(())
    }
}
