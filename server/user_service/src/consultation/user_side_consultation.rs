// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::util::session::User;

use super::SkyWayCredential;

pub(crate) async fn get_user_side_consultation(
    User { account_id }: User,
    query: Query<UserSideConsultationQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<UserSideConsultationResult> {
    todo!()
}

#[derive(Deserialize)]
pub(crate) struct UserSideConsultationQuery {
    pub(crate) consultation_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct UserSideConsultationResult {
    pub(crate) user_account_peer_id: String,
    pub(crate) credential: SkyWayCredential,
    pub(crate) consultant_peer_id: Option<String>,
}
