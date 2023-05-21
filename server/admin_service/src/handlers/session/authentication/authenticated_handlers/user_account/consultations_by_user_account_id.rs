// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use tracing::error;

use crate::err::unexpected_err_resp;

use super::super::admin::Admin;
use super::{
    validate_account_id_is_positive, Consultation, ConsultationsResult, UserAccountIdQuery,
};

pub(crate) async fn get_consultations_by_user_account_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<UserAccountIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<ConsultationsResult> {
    let query = query.0;
    let op = ConsultationsOperationImpl { pool };
    get_consultations_by_user_account_id_internal(query.user_account_id, op).await
}

async fn get_consultations_by_user_account_id_internal(
    user_account_id: i64,
    op: impl ConsultationsOperation,
) -> RespResult<ConsultationsResult> {
    validate_account_id_is_positive(user_account_id)?;
    let consultations = op
        .get_consultations_by_user_account_id(user_account_id)
        .await?;
    Ok((StatusCode::OK, Json(ConsultationsResult { consultations })))
}

#[async_trait]
trait ConsultationsOperation {
    async fn get_consultations_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<Consultation>, ErrResp>;
}

struct ConsultationsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationsOperation for ConsultationsOperationImpl {
    async fn get_consultations_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<Consultation>, ErrResp> {
        let models = entity::consultation::Entity::find()
            .filter(entity::consultation::Column::UserAccountId.eq(user_account_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter consultation (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| Consultation {
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
            })
            .collect::<Vec<Consultation>>())
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::ErrResp;

    use crate::err::Code;

    use super::*;

    struct ConsultationsOperationMock {
        user_account_id: i64,
        consultations: Vec<Consultation>,
    }

    #[async_trait]
    impl ConsultationsOperation for ConsultationsOperationMock {
        async fn get_consultations_by_user_account_id(
            &self,
            user_account_id: i64,
        ) -> Result<Vec<Consultation>, ErrResp> {
            if self.user_account_id != user_account_id {
                return Ok(vec![]);
            }
            Ok(self.consultations.clone())
        }
    }

    fn create_dummy_consultation1(user_account_id: i64) -> Consultation {
        Consultation {
            consultation_id: 1,
            user_account_id,
            consultant_id: 510,
            meeting_at: "2023-04-13T14:00:00.0000+09:00 ".to_string(),
            room_name: "1241cd91a9c3433f98d16f40f51a5090".to_string(),
            user_account_entered_at: None,
            consultant_entered_at: None,
        }
    }

    fn create_dummy_consultation2(user_account_id: i64) -> Consultation {
        Consultation {
            consultation_id: 2,
            user_account_id,
            consultant_id: 6110,
            meeting_at: "2023-04-15T14:00:00.0000+09:00 ".to_string(),
            room_name: "3241cd91a9c3433f98d16f40f51a5090".to_string(),
            user_account_entered_at: Some("2023-04-15T13:58:32.2424+09:00 ".to_string()),
            consultant_entered_at: Some("2023-04-15T13:57:42.3435+09:00 ".to_string()),
        }
    }

    #[tokio::test]

    async fn get_consultations_by_user_account_id_internal_success_1_result() {
        let user_account_id = 64431;
        let consultation1 = create_dummy_consultation1(user_account_id);
        let op_mock = ConsultationsOperationMock {
            user_account_id,
            consultations: vec![consultation1.clone()],
        };

        let result = get_consultations_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(vec![consultation1], resp.1 .0.consultations);
    }

    #[tokio::test]

    async fn get_consultations_by_user_account_id_internal_success_2_results() {
        let user_account_id = 64431;
        let consultation1 = create_dummy_consultation1(user_account_id);
        let consultation2 = create_dummy_consultation2(user_account_id);
        let op_mock = ConsultationsOperationMock {
            user_account_id,
            consultations: vec![consultation1.clone(), consultation2.clone()],
        };

        let result = get_consultations_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(vec![consultation1, consultation2], resp.1 .0.consultations);
    }

    #[tokio::test]

    async fn get_consultations_by_user_account_id_internal_success_no_result() {
        let user_account_id = 64431;
        let op_mock = ConsultationsOperationMock {
            user_account_id,
            consultations: vec![],
        };

        let result = get_consultations_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            Vec::<Consultation>::with_capacity(0),
            resp.1 .0.consultations
        );
    }

    #[tokio::test]
    async fn get_consultations_by_user_account_id_internal_fail_user_account_id_is_zero() {
        let user_account_id = 0;
        let op_mock = ConsultationsOperationMock {
            user_account_id,
            consultations: vec![],
        };

        let result = get_consultations_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }

    #[tokio::test]
    async fn get_consultations_by_user_account_id_internal_fail_user_account_id_is_negative() {
        let user_account_id = -1;
        let op_mock = ConsultationsOperationMock {
            user_account_id,
            consultations: vec![],
        };

        let result = get_consultations_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(resp.0, StatusCode::BAD_REQUEST);
        assert_eq!(resp.1 .0.code, Code::AccountIdIsNotPositive as u32)
    }
}
