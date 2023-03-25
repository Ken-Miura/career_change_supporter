// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, ColumnTrait, DatabaseConnection, EntityTrait,
    PaginatorTrait, QueryFilter, Set,
};
use serde::Serialize;
use totp_rs::Secret;
use tracing::error;

use crate::err::Code;
use crate::mfa::{ensure_mfa_is_not_enabled, MAX_NUM_OF_TEMP_MFA_SECRETS};
use crate::{err::unexpected_err_resp, util::session::user::User};

const VALID_PERIOD_IN_MINUTE: i64 = 15;

pub(crate) async fn post_temp_mfa_secret(
    User { user_info }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<PostTempMfaSecretResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let base32_encoded_secret = generate_base32_encoded_secret()?;
    let op = TempMfaSecretResultOperationImpl { pool };
    handle_temp_mfp_secret(
        user_info.account_id,
        user_info.mfa_enabled_at.is_some(),
        base32_encoded_secret,
        current_date_time,
        op,
    )
    .await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct PostTempMfaSecretResult {}

fn generate_base32_encoded_secret() -> Result<String, ErrResp> {
    let secret = Secret::generate_secret().to_encoded();
    match secret {
        Secret::Raw(raw_secret) => {
            error!("Secret::Raw is unexpected (value: {:?})", raw_secret);
            Err(unexpected_err_resp())
        }
        Secret::Encoded(base32_encoded_secret) => Ok(base32_encoded_secret),
    }
}

async fn handle_temp_mfp_secret(
    account_id: i64,
    mfa_enabled: bool,
    base32_encoded_secret: String,
    current_date_time: DateTime<FixedOffset>,
    op: impl TempMfaSecretResultOperation,
) -> RespResult<PostTempMfaSecretResult> {
    ensure_mfa_is_not_enabled(mfa_enabled)?;
    ensure_temp_mfa_secre_does_not_reach_max_count(account_id, MAX_NUM_OF_TEMP_MFA_SECRETS, &op)
        .await?;

    let expiry_date_time = current_date_time + chrono::Duration::minutes(VALID_PERIOD_IN_MINUTE);
    op.create_temp_mfa_secret(account_id, base32_encoded_secret.clone(), expiry_date_time)
        .await?;

    Ok((StatusCode::OK, Json(PostTempMfaSecretResult {})))
}

async fn ensure_temp_mfa_secre_does_not_reach_max_count(
    account_id: i64,
    max_count: u64,
    op: &impl TempMfaSecretResultOperation,
) -> Result<(), ErrResp> {
    let count = op.count_temp_mfa_secret(account_id).await?;
    if count >= max_count {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ReachTempMfaSecretLimit as u32,
            }),
        ));
    }
    Ok(())
}

#[async_trait]
trait TempMfaSecretResultOperation {
    async fn count_temp_mfa_secret(&self, account_id: i64) -> Result<u64, ErrResp>;

    async fn create_temp_mfa_secret(
        &self,
        account_id: i64,
        base32_encoded_secret: String,
        expiry_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct TempMfaSecretResultOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl TempMfaSecretResultOperation for TempMfaSecretResultOperationImpl {
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
        expiry_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        let active_model = entity::temp_mfa_secret::ActiveModel {
            temp_mfa_secret_id: NotSet,
            user_account_id: Set(account_id),
            base32_encoded_secret: Set(base32_encoded_secret),
            expired_at: Set(expiry_date_time),
        };
        let _ = active_model.insert(&self.pool).await.map_err(|e| {
            error!(
                "failed to insert temp_mfa_secret (user_account_id: {}, expired_at: {}): {}",
                account_id, expiry_date_time, e
            );
            unexpected_err_resp()
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::generate_base32_encoded_secret;

    #[test]
    fn generate_base32_encoded_secret_finish_successfully() {
        // 出力される文字列は、シードを受け付けるパラメータがなく、完全ランダムなため入出力を指定したテストの記述は出来ない
        // ただ、関数の実行にあたって、Errが返されたり、panicが発生したりせず無事に完了することは確かめておく
        let _ = generate_base32_encoded_secret().expect("failed to get Ok");
    }

    // TODO: Add test
}
