// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, FixedOffset, Utc};
use common::util::validator::uuid_validator::validate_uuid;
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::err::{unexpected_err_resp, Code};
use crate::util;
use crate::util::session::User;

use super::{
    generate_sky_way_credential_auth_token, validate_consultation_id_is_positive, Consultation,
    SkyWayCredential, SKY_WAY_SECRET_KEY,
};

pub(crate) async fn get_user_side_info(
    User { account_id }: User,
    query: Query<UserSideInfoQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<UserSideInfoResult> {
    let consultation_id = query.0.consultation_id;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let peer_id = Uuid::new_v4().simple().to_string();
    let op = UserSideInfoOperationImpl { pool };
    handle_user_side_info(
        account_id,
        consultation_id,
        &current_date_time,
        peer_id.as_str(),
        op,
    )
    .await
}

#[derive(Deserialize)]
pub(crate) struct UserSideInfoQuery {
    consultation_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct UserSideInfoResult {
    user_account_peer_id: String,
    credential: SkyWayCredential,
    consultant_peer_id: Option<String>,
}

async fn handle_user_side_info(
    account_id: i64,
    consultation_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    peer_id: &str,
    op: impl UserSideInfoOperation,
) -> RespResult<UserSideInfoResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    validate_uuid(peer_id).map_err(|e| {
        error!("failed to validate {}: {}", peer_id, e);
        // peer_idは、ユーザーから渡されるものではなく、サーバで生成するものなので失敗はunexpected_err_resp
        unexpected_err_resp()
    })?;
    validate_identity_exists(account_id, &op).await?;
    let result = get_consultation_by_consultation_id(consultation_id, &op).await?;
    ensure_user_account_id_is_valid(result.user_account_id, account_id)?;
    // 時間チェック
    // Disableチェック（いらない？）
    todo!()
    // println!("{}", consultation_id);
    // let user_account_peer_id = "11b060e0b9f74e898c55afff5e12e399";
    // let timestamp = current_date_time.timestamp();
    // let ttl = 60 * 60;
    // let auth_token = generate_sky_way_credential_auth_token(
    //     user_account_peer_id,
    //     timestamp,
    //     ttl,
    //     (*SKY_WAY_SECRET_KEY).as_str(),
    // )?;
    // let credential = SkyWayCredential {
    //     auth_token,
    //     ttl,
    //     timestamp,
    // };
    // Ok((
    //     StatusCode::OK,
    //     Json(UserSideInfoResult {
    //         user_account_peer_id: user_account_peer_id.to_string(),
    //         credential,
    //         consultant_peer_id: None,
    //     }),
    // ))
}

#[async_trait]
trait UserSideInfoOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;

    async fn find_consultation_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<Consultation>, ErrResp>;
}

struct UserSideInfoOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl UserSideInfoOperation for UserSideInfoOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::identity_checker::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn find_consultation_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<Consultation>, ErrResp> {
        super::find_consultation_by_consultation_id(consultation_id, &self.pool).await
    }
}

async fn validate_identity_exists(
    account_id: i64,
    op: &impl UserSideInfoOperation,
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

async fn get_consultation_by_consultation_id(
    consultation_id: i64,
    op: &impl UserSideInfoOperation,
) -> Result<Consultation, ErrResp> {
    let consultation_option = op
        .find_consultation_by_consultation_id(consultation_id)
        .await?;
    if let Some(consultation) = consultation_option {
        Ok(consultation)
    } else {
        error!(
            "no consultation (consultation_id: {}) found",
            consultation_id
        );
        Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationFound as u32,
            }),
        ))
    }
}

fn ensure_user_account_id_is_valid(
    user_account_id_in_consultation: i64,
    user_account_id: i64,
) -> Result<(), ErrResp> {
    if user_account_id_in_consultation != user_account_id {
        error!(
            "user_account_id in consultation ({}) is not same as passed user_accound_id ({})",
            user_account_id_in_consultation, user_account_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationFound as u32,
            }),
        ));
    }
    Ok(())
}
