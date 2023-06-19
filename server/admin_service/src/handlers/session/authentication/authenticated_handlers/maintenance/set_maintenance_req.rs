use axum::async_trait;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::{RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::handlers::session::authentication::authenticated_handlers::admin::Admin;

pub(crate) async fn post_set_maintenance_req(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
    Json(req): Json<SetMaintenanceReq>,
) -> RespResult<SetMaintenanceReqResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = SetMaintenanceReqOperationImpl { pool };
    handle_set_maintenance_req(
        req.start_time_in_jst,
        req.end_time_in_jst,
        current_date_time,
        &op,
    )
    .await
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
pub(crate) struct SetMaintenanceReq {
    start_time_in_jst: MaintenanceTime,
    end_time_in_jst: MaintenanceTime,
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
struct MaintenanceTime {
    year: u16, // 西暦
    day: u8,
    hour: u8,
    minute: u8,
    second: u8,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct SetMaintenanceReqResult {}

#[async_trait]
trait SetMaintenanceReqOperation {}

struct SetMaintenanceReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl SetMaintenanceReqOperation for SetMaintenanceReqOperationImpl {}

async fn handle_set_maintenance_req(
    start_time_in_jst: MaintenanceTime,
    end_time_in_jst: MaintenanceTime,
    current_date_time: DateTime<FixedOffset>,
    op: &impl SetMaintenanceReqOperation,
) -> RespResult<SetMaintenanceReqResult> {
    todo!()
}
