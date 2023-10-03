// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;

use super::super::admin::Admin;
use super::super::{validate_consultation_id_is_positive, ConsultationIdQuery};
use crate::handlers::session::authentication::authenticated_handlers::Consultation;

pub(crate) async fn get_consultation_by_consultation_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<ConsultationIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<ConsultationResult> {
    let query = query.0;
    let op = ConsultationOperationImpl { pool };
    get_consultation_by_consultation_id_internal(query.consultation_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct ConsultationResult {
    consultation: Option<Consultation>,
}

async fn get_consultation_by_consultation_id_internal(
    consultation_id: i64,
    op: impl ConsultationOperation,
) -> RespResult<ConsultationResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    let consultation = op
        .get_consultation_by_consultation_id(consultation_id)
        .await?;
    Ok((StatusCode::OK, Json(ConsultationResult { consultation })))
}

#[async_trait]
trait ConsultationOperation {
    async fn get_consultation_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<Consultation>, ErrResp>;
}

struct ConsultationOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationOperation for ConsultationOperationImpl {
    async fn get_consultation_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<Consultation>, ErrResp> {
        let model = entity::consultation::Entity::find_by_id(consultation_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find consultation (consultation_id: {}): {}",
                    consultation_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| Consultation {
            consultation_id: m.consultation_id,
            user_account_id: m.user_account_id,
            consultant_id: m.consultant_id,
            meeting_at: m
                .meeting_at
                .with_timezone(&(*JAPANESE_TIME_ZONE))
                .to_rfc3339(),
            room_name: m.room_name,
            user_account_entered_at: m
                .user_account_entered_at
                .map(|dt| dt.with_timezone(&(*JAPANESE_TIME_ZONE)).to_rfc3339()),
            consultant_entered_at: m
                .consultant_entered_at
                .map(|dt| dt.with_timezone(&(*JAPANESE_TIME_ZONE)).to_rfc3339()),
        }))
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::ErrResp;

    use crate::err::Code;

    use super::*;

    struct ConsultationOperationMock {
        consultation_id: i64,
        consultation: Consultation,
    }

    #[async_trait]
    impl ConsultationOperation for ConsultationOperationMock {
        async fn get_consultation_by_consultation_id(
            &self,
            consultation_id: i64,
        ) -> Result<Option<Consultation>, ErrResp> {
            if self.consultation_id != consultation_id {
                return Ok(None);
            }
            Ok(Some(self.consultation.clone()))
        }
    }

    fn create_dummy_consultation1(consultation_id: i64) -> Consultation {
        Consultation {
            consultation_id,
            user_account_id: 10,
            consultant_id: 510,
            meeting_at: "2023-04-13T14:00:00.0000+09:00 ".to_string(),
            room_name: "1241cd91a9c3433f98d16f40f51a5090".to_string(),
            user_account_entered_at: None,
            consultant_entered_at: None,
        }
    }

    #[tokio::test]

    async fn get_consultation_by_consultation_id_internal_success_1_result() {
        let consultation_id = 64431;
        let consultation1 = create_dummy_consultation1(consultation_id);
        let op_mock = ConsultationOperationMock {
            consultation_id,
            consultation: consultation1.clone(),
        };

        let result = get_consultation_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Some(consultation1), resp.1 .0.consultation);
    }

    #[tokio::test]

    async fn get_consultation_by_consultation_id_internal_success_no_result() {
        let consultation_id = 64431;
        let consultation1 = create_dummy_consultation1(consultation_id);
        let op_mock = ConsultationOperationMock {
            consultation_id,
            consultation: consultation1,
        };
        let dummy_id = consultation_id + 501;

        let result = get_consultation_by_consultation_id_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(None, resp.1 .0.consultation);
    }

    #[tokio::test]
    async fn get_consultation_by_consultation_id_internal_fail_consultation_id_is_zero() {
        let consultation_id = 0;
        let consultation1 = create_dummy_consultation1(consultation_id);
        let op_mock = ConsultationOperationMock {
            consultation_id,
            consultation: consultation1,
        };

        let result = get_consultation_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_consultation_by_consultation_id_internal_fail_consultation_id_is_negative() {
        let consultation_id = -1;
        let consultation1 = create_dummy_consultation1(consultation_id);
        let op_mock = ConsultationOperationMock {
            consultation_id,
            consultation: consultation1,
        };

        let result = get_consultation_by_consultation_id_internal(consultation_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::ConsultationIdIsNotPositive as u32)
    }
}
