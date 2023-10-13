// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, Duration, FixedOffset, TimeZone, Utc};
use common::util::Maintenance;
use common::{ApiError, ErrResp, RespResult, JAPANESE_TIME_ZONE, LENGTH_OF_MEETING_IN_MINUTE};
use entity::sea_orm::ActiveValue::NotSet;
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::authentication::authenticated_handlers::admin::Admin;

const MAX_MAINTENANCE_DURATION_IN_HOURS: i64 = 72;

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

    async fn set_maintenance(
        &self,
        start_time: DateTime<FixedOffset>,
        end_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
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
                maintenance_id: m.maintenance_id,
                maintenance_start_at_in_jst: m
                    .maintenance_start_at
                    .with_timezone(&*JAPANESE_TIME_ZONE),
                maintenance_end_at_in_jst: m.maintenance_end_at.with_timezone(&*JAPANESE_TIME_ZONE),
            })
            .collect::<Vec<Maintenance>>())
    }

    async fn set_maintenance(
        &self,
        start_time: DateTime<FixedOffset>,
        end_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        let m = entity::maintenance::ActiveModel {
            maintenance_id: NotSet,
            maintenance_start_at: Set(start_time),
            maintenance_end_at: Set(end_time),
        };
        let _ = m.insert(&self.pool).await.map_err(|e| {
            error!(
                "failed to insert maintenance (start_time: {}, end_time: {}): {}",
                start_time, end_time, e
            );
            unexpected_err_resp()
        })?;
        Ok(())
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
    let maintenance_time = et - st;
    let limit = chrono::Duration::hours(MAX_MAINTENANCE_DURATION_IN_HOURS);
    if maintenance_time > limit {
        error!(
            "maintenance time ({}) exceeds max maintenance duration ({})",
            maintenance_time, limit
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ExceedsMaxMaintenanceDurationLimit as u32,
            }),
        ));
    }
    ensure_there_is_no_overwrap(current_date_time, st, et, op).await?;

    op.set_maintenance(st, et).await?;

    Ok((StatusCode::OK, Json(SetMaintenanceReqResult {})))
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
        // ２つの時間帯が重なる条件（重ならない条件をド・モルガンの法則で反転）
        // 参考: https://yucatio.hatenablog.com/entry/2018/08/16/175914
        if existing_maintenance.maintenance_end_at_in_jst > maintenance_start_time
            && maintenance_end_time > existing_maintenance.maintenance_start_at_in_jst
        {
            error!(
                "maintenance_start_time ({}), maintenance_end_time ({}) is wrapped with {:?}",
                maintenance_start_time, maintenance_end_time, existing_maintenance
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

// データベースにアクセスする際のSQLの間違いや
// 依存するORMの間違いがあった場合に備えて、本当に決済の停止対象であることを確認する
// 本来であれば不要な処理だが、お金に関連する内容なのでアプリケーション側でもテストできるようにコードを書いておく
fn ensure_there_is_overwrap_between_meeting_and_maintenance(
    maintenance_start_time: DateTime<FixedOffset>,
    maintenance_end_time: DateTime<FixedOffset>,
    meeting_date_times: &Vec<DateTime<FixedOffset>>,
) -> Result<(), ErrResp> {
    for meeting_date_time in meeting_date_times {
        let meeting_start_time = *meeting_date_time;
        let meeting_end_time =
            *meeting_date_time + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64);
        // ２つの時間帯が重ならない条件
        // 参考: https://yucatio.hatenablog.com/entry/2018/08/16/175914
        if maintenance_end_time <= meeting_start_time || meeting_end_time <= maintenance_start_time
        {
            error!(
                "thre is no overwrap between meeting ({} to {}) and maintenance ({} to {})",
                meeting_start_time, meeting_end_time, maintenance_start_time, maintenance_end_time
            );
            return Err(unexpected_err_resp());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    struct SetMaintenanceReqOperationMock {
        current_date_time: DateTime<FixedOffset>,
        maintenances: Vec<Maintenance>,
        start_time: DateTime<FixedOffset>,
        end_time: DateTime<FixedOffset>,
    }

    #[async_trait]
    impl SetMaintenanceReqOperation for SetMaintenanceReqOperationMock {
        async fn filter_maintenance_by_maintenance_end_at(
            &self,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<Vec<Maintenance>, ErrResp> {
            assert_eq!(self.current_date_time, current_date_time);
            Ok(self.maintenances.clone())
        }

        async fn set_maintenance(
            &self,
            start_time: DateTime<FixedOffset>,
            end_time: DateTime<FixedOffset>,
        ) -> Result<(), ErrResp> {
            assert_eq!(self.start_time, start_time);
            assert_eq!(self.end_time, end_time);
            Ok(())
        }
    }

    #[tokio::test]
    async fn handle_set_maintenance_req_success() {
        let start_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 12,
            minute: 0,
            second: 0,
        };
        let end_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 16,
            minute: 0,
            second: 0,
        };
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 21, 13, 52, 24)
            .unwrap();
        let op = SetMaintenanceReqOperationMock {
            current_date_time,
            maintenances: vec![],
            start_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    start_time_in_jst.year as i32,
                    start_time_in_jst.month as u32,
                    start_time_in_jst.day as u32,
                    start_time_in_jst.hour as u32,
                    start_time_in_jst.minute as u32,
                    start_time_in_jst.second as u32,
                )
                .unwrap(),
            end_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    end_time_in_jst.year as i32,
                    end_time_in_jst.month as u32,
                    end_time_in_jst.day as u32,
                    end_time_in_jst.hour as u32,
                    end_time_in_jst.minute as u32,
                    end_time_in_jst.second as u32,
                )
                .unwrap(),
        };

        let result =
            handle_set_maintenance_req(start_time_in_jst, end_time_in_jst, current_date_time, &op)
                .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(resp.0, StatusCode::OK);
        assert_eq!(resp.1 .0, SetMaintenanceReqResult {});
    }

    #[tokio::test]
    async fn handle_set_maintenance_req_success_max_maintenance_duration() {
        let start_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 12,
            minute: 0,
            second: 0,
        };
        let end_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 26,
            hour: 12,
            minute: 0,
            second: 0,
        };
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 21, 13, 52, 24)
            .unwrap();
        let op = SetMaintenanceReqOperationMock {
            current_date_time,
            maintenances: vec![],
            start_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    start_time_in_jst.year as i32,
                    start_time_in_jst.month as u32,
                    start_time_in_jst.day as u32,
                    start_time_in_jst.hour as u32,
                    start_time_in_jst.minute as u32,
                    start_time_in_jst.second as u32,
                )
                .unwrap(),
            end_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    end_time_in_jst.year as i32,
                    end_time_in_jst.month as u32,
                    end_time_in_jst.day as u32,
                    end_time_in_jst.hour as u32,
                    end_time_in_jst.minute as u32,
                    end_time_in_jst.second as u32,
                )
                .unwrap(),
        };

        let result =
            handle_set_maintenance_req(start_time_in_jst, end_time_in_jst, current_date_time, &op)
                .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(resp.0, StatusCode::OK);
        assert_eq!(resp.1 .0, SetMaintenanceReqResult {});
    }

    #[tokio::test]
    async fn handle_set_maintenance_req_success_no_overwrap_maintenances() {
        let start_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 12,
            minute: 0,
            second: 0,
        };
        let end_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 16,
            minute: 0,
            second: 0,
        };
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 21, 13, 52, 24)
            .unwrap();
        let m1 = Maintenance {
            maintenance_id: 1,
            maintenance_start_at_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 8, 0, 0)
                .unwrap(),
            maintenance_end_at_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 12, 0, 0)
                .unwrap(),
        };
        let m2 = Maintenance {
            maintenance_id: 2,
            maintenance_start_at_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 16, 0, 0)
                .unwrap(),
            maintenance_end_at_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 20, 0, 0)
                .unwrap(),
        };
        let op = SetMaintenanceReqOperationMock {
            current_date_time,
            maintenances: vec![m1, m2],
            start_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    start_time_in_jst.year as i32,
                    start_time_in_jst.month as u32,
                    start_time_in_jst.day as u32,
                    start_time_in_jst.hour as u32,
                    start_time_in_jst.minute as u32,
                    start_time_in_jst.second as u32,
                )
                .unwrap(),
            end_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    end_time_in_jst.year as i32,
                    end_time_in_jst.month as u32,
                    end_time_in_jst.day as u32,
                    end_time_in_jst.hour as u32,
                    end_time_in_jst.minute as u32,
                    end_time_in_jst.second as u32,
                )
                .unwrap(),
        };

        let result =
            handle_set_maintenance_req(start_time_in_jst, end_time_in_jst, current_date_time, &op)
                .await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(resp.0, StatusCode::OK);
        assert_eq!(resp.1 .0, SetMaintenanceReqResult {});
    }

    #[tokio::test]
    async fn handle_set_maintenance_req_fail_overwrap_maintenances1() {
        let start_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 12,
            minute: 0,
            second: 0,
        };
        let end_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 16,
            minute: 0,
            second: 0,
        };
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 21, 13, 52, 24)
            .unwrap();
        let m1 = Maintenance {
            maintenance_id: 1,
            maintenance_start_at_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 11, 0, 0)
                .unwrap(),
            maintenance_end_at_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 13, 0, 0)
                .unwrap(),
        };
        let op = SetMaintenanceReqOperationMock {
            current_date_time,
            maintenances: vec![m1],
            start_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    start_time_in_jst.year as i32,
                    start_time_in_jst.month as u32,
                    start_time_in_jst.day as u32,
                    start_time_in_jst.hour as u32,
                    start_time_in_jst.minute as u32,
                    start_time_in_jst.second as u32,
                )
                .unwrap(),
            end_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    end_time_in_jst.year as i32,
                    end_time_in_jst.month as u32,
                    end_time_in_jst.day as u32,
                    end_time_in_jst.hour as u32,
                    end_time_in_jst.minute as u32,
                    end_time_in_jst.second as u32,
                )
                .unwrap(),
        };

        let result =
            handle_set_maintenance_req(start_time_in_jst, end_time_in_jst, current_date_time, &op)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::MaintenanceAlreadyHasBeenSet as u32);
    }

    #[tokio::test]
    async fn handle_set_maintenance_req_fail_overwrap_maintenances2() {
        let start_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 12,
            minute: 0,
            second: 0,
        };
        let end_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 16,
            minute: 0,
            second: 0,
        };
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 21, 13, 52, 24)
            .unwrap();
        let m1 = Maintenance {
            maintenance_id: 1,
            maintenance_start_at_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 15, 0, 0)
                .unwrap(),
            maintenance_end_at_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 18, 0, 0)
                .unwrap(),
        };
        let op = SetMaintenanceReqOperationMock {
            current_date_time,
            maintenances: vec![m1],
            start_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    start_time_in_jst.year as i32,
                    start_time_in_jst.month as u32,
                    start_time_in_jst.day as u32,
                    start_time_in_jst.hour as u32,
                    start_time_in_jst.minute as u32,
                    start_time_in_jst.second as u32,
                )
                .unwrap(),
            end_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    end_time_in_jst.year as i32,
                    end_time_in_jst.month as u32,
                    end_time_in_jst.day as u32,
                    end_time_in_jst.hour as u32,
                    end_time_in_jst.minute as u32,
                    end_time_in_jst.second as u32,
                )
                .unwrap(),
        };

        let result =
            handle_set_maintenance_req(start_time_in_jst, end_time_in_jst, current_date_time, &op)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::MaintenanceAlreadyHasBeenSet as u32);
    }

    #[tokio::test]
    async fn handle_set_maintenance_req_fail_overwrap_maintenances3() {
        let start_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 12,
            minute: 0,
            second: 0,
        };
        let end_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 16,
            minute: 0,
            second: 0,
        };
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 21, 13, 52, 24)
            .unwrap();
        let m1 = Maintenance {
            maintenance_id: 1,
            maintenance_start_at_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 11, 0, 0)
                .unwrap(),
            maintenance_end_at_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 17, 0, 0)
                .unwrap(),
        };
        let op = SetMaintenanceReqOperationMock {
            current_date_time,
            maintenances: vec![m1],
            start_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    start_time_in_jst.year as i32,
                    start_time_in_jst.month as u32,
                    start_time_in_jst.day as u32,
                    start_time_in_jst.hour as u32,
                    start_time_in_jst.minute as u32,
                    start_time_in_jst.second as u32,
                )
                .unwrap(),
            end_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    end_time_in_jst.year as i32,
                    end_time_in_jst.month as u32,
                    end_time_in_jst.day as u32,
                    end_time_in_jst.hour as u32,
                    end_time_in_jst.minute as u32,
                    end_time_in_jst.second as u32,
                )
                .unwrap(),
        };

        let result =
            handle_set_maintenance_req(start_time_in_jst, end_time_in_jst, current_date_time, &op)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::MaintenanceAlreadyHasBeenSet as u32);
    }

    #[tokio::test]
    async fn handle_set_maintenance_req_fail_overwrap_maintenances4() {
        let start_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 12,
            minute: 0,
            second: 0,
        };
        let end_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 16,
            minute: 0,
            second: 0,
        };
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 21, 13, 52, 24)
            .unwrap();
        let m1 = Maintenance {
            maintenance_id: 1,
            maintenance_start_at_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 13, 0, 0)
                .unwrap(),
            maintenance_end_at_in_jst: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(2023, 6, 23, 15, 0, 0)
                .unwrap(),
        };
        let op = SetMaintenanceReqOperationMock {
            current_date_time,
            maintenances: vec![m1],
            start_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    start_time_in_jst.year as i32,
                    start_time_in_jst.month as u32,
                    start_time_in_jst.day as u32,
                    start_time_in_jst.hour as u32,
                    start_time_in_jst.minute as u32,
                    start_time_in_jst.second as u32,
                )
                .unwrap(),
            end_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    end_time_in_jst.year as i32,
                    end_time_in_jst.month as u32,
                    end_time_in_jst.day as u32,
                    end_time_in_jst.hour as u32,
                    end_time_in_jst.minute as u32,
                    end_time_in_jst.second as u32,
                )
                .unwrap(),
        };

        let result =
            handle_set_maintenance_req(start_time_in_jst, end_time_in_jst, current_date_time, &op)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::MaintenanceAlreadyHasBeenSet as u32);
    }

    #[tokio::test]
    async fn handle_set_maintenance_req_fail_illegal_date_time1() {
        let start_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 25,
            minute: 0,
            second: 0,
        };
        let end_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 24,
            hour: 16,
            minute: 0,
            second: 0,
        };
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 21, 13, 52, 24)
            .unwrap();
        let op = SetMaintenanceReqOperationMock {
            current_date_time,
            maintenances: vec![],
            start_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    start_time_in_jst.year as i32,
                    start_time_in_jst.month as u32,
                    start_time_in_jst.day as u32,
                    12, // start_time_in_jstをそのまま使うとエラーになるので固定値を使う
                    start_time_in_jst.minute as u32,
                    start_time_in_jst.second as u32,
                )
                .unwrap(),
            end_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    end_time_in_jst.year as i32,
                    end_time_in_jst.month as u32,
                    end_time_in_jst.day as u32,
                    end_time_in_jst.hour as u32,
                    end_time_in_jst.minute as u32,
                    end_time_in_jst.second as u32,
                )
                .unwrap(),
        };

        let result =
            handle_set_maintenance_req(start_time_in_jst, end_time_in_jst, current_date_time, &op)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::IllegalDateTime as u32);
    }

    #[tokio::test]
    async fn handle_set_maintenance_req_fail_illegal_date_time2() {
        let start_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 23,
            minute: 0,
            second: 0,
        };
        let end_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 32,
            hour: 16,
            minute: 0,
            second: 0,
        };
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 21, 13, 52, 24)
            .unwrap();
        let op = SetMaintenanceReqOperationMock {
            current_date_time,
            maintenances: vec![],
            start_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    start_time_in_jst.year as i32,
                    start_time_in_jst.month as u32,
                    start_time_in_jst.day as u32,
                    start_time_in_jst.hour as u32,
                    start_time_in_jst.minute as u32,
                    start_time_in_jst.second as u32,
                )
                .unwrap(),
            end_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    end_time_in_jst.year as i32,
                    end_time_in_jst.month as u32,
                    27, // end_time_in_jstをそのまま使うとエラーになるので固定値を使う
                    end_time_in_jst.hour as u32,
                    end_time_in_jst.minute as u32,
                    end_time_in_jst.second as u32,
                )
                .unwrap(),
        };

        let result =
            handle_set_maintenance_req(start_time_in_jst, end_time_in_jst, current_date_time, &op)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::IllegalDateTime as u32);
    }

    #[tokio::test]
    async fn handle_set_maintenance_req_fail_illegal_maintenance_date_time1() {
        let start_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 24,
            hour: 15,
            minute: 0,
            second: 0,
        };
        let end_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 24,
            hour: 18,
            minute: 0,
            second: 0,
        };
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 24, 18, 0, 0)
            .unwrap();
        let op = SetMaintenanceReqOperationMock {
            current_date_time,
            maintenances: vec![],
            start_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    start_time_in_jst.year as i32,
                    start_time_in_jst.month as u32,
                    start_time_in_jst.day as u32,
                    start_time_in_jst.hour as u32,
                    start_time_in_jst.minute as u32,
                    start_time_in_jst.second as u32,
                )
                .unwrap(),
            end_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    end_time_in_jst.year as i32,
                    end_time_in_jst.month as u32,
                    end_time_in_jst.day as u32,
                    end_time_in_jst.hour as u32,
                    end_time_in_jst.minute as u32,
                    end_time_in_jst.second as u32,
                )
                .unwrap(),
        };

        let result =
            handle_set_maintenance_req(start_time_in_jst, end_time_in_jst, current_date_time, &op)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::IllegalMaintenanceDateTime as u32);
    }

    #[tokio::test]
    async fn handle_set_maintenance_req_fail_illegal_maintenance_date_time2() {
        let start_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 24,
            hour: 15,
            minute: 0,
            second: 0,
        };
        let end_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 24,
            hour: 18,
            minute: 0,
            second: 0,
        };
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 24, 18, 0, 1)
            .unwrap();
        let op = SetMaintenanceReqOperationMock {
            current_date_time,
            maintenances: vec![],
            start_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    start_time_in_jst.year as i32,
                    start_time_in_jst.month as u32,
                    start_time_in_jst.day as u32,
                    start_time_in_jst.hour as u32,
                    start_time_in_jst.minute as u32,
                    start_time_in_jst.second as u32,
                )
                .unwrap(),
            end_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    end_time_in_jst.year as i32,
                    end_time_in_jst.month as u32,
                    end_time_in_jst.day as u32,
                    end_time_in_jst.hour as u32,
                    end_time_in_jst.minute as u32,
                    end_time_in_jst.second as u32,
                )
                .unwrap(),
        };

        let result =
            handle_set_maintenance_req(start_time_in_jst, end_time_in_jst, current_date_time, &op)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::IllegalMaintenanceDateTime as u32);
    }

    #[tokio::test]
    async fn handle_set_maintenance_req_fail_illegal_maintenance_date_time3() {
        let start_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 24,
            hour: 18,
            minute: 0,
            second: 0,
        };
        let end_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 24,
            hour: 18,
            minute: 0,
            second: 0,
        };
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 24, 13, 0, 0)
            .unwrap();
        let op = SetMaintenanceReqOperationMock {
            current_date_time,
            maintenances: vec![],
            start_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    start_time_in_jst.year as i32,
                    start_time_in_jst.month as u32,
                    start_time_in_jst.day as u32,
                    start_time_in_jst.hour as u32,
                    start_time_in_jst.minute as u32,
                    start_time_in_jst.second as u32,
                )
                .unwrap(),
            end_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    end_time_in_jst.year as i32,
                    end_time_in_jst.month as u32,
                    end_time_in_jst.day as u32,
                    end_time_in_jst.hour as u32,
                    end_time_in_jst.minute as u32,
                    end_time_in_jst.second as u32,
                )
                .unwrap(),
        };

        let result =
            handle_set_maintenance_req(start_time_in_jst, end_time_in_jst, current_date_time, &op)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::IllegalMaintenanceDateTime as u32);
    }

    #[tokio::test]
    async fn handle_set_maintenance_req_fail_illegal_maintenance_date_time4() {
        let start_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 24,
            hour: 18,
            minute: 0,
            second: 1,
        };
        let end_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 24,
            hour: 18,
            minute: 0,
            second: 0,
        };
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 24, 13, 0, 0)
            .unwrap();
        let op = SetMaintenanceReqOperationMock {
            current_date_time,
            maintenances: vec![],
            start_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    start_time_in_jst.year as i32,
                    start_time_in_jst.month as u32,
                    start_time_in_jst.day as u32,
                    start_time_in_jst.hour as u32,
                    start_time_in_jst.minute as u32,
                    start_time_in_jst.second as u32,
                )
                .unwrap(),
            end_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    end_time_in_jst.year as i32,
                    end_time_in_jst.month as u32,
                    end_time_in_jst.day as u32,
                    end_time_in_jst.hour as u32,
                    end_time_in_jst.minute as u32,
                    end_time_in_jst.second as u32,
                )
                .unwrap(),
        };

        let result =
            handle_set_maintenance_req(start_time_in_jst, end_time_in_jst, current_date_time, &op)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::IllegalMaintenanceDateTime as u32);
    }

    #[tokio::test]
    async fn handle_set_maintenance_req_fail_over_max_maintenance_duration() {
        let start_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 23,
            hour: 12,
            minute: 0,
            second: 0,
        };
        let end_time_in_jst = MaintenanceTime {
            year: 2023,
            month: 6,
            day: 26,
            hour: 12,
            minute: 0,
            second: 1,
        };
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 6, 21, 13, 52, 24)
            .unwrap();
        let op = SetMaintenanceReqOperationMock {
            current_date_time,
            maintenances: vec![],
            start_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    start_time_in_jst.year as i32,
                    start_time_in_jst.month as u32,
                    start_time_in_jst.day as u32,
                    start_time_in_jst.hour as u32,
                    start_time_in_jst.minute as u32,
                    start_time_in_jst.second as u32,
                )
                .unwrap(),
            end_time: JAPANESE_TIME_ZONE
                .with_ymd_and_hms(
                    end_time_in_jst.year as i32,
                    end_time_in_jst.month as u32,
                    end_time_in_jst.day as u32,
                    end_time_in_jst.hour as u32,
                    end_time_in_jst.minute as u32,
                    end_time_in_jst.second as u32,
                )
                .unwrap(),
        };

        let result =
            handle_set_maintenance_req(start_time_in_jst, end_time_in_jst, current_date_time, &op)
                .await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(
            resp.1 .0.code,
            Code::ExceedsMaxMaintenanceDurationLimit as u32
        );
    }
}
