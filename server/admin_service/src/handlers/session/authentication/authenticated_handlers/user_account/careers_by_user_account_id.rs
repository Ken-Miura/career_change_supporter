// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::Datelike;
use common::util::{Career, Ymd};
use common::{ErrResp, RespResult};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;

use super::super::admin::Admin;
use super::UserAccountIdQuery;

pub(crate) async fn get_careers_by_user_account_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<UserAccountIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<CareersResult> {
    let query = query.0;
    let op = CareersOperationImpl { pool };
    get_careers_by_user_account_id_internal(query.user_account_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct CareersResult {
    careers: Vec<CareerWithId>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct CareerWithId {
    career_id: i64,
    career: Career,
}

async fn get_careers_by_user_account_id_internal(
    user_account_id: i64,
    op: impl CareersOperation,
) -> RespResult<CareersResult> {
    let careers = op.get_careers_by_user_account_id(user_account_id).await?;
    Ok((StatusCode::OK, Json(CareersResult { careers })))
}

#[async_trait]
trait CareersOperation {
    async fn get_careers_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<CareerWithId>, ErrResp>;
}

struct CareersOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl CareersOperation for CareersOperationImpl {
    async fn get_careers_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<CareerWithId>, ErrResp> {
        let models = entity::career::Entity::find()
            .filter(entity::career::Column::UserAccountId.eq(user_account_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter careers (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| CareerWithId {
                career_id: m.career_id,
                career: Career {
                    company_name: m.company_name,
                    department_name: m.department_name,
                    office: m.office,
                    career_start_date: Ymd {
                        year: m.career_start_date.year(),
                        month: m.career_start_date.month(),
                        day: m.career_start_date.day(),
                    },
                    career_end_date: m.career_end_date.map(|date| Ymd {
                        year: date.year(),
                        month: date.month(),
                        day: date.day(),
                    }),
                    contract_type: m.contract_type,
                    profession: m.profession,
                    annual_income_in_man_yen: m.annual_income_in_man_yen,
                    is_manager: m.is_manager,
                    position_name: m.position_name,
                    is_new_graduate: m.is_new_graduate,
                    note: m.note,
                },
            })
            .collect::<Vec<CareerWithId>>())
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::{util::Ymd, ErrResp};

    use super::*;

    struct CareersOperationMock {
        user_account_id: i64,
        careers: Vec<CareerWithId>,
    }

    #[async_trait]
    impl CareersOperation for CareersOperationMock {
        async fn get_careers_by_user_account_id(
            &self,
            user_account_id: i64,
        ) -> Result<Vec<CareerWithId>, ErrResp> {
            if self.user_account_id != user_account_id {
                return Ok(vec![]);
            }
            Ok(self.careers.clone())
        }
    }

    fn create_dummy_career_with_id1() -> CareerWithId {
        CareerWithId {
            career_id: 1,
            career: Career {
                company_name: "テスト１株式会社".to_string(),
                department_name: Some("営業二課".to_string()),
                office: Some("新宿事業所".to_string()),
                career_start_date: Ymd {
                    year: 2008,
                    month: 4,
                    day: 1,
                },
                career_end_date: Some(Ymd {
                    year: 2016,
                    month: 7,
                    day: 31,
                }),
                contract_type: "regular".to_string(),
                profession: Some("営業".to_string()),
                annual_income_in_man_yen: Some(500),
                is_manager: true,
                position_name: Some("部長".to_string()),
                is_new_graduate: true,
                note: Some("備考１".to_string()),
            },
        }
    }

    fn create_dummy_career_with_id2() -> CareerWithId {
        CareerWithId {
            career_id: 2,
            career: Career {
                company_name: "テスト２株式会社".to_string(),
                department_name: Some("開発".to_string()),
                office: Some("札幌事業所".to_string()),
                career_start_date: Ymd {
                    year: 2008,
                    month: 4,
                    day: 1,
                },
                career_end_date: None,
                contract_type: "contract".to_string(),
                profession: Some("エンジニア".to_string()),
                annual_income_in_man_yen: Some(500),
                is_manager: false,
                position_name: None,
                is_new_graduate: false,
                note: Some("備考２".to_string()),
            },
        }
    }

    #[tokio::test]

    async fn get_careers_by_user_account_id_internal_success_1_result() {
        let user_account_id = 64431;
        let career1 = create_dummy_career_with_id1();
        let op_mock = CareersOperationMock {
            user_account_id,
            careers: vec![career1.clone()],
        };

        let result = get_careers_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(vec![career1], resp.1 .0.careers);
    }

    #[tokio::test]

    async fn get_careers_by_user_account_id_internal_success_2_results() {
        let user_account_id = 64431;
        let career1 = create_dummy_career_with_id1();
        let career2 = create_dummy_career_with_id2();
        let op_mock = CareersOperationMock {
            user_account_id,
            careers: vec![career1.clone(), career2.clone()],
        };

        let result = get_careers_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(vec![career1, career2], resp.1 .0.careers);
    }

    #[tokio::test]

    async fn get_careers_by_user_account_id_internal_success_no_result() {
        let user_account_id = 64431;
        let op_mock = CareersOperationMock {
            user_account_id,
            careers: vec![],
        };

        let result = get_careers_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Vec::<CareerWithId>::with_capacity(0), resp.1 .0.careers);
    }
}
