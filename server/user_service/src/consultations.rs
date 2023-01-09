// Copyright 2023 Ken Miura

use axum::extract::State;
use chrono::Utc;
use common::{RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use crate::util::session::User;

pub(crate) async fn get_consultations(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
) -> RespResult<ConsultationsResult> {
    let _current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    todo!()
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationsResult {}
