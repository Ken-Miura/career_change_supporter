// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::util::Identity;
use common::{ErrResp, RespResult};
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

use super::super::admin::Admin;
use super::super::find_identity_by_user_account_id;
use super::UserAccountIdQuery;

pub(crate) async fn get_identity_option_by_user_account_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<UserAccountIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<IdentityResult> {
    let query = query.0;
    let op = IdentityOperationImpl { pool };
    get_identity_option_by_user_account_id_internal(query.user_account_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct IdentityResult {
    identity_option: Option<Identity>,
}

async fn get_identity_option_by_user_account_id_internal(
    user_account_id: i64,
    op: impl IdentityOperation,
) -> RespResult<IdentityResult> {
    let identity_option = op
        .get_identity_option_by_user_account_id(user_account_id)
        .await?;
    Ok((StatusCode::OK, Json(IdentityResult { identity_option })))
}

#[async_trait]
trait IdentityOperation {
    async fn get_identity_option_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Option<Identity>, ErrResp>;
}

struct IdentityOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl IdentityOperation for IdentityOperationImpl {
    async fn get_identity_option_by_user_account_id(
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

    use super::*;

    struct IdentityOperationMock {
        user_account_id: i64,
        identity: Identity,
    }

    #[async_trait]
    impl IdentityOperation for IdentityOperationMock {
        async fn get_identity_option_by_user_account_id(
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

    async fn get_identity_option_by_user_account_id_internal_success() {
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

        let result =
            get_identity_option_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(
            identity,
            resp.1 .0.identity_option.expect("failed to get identity")
        );
    }

    #[tokio::test]

    async fn get_identity_option_by_user_account_id_internal_success_no_identity_found() {
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

        let result = get_identity_option_by_user_account_id_internal(dummy_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(None, resp.1 .0.identity_option);
    }
}
