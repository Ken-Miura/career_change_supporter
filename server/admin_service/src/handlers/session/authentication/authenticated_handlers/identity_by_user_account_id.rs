// Copyright 2022 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::Datelike;
use common::util::{Identity, Ymd};
use common::{ApiError, ErrResp, RespResult};
use entity::identity;
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use serde::Deserialize;
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

use super::admin::Admin;

pub(crate) async fn get_identity_by_user_account_id(
    Admin { account_id: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<GetIdentityQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<Identity> {
    let query = query.0;
    let op = IdentityOperationImpl { pool };
    get_identity_by_user_account_id_internal(query.user_account_id, op).await
}

#[derive(Deserialize)]
pub(crate) struct GetIdentityQuery {
    user_account_id: i64,
}

async fn get_identity_by_user_account_id_internal(
    user_account_id: i64,
    op: impl IdentityOperation,
) -> RespResult<Identity> {
    let identity_option = op.get_identity_by_user_account_id(user_account_id).await?;
    let identity = identity_option.ok_or_else(|| {
        error!("no identity (user account id: {}) found", user_account_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoIdentityFound as u32,
            }),
        )
    })?;
    Ok((StatusCode::OK, Json(identity)))
}

#[async_trait]
trait IdentityOperation {
    async fn get_identity_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<Identity>, ErrResp>;
}

struct IdentityOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl IdentityOperation for IdentityOperationImpl {
    async fn get_identity_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<Identity>, ErrResp> {
        let model = entity::prelude::Identity::find_by_id(user_account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find identity (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(IdentityOperationImpl::convert_identity_model_to_identity))
    }
}

impl IdentityOperationImpl {
    fn convert_identity_model_to_identity(identity_model: identity::Model) -> Identity {
        let date = identity_model.date_of_birth;
        let ymd = Ymd {
            year: date.year(),
            month: date.month(),
            day: date.day(),
        };
        Identity {
            last_name: identity_model.last_name,
            first_name: identity_model.first_name,
            last_name_furigana: identity_model.last_name_furigana,
            first_name_furigana: identity_model.first_name_furigana,
            date_of_birth: ymd,
            prefecture: identity_model.prefecture,
            city: identity_model.city,
            address_line1: identity_model.address_line1,
            address_line2: identity_model.address_line2,
            telephone_number: identity_model.telephone_number,
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::{
        util::{Identity, Ymd},
        ErrResp,
    };

    use crate::err::Code;

    use super::*;

    struct IdentityOperationMock {
        user_account_id: i64,
        identity: Identity,
    }

    #[async_trait]
    impl IdentityOperation for IdentityOperationMock {
        async fn get_identity_by_user_account_id(
            &self,
            user_account_id: i64,
        ) -> Result<Option<Identity>, ErrResp> {
            if self.user_account_id != user_account_id {
                return Ok(None);
            }
            Ok(Some(self.identity.clone()))
        }
    }

    #[tokio::test]

    async fn get_identity_by_user_account_id_internal_success() {
        let user_account_id = 64431;
        let identity = Identity {
            last_name: String::from("田中"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("タナカ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: 1978,
                month: 11,
                day: 23,
            },
            prefecture: String::from("和歌山県"),
            city: String::from("和歌山市"),
            address_line1: String::from("小松原通１−１"),
            address_line2: None,
            telephone_number: String::from("08043218765"),
        };
        let op_mock = IdentityOperationMock {
            user_account_id,
            identity: identity.clone(),
        };

        let result = get_identity_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(identity, resp.1 .0);
    }

    #[tokio::test]

    async fn get_identity_by_user_account_id_internal_fail_no_identity_found() {
        let user_account_id = 64431;
        let identity = Identity {
            last_name: String::from("田中"),
            first_name: String::from("太郎"),
            last_name_furigana: String::from("タナカ"),
            first_name_furigana: String::from("タロウ"),
            date_of_birth: Ymd {
                year: 1978,
                month: 11,
                day: 23,
            },
            prefecture: String::from("和歌山県"),
            city: String::from("和歌山市"),
            address_line1: String::from("小松原通１−１"),
            address_line2: None,
            telephone_number: String::from("08043218765"),
        };
        let op_mock = IdentityOperationMock {
            user_account_id,
            identity: identity.clone(),
        };
        let dummy_id = user_account_id + 451;

        let result = get_identity_by_user_account_id_internal(dummy_id, op_mock).await;

        let resp = result.expect_err("failed to get Err");
        assert_eq!(StatusCode::BAD_REQUEST, resp.0);
        assert_eq!(Code::NoIdentityFound as u32, resp.1 .0.code);
    }
}
