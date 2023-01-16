// Copyright 2023 Ken Miura

use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::Utc;
use common::{RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::util::session::User;

use super::{generate_sky_way_credential_auth_token, SkyWayCredential, SKY_WAY_SECRET_KEY};

pub(crate) async fn get_user_side_consultation(
    User { account_id: _ }: User,
    _query: Query<UserSideConsultationQuery>,
    State(_pool): State<DatabaseConnection>,
) -> RespResult<UserSideConsultationResult> {
    let user_account_peer_id = "11b060e0b9f74e898c55afff5e12e399";
    let timestamp = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE)).timestamp();
    let ttl = 60 * 60;
    let auth_token = generate_sky_way_credential_auth_token(
        user_account_peer_id,
        timestamp,
        ttl,
        (*SKY_WAY_SECRET_KEY).as_str(),
    )?;
    let credential = SkyWayCredential {
        auth_token,
        ttl,
        timestamp,
    };
    Ok((
        StatusCode::OK,
        Json(UserSideConsultationResult {
            user_account_peer_id: user_account_peer_id.to_string(),
            credential,
            consultant_peer_id: None,
        }),
    ))
}

#[derive(Deserialize)]
pub(crate) struct UserSideConsultationQuery {
    pub(crate) _consultation_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct UserSideConsultationResult {
    pub(crate) user_account_peer_id: String,
    pub(crate) credential: SkyWayCredential,
    pub(crate) consultant_peer_id: Option<String>,
}
