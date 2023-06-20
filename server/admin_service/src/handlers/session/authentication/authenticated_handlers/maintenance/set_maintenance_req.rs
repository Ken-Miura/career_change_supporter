use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, TimeZone, Utc};
use common::util::Maintenance;
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
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
trait SetMaintenanceReqOperation {
    async fn filter_maintenance_by_maintenance_end_at(
        &self,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<Maintenance>, ErrResp>;
}

struct SetMaintenanceReqOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl SetMaintenanceReqOperation for SetMaintenanceReqOperationImpl {
    async fn filter_maintenance_by_maintenance_end_at(
        &self,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<Maintenance>, ErrResp> {
        let maintenances = entity::maintenance::Entity::find()
            .filter(entity::maintenance::Column::MaintenanceEndAt.gte(current_date_time))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter maintenance (current_date_time: {}): {}",
                    current_date_time, e
                );
                unexpected_err_resp()
            })?;
        Ok(maintenances
            .into_iter()
            .map(|m| Maintenance {
                maintenance_start_at_in_jst: m
                    .maintenance_start_at
                    .with_timezone(&*JAPANESE_TIME_ZONE),
                maintenance_end_at_in_jst: m.maintenance_end_at.with_timezone(&*JAPANESE_TIME_ZONE),
            })
            .collect::<Vec<Maintenance>>())
    }
}

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
    ensure_there_is_no_overwrap(current_date_time, st, et, op).await?;
    // 設定＋決済の停止
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

async fn ensure_there_is_no_overwrap(
    current_date_time: DateTime<FixedOffset>,
    maintenance_start_time: DateTime<FixedOffset>,
    maintenance_end_time: DateTime<FixedOffset>,
    op: &impl SetMaintenanceReqOperation,
) -> Result<(), ErrResp> {
    let existing_maintenances = op
        .filter_maintenance_by_maintenance_end_at(current_date_time)
        .await?;
    for existing_maintenance in existing_maintenances {
        if existing_maintenance.maintenance_start_at_in_jst <= maintenance_start_time
            && maintenance_start_time <= existing_maintenance.maintenance_end_at_in_jst
        {
            error!(
                "maintenance_start_time {} is in {:?}",
                maintenance_start_time, existing_maintenance
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::MaintenanceAlreadyHasBeenSet as u32,
                }),
            ));
        }
        if existing_maintenance.maintenance_start_at_in_jst <= maintenance_end_time
            && maintenance_end_time <= existing_maintenance.maintenance_end_at_in_jst
        {
            error!(
                "maintenance_end_time {} is in {:?}",
                maintenance_end_time, existing_maintenance
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::MaintenanceAlreadyHasBeenSet as u32,
                }),
            ));
        }
    }
    Ok(())
}
