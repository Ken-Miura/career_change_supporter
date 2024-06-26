// Copyright 2022 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::util::Identity;
use common::{ApiError, ErrResp, RespResult};
use entity::sea_orm::DatabaseConnection;
use serde::Deserialize;
use tracing::error;

use crate::err::Code;

use super::admin::Admin;
use super::find_identity_by_user_account_id;

pub(crate) async fn get_identity_by_user_account_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
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
        find_identity_by_user_account_id(&self.pool, user_account_id).await
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
