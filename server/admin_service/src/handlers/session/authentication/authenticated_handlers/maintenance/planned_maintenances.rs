// Copyright 2021 Ken Miura

use axum::extract::State;
use axum::Json;
use axum::{async_trait, http::StatusCode};
use chrono::{DateTime, FixedOffset, Utc};
use common::{util::Maintenance, ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use tracing::error;

use crate::{
    err::unexpected_err_resp,
    handlers::session::authentication::authenticated_handlers::admin::Admin,
};

pub(crate) async fn get_planned_maintenances(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    State(pool): State<DatabaseConnection>,
) -> RespResult<PlannedMaintenancesResult> {
    let op = PlannedMaintenancesOperationImpl { pool };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    handle_planned_maintenances(current_date_time, &op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct PlannedMaintenancesResult {
    planned_maintenances: Vec<PlannedMaintenance>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct PlannedMaintenance {
    maintenance_start_at_in_jst: String, // RFC 3339形式の文字列
    maintenance_end_at_in_jst: String,   // RFC 3339形式の文字列
    description: String,
}

#[async_trait]
trait PlannedMaintenancesOperation {
    async fn filter_maintenance_by_maintenance_end_at(
        &self,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<Maintenance>, ErrResp>;
}

struct PlannedMaintenancesOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl PlannedMaintenancesOperation for PlannedMaintenancesOperationImpl {
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
                description: m.description,
            })
            .collect::<Vec<Maintenance>>())
    }
}

async fn handle_planned_maintenances(
    current_date_time: DateTime<FixedOffset>,
    op: &impl PlannedMaintenancesOperation,
) -> RespResult<PlannedMaintenancesResult> {
    let results = op
        .filter_maintenance_by_maintenance_end_at(current_date_time)
        .await?;
    let planned_maintenances = results
        .into_iter()
        .map(|m| PlannedMaintenance {
            maintenance_start_at_in_jst: m
                .maintenance_start_at_in_jst
                .with_timezone(&(*JAPANESE_TIME_ZONE))
                .to_rfc3339(),
            maintenance_end_at_in_jst: m
                .maintenance_end_at_in_jst
                .with_timezone(&(*JAPANESE_TIME_ZONE))
                .to_rfc3339(),
            description: m.description,
        })
        .collect();
    Ok((
        StatusCode::OK,
        Json(PlannedMaintenancesResult {
            planned_maintenances,
        }),
    ))
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    struct PlannedMaintenancesOperationMock {
        current_date_time: DateTime<FixedOffset>,
        maintenances: Vec<Maintenance>,
    }

    #[async_trait]
    impl PlannedMaintenancesOperation for PlannedMaintenancesOperationMock {
        async fn filter_maintenance_by_maintenance_end_at(
            &self,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<Vec<Maintenance>, ErrResp> {
            assert_eq!(self.current_date_time, current_date_time);
            Ok(self.maintenances.clone())
        }
    }

    #[tokio::test]

    async fn handle_planned_maintenances_success_empty_result() {
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 11, 15, 30, 45)
            .unwrap();
        let op = PlannedMaintenancesOperationMock {
            current_date_time,
            maintenances: vec![],
        };

        let result = handle_planned_maintenances(current_date_time, &op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            Vec::<PlannedMaintenance>::with_capacity(0),
            resp.1 .0.planned_maintenances
        );
    }
}
