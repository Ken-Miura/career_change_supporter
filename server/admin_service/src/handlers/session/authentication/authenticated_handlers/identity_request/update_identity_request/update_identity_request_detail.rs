// Copyright 2021 Ken Miura

use axum::async_trait;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::Datelike;
use common::util::Ymd;
use common::{ApiError, ErrResp, RespResult};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use entity::update_identity_req;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::unexpected_err_resp;
use crate::err::Code::NoUpdateIdentityReqDetailFound;
use crate::handlers::session::authentication::authenticated_handlers::admin::Admin;

pub(crate) async fn get_update_identity_request_detail(
    Admin { account_id: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<UpdateIdentityReqDetailQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<UpdateIdentityReqDetail> {
    let query = query.0;
    let op = UpdateIdentityReqDetailOperationImpl { pool };
    get_update_identity_req_detail(query.user_account_id, op).await
}

#[derive(Deserialize)]
pub(crate) struct UpdateIdentityReqDetailQuery {
    pub(crate) user_account_id: i64,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct UpdateIdentityReqDetail {
    pub(crate) last_name: String,
    pub(crate) first_name: String,
    pub(crate) last_name_furigana: String,
    pub(crate) first_name_furigana: String,
    pub(crate) date_of_birth: Ymd,
    pub(crate) prefecture: String,
    pub(crate) city: String,
    pub(crate) address_line1: String,
    pub(crate) address_line2: Option<String>,
    pub(crate) telephone_number: String,
    pub(crate) image1_file_name_without_ext: String,
    pub(crate) image2_file_name_without_ext: Option<String>,
}

async fn get_update_identity_req_detail(
    user_account_id: i64,
    op: impl UpdateIdentityReqDetailOperation,
) -> RespResult<UpdateIdentityReqDetail> {
    let req_detail_option = op.get_update_identity_req_detail(user_account_id).await?;
    let req_detail = req_detail_option.ok_or_else(|| {
        error!(
            "no update identity request (user account id: {}) found",
            user_account_id
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: NoUpdateIdentityReqDetailFound as u32,
            }),
        )
    })?;
    Ok((StatusCode::OK, Json(req_detail)))
}

#[async_trait]
trait UpdateIdentityReqDetailOperation {
    async fn get_update_identity_req_detail(
        &self,
        user_account_id: i64,
    ) -> Result<Option<UpdateIdentityReqDetail>, ErrResp>;
}

struct UpdateIdentityReqDetailOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl UpdateIdentityReqDetailOperation for UpdateIdentityReqDetailOperationImpl {
    async fn get_update_identity_req_detail(
        &self,
        user_account_id: i64,
    ) -> Result<Option<UpdateIdentityReqDetail>, ErrResp> {
        let result = update_identity_req::Entity::find_by_id(user_account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find update_identity_req (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(result.map(|m| UpdateIdentityReqDetail {
            last_name: m.last_name,
            first_name: m.first_name,
            last_name_furigana: m.last_name_furigana,
            first_name_furigana: m.first_name_furigana,
            date_of_birth: Ymd {
                year: m.date_of_birth.year(),
                month: m.date_of_birth.month(),
                day: m.date_of_birth.day(),
            },
            prefecture: m.prefecture,
            city: m.city,
            address_line1: m.address_line1,
            address_line2: m.address_line2,
            telephone_number: m.telephone_number,
            image1_file_name_without_ext: m.image1_file_name_without_ext,
            image2_file_name_without_ext: m.image2_file_name_without_ext,
        }))
    }
}

#[cfg(test)]
mod tests {
    use crate::err::Code::NoUpdateIdentityReqDetailFound;
    use async_session::async_trait;
    use axum::http::StatusCode;
    use common::{util::Ymd, ErrResp};

    use super::{
        get_update_identity_req_detail, UpdateIdentityReqDetail, UpdateIdentityReqDetailOperation,
    };

    struct UpdateIdentityReqDetailOperationMock {
        user_account_id: i64,
        update_identity_req_detail: UpdateIdentityReqDetail,
    }

    #[async_trait]
    impl UpdateIdentityReqDetailOperation for UpdateIdentityReqDetailOperationMock {
        async fn get_update_identity_req_detail(
            &self,
            user_account_id: i64,
        ) -> Result<Option<UpdateIdentityReqDetail>, ErrResp> {
            if self.user_account_id != user_account_id {
                return Ok(None);
            }
            Ok(Some(self.update_identity_req_detail.clone()))
        }
    }

    #[tokio::test]

    async fn get_update_identity_req_detail_success() {
        let user_account_id = 5135;
        let date_of_birth = Ymd {
            year: 1991,
            month: 4,
            day: 1,
        };
        let update_identity_req_detail = UpdateIdentityReqDetail {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth,
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("08012345678"),
            image1_file_name_without_ext: String::from("bcc72c586be3b2a70d6652ff74c6a484"),
            image2_file_name_without_ext: None,
        };
        let op_mock = UpdateIdentityReqDetailOperationMock {
            user_account_id,
            update_identity_req_detail: update_identity_req_detail.clone(),
        };

        let result = get_update_identity_req_detail(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(update_identity_req_detail, resp.1 .0);
    }

    #[tokio::test]

    async fn get_update_identity_req_detail_fail_no_req_detail_found() {
        let user_account_id = 5135;
        let date_of_birth = Ymd {
            year: 1991,
            month: 4,
            day: 1,
        };
        let update_identity_req_detail = UpdateIdentityReqDetail {
            last_name: String::from("山田"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("ヤマダ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth,
            prefecture: String::from("東京都"),
            city: String::from("町田市"),
            address_line1: String::from("森の里２−２２−２"),
            address_line2: None,
            telephone_number: String::from("08012345678"),
            image1_file_name_without_ext: String::from("bcc72c586be3b2a70d6652ff74c6a484"),
            image2_file_name_without_ext: None,
        };
        let op_mock = UpdateIdentityReqDetailOperationMock {
            user_account_id: user_account_id + 6230,
            update_identity_req_detail: update_identity_req_detail.clone(),
        };

        let result = get_update_identity_req_detail(user_account_id, op_mock).await;

        let err_resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, err_resp.0);
        assert_eq!(NoUpdateIdentityReqDetailFound as u32, err_resp.1 .0.code);
    }
}
