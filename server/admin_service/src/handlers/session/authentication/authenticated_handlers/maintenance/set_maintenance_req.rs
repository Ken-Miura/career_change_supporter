use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::Code;
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
    month: u8,
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
    let st = convert_maintenance_time_type(&start_time_in_jst)?;
    let et = convert_maintenance_time_type(&end_time_in_jst)?;
    if current_date_time >= et {
        error!(
            "current date time ({}) passes maintenance end time ({})",
            current_date_time, et
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::IllegalMaintenanceDateTime as u32,
            }),
        ));
    }
    if st >= et {
        error!(
            "maintenance start time ({}) is after maintenance end time ({})",
            st, et
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::IllegalMaintenanceDateTime as u32,
            }),
        ));
    }
    todo!()
}

fn convert_maintenance_time_type(mt: &MaintenanceTime) -> Result<DateTime<FixedOffset>, ErrResp> {
    let result = JAPANESE_TIME_ZONE
        .with_ymd_and_hms(
            mt.year as i32,
            mt.month as u32,
            mt.day as u32,
            mt.hour as u32,
            mt.minute as u32,
            mt.second as u32,
        )
        .single()
        .ok_or_else(|| {
            error!("illegal date time: {:?}", mt);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::IllegalDateTime as u32,
                }),
            )
        })?;
    Ok(result)
}
