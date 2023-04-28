// Copyright 2022 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::Datelike;
use common::util::Ymd;
use common::{util::Career, RespResult};
use common::{ApiError, ErrResp};
use entity::career;
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::authentication::authenticated_handlers::authenticated_users::user::User;

pub(crate) async fn career(
    User { user_info }: User,
    param: Query<GetCareerQueryParam>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<Career> {
    let param = param.0;
    let op = GetCareerOperationImpl::new(pool);
    handle_career_req(user_info.account_id, param.career_id, op).await
}

#[derive(Deserialize)]
pub(crate) struct GetCareerQueryParam {
    pub(crate) career_id: i64,
}

async fn handle_career_req(
    account_id: i64,
    career_id: i64,
    op: impl GetCareerOperation,
) -> RespResult<Career> {
    // 任意の職務経歴の取得を防ぐため、必ずログインユーザーのアカウントIDでフィルターをかけた結果を利用
    let careers = op.filter_careers_by_account_id(account_id).await?;
    for career in careers {
        if career.0 == career_id {
            return Ok((StatusCode::OK, Json(career.1)));
        }
    }
    error!(
        "No career associated with user account found (account_id: {}, career_id: {})",
        account_id, career_id
    );
    Err((
        StatusCode::BAD_REQUEST,
        Json(ApiError {
            code: Code::NoCareerToHandleFound as u32,
        }),
    ))
}

#[async_trait]
trait GetCareerOperation {
    async fn filter_careers_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Vec<(i64, Career)>, ErrResp>;
}

struct GetCareerOperationImpl {
    pool: DatabaseConnection,
}

impl GetCareerOperationImpl {
    fn new(pool: DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GetCareerOperation for GetCareerOperationImpl {
    async fn filter_careers_by_account_id(
        &self,
        account_id: i64,
    ) -> Result<Vec<(i64, Career)>, ErrResp> {
        let models = career::Entity::find()
            .filter(career::Column::UserAccountId.eq(account_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter career (user_account_id: {}): {}",
                    account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| {
                (
                    m.career_id,
                    Career {
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
                )
            })
            .collect::<Vec<(i64, Career)>>())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    struct GetCareerOperationMock {
        account_id: i64,
        careers: Vec<(i64, Career)>,
    }

    #[async_trait]
    impl GetCareerOperation for GetCareerOperationMock {
        async fn filter_careers_by_account_id(
            &self,
            account_id: i64,
        ) -> Result<Vec<(i64, Career)>, ErrResp> {
            assert_eq!(self.account_id, account_id);
            Ok(self.careers.clone())
        }
    }

    #[tokio::test]
    async fn handle_career_req_success() {
        let account_id = 45;
        let career1_id = 642;
        let career1 = create_dummy_career1();
        let career2_id = 511;
        let career2 = create_dummy_career2();
        let career3_id = 5552;
        let career3 = create_dummy_career3();
        let op = GetCareerOperationMock {
            account_id,
            careers: vec![
                (career1_id, career1.clone()),
                (career2_id, career2),
                (career3_id, career3),
            ],
        };

        let result = handle_career_req(account_id, career1_id, op).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(career1, resp.1 .0);
    }

    fn create_dummy_career1() -> Career {
        Career {
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
        }
    }

    fn create_dummy_career2() -> Career {
        Career {
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
        }
    }

    fn create_dummy_career3() -> Career {
        Career {
            company_name: "テスト３株式会社".to_string(),
            department_name: None,
            office: None,
            career_start_date: Ymd {
                year: 2006,
                month: 4,
                day: 1,
            },
            career_end_date: None,
            contract_type: "other".to_string(),
            profession: Some("企画".to_string()),
            annual_income_in_man_yen: Some(500),
            is_manager: false,
            position_name: None,
            is_new_graduate: false,
            note: Some("備考３".to_string()),
        }
    }

    #[tokio::test]
    async fn handle_career_req_fail_no_career_to_handle_found() {
        let account_id = 45;
        let career1_id = 642;
        let career1 = create_dummy_career1();
        let career2_id = 511;
        let career2 = create_dummy_career2();
        let career3_id = 5552;
        let career3 = create_dummy_career3();
        let op = GetCareerOperationMock {
            account_id,
            careers: vec![
                (career1_id, career1.clone()),
                (career2_id, career2),
                (career3_id, career3),
            ],
        };
        let dummy_career_id = 56015;
        assert_ne!(dummy_career_id, career1_id);
        assert_ne!(dummy_career_id, career2_id);
        assert_ne!(dummy_career_id, career3_id);

        let result = handle_career_req(account_id, dummy_career_id, op).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::NoCareerToHandleFound as u32, resp.1 .0.code);
    }
}
