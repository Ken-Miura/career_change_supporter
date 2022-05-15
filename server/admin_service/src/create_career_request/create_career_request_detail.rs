// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    Json,
};
use chrono::Datelike;
use common::util::Ymd;
use common::{ApiError, ErrResp, RespResult};
use entity::create_career_req;
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::unexpected_err_resp;
use crate::err::Code::NoCreateCareerReqDetailFound;
use crate::util::session::Admin;

pub(crate) async fn get_create_career_request_detail(
    Admin { account_id: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<CreateCareerReqDetailQuery>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<CreateCareerReqDetail> {
    let query = query.0;
    let op = CreateCareerReqDetailOperationImpl { pool };
    get_create_career_req_detail(query.create_career_req_id, op).await
}

#[derive(Deserialize)]
pub(crate) struct CreateCareerReqDetailQuery {
    pub(crate) create_career_req_id: i64,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct CreateCareerReqDetail {
    pub(crate) user_account_id: i64,
    pub(crate) company_name: String,
    pub(crate) department_name: Option<String>,
    pub(crate) office: Option<String>,
    pub(crate) career_start_date: Ymd,
    pub(crate) career_end_date: Option<Ymd>,
    pub(crate) contract_type: String,
    pub(crate) profession: Option<String>,
    pub(crate) annual_income_in_man_yen: Option<i32>,
    pub(crate) is_manager: bool,
    pub(crate) position_name: Option<String>,
    pub(crate) is_new_graduate: bool,
    pub(crate) note: Option<String>,
    pub(crate) image1_file_name_without_ext: String,
    pub(crate) image2_file_name_without_ext: Option<String>,
}

async fn get_create_career_req_detail(
    create_career_req_id: i64,
    op: impl CreateCareerReqDetailOperation,
) -> RespResult<CreateCareerReqDetail> {
    let req_detail_option = op
        .get_create_career_req_detail(create_career_req_id)
        .await?;
    let req_detail = req_detail_option.ok_or_else(|| {
        error!(
            "no create career request (create career request id: {}) found",
            create_career_req_id
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: NoCreateCareerReqDetailFound as u32,
            }),
        )
    })?;
    Ok((StatusCode::OK, Json(req_detail)))
}

#[async_trait]
trait CreateCareerReqDetailOperation {
    async fn get_create_career_req_detail(
        &self,
        create_career_req_id: i64,
    ) -> Result<Option<CreateCareerReqDetail>, ErrResp>;
}

struct CreateCareerReqDetailOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl CreateCareerReqDetailOperation for CreateCareerReqDetailOperationImpl {
    async fn get_create_career_req_detail(
        &self,
        create_career_req_id: i64,
    ) -> Result<Option<CreateCareerReqDetail>, ErrResp> {
        let result = create_career_req::Entity::find_by_id(create_career_req_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find create_career_req (create_career_req_id: {}): {}",
                    create_career_req_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(result.map(|m| CreateCareerReqDetail {
            user_account_id: m.user_account_id,
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
            image1_file_name_without_ext: m.image1_file_name_without_ext,
            image2_file_name_without_ext: m.image2_file_name_without_ext,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::err::Code::NoCreateCareerReqDetailFound;
    use async_session::async_trait;
    use axum::http::StatusCode;
    use common::{util::Ymd, ErrResp};

    use super::{
        get_create_career_req_detail, CreateCareerReqDetail, CreateCareerReqDetailOperation,
    };

    struct CreateCareerReqDetailOperationMock {
        create_career_req_id: i64,
        create_career_req_detail: CreateCareerReqDetail,
    }

    #[async_trait]
    impl CreateCareerReqDetailOperation for CreateCareerReqDetailOperationMock {
        async fn get_create_career_req_detail(
            &self,
            create_career_req_id: i64,
        ) -> Result<Option<CreateCareerReqDetail>, ErrResp> {
            if self.create_career_req_id != create_career_req_id {
                return Ok(None);
            }
            Ok(Some(self.create_career_req_detail.clone()))
        }
    }

    #[tokio::test]

    async fn get_create_career_req_detail_success() {
        let create_career_req_id = 5135;
        let career_start_date = Ymd {
            year: 1991,
            month: 4,
            day: 1,
        };
        let create_career_req_detail = CreateCareerReqDetail {
            user_account_id: 123,
            company_name: "テスト株式会社１".to_string(),
            department_name: None,
            office: None,
            career_start_date,
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
            image1_file_name_without_ext: String::from("bcc72c586be3b2a70d6652ff74c6a484"),
            image2_file_name_without_ext: None,
        };
        let op_mock = CreateCareerReqDetailOperationMock {
            create_career_req_id,
            create_career_req_detail: create_career_req_detail.clone(),
        };

        let result = get_create_career_req_detail(create_career_req_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(create_career_req_detail, resp.1 .0);
    }

    #[tokio::test]

    async fn get_create_career_req_detail_fail_no_req_detail_found() {
        let create_career_req_id = 5135;
        let career_start_date = Ymd {
            year: 1991,
            month: 4,
            day: 1,
        };
        let create_career_req_detail = CreateCareerReqDetail {
            user_account_id: 123,
            company_name: "テスト株式会社１".to_string(),
            department_name: None,
            office: None,
            career_start_date,
            career_end_date: None,
            contract_type: "regular".to_string(),
            profession: None,
            annual_income_in_man_yen: None,
            is_manager: false,
            position_name: None,
            is_new_graduate: true,
            note: None,
            image1_file_name_without_ext: String::from("bcc72c586be3b2a70d6652ff74c6a484"),
            image2_file_name_without_ext: None,
        };
        let op_mock = CreateCareerReqDetailOperationMock {
            create_career_req_id: create_career_req_id + 6230,
            create_career_req_detail: create_career_req_detail.clone(),
        };

        let result = get_create_career_req_detail(create_career_req_id, op_mock).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(NoCreateCareerReqDetailFound as u32, err_resp.1 .0.code);
    }
}
