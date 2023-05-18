// Copyright 2023 Ken Miura

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::{async_trait, Json};
use common::{ErrResp, RespResult};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Serialize;
use tracing::error;

use crate::err::unexpected_err_resp;

use super::super::admin::Admin;
use super::UserAccountIdQuery;

pub(crate) async fn get_agreements_by_user_account_id(
    Admin { admin_info: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<UserAccountIdQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<AgreementsResult> {
    let query = query.0;
    let op = AgreementsOperationImpl { pool };
    get_agreements_by_user_account_id_internal(query.user_account_id, op).await
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub(crate) struct AgreementsResult {
    agreements: Vec<Agreement>,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
struct Agreement {
    email_address: String,
    version: i32,
    agreed_at: String, // RFC 3339形式の文字列
}

async fn get_agreements_by_user_account_id_internal(
    user_account_id: i64,
    op: impl AgreementsOperation,
) -> RespResult<AgreementsResult> {
    let agreements = op
        .get_agreements_by_user_account_id(user_account_id)
        .await?;
    Ok((StatusCode::OK, Json(AgreementsResult { agreements })))
}

#[async_trait]
trait AgreementsOperation {
    async fn get_agreements_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<Agreement>, ErrResp>;
}

struct AgreementsOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl AgreementsOperation for AgreementsOperationImpl {
    async fn get_agreements_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<Vec<Agreement>, ErrResp> {
        let models = entity::terms_of_use::Entity::find()
            .filter(entity::terms_of_use::Column::UserAccountId.eq(user_account_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter terms_of_use (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| Agreement {
                email_address: m.email_address,
                version: m.ver,
                agreed_at: m.agreed_at.to_rfc3339(),
            })
            .collect::<Vec<Agreement>>())
    }
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use axum::http::StatusCode;
    use common::ErrResp;

    use super::*;

    struct AgreementsOperationMock {
        user_account_id: i64,
        agreements: Vec<Agreement>,
    }

    #[async_trait]
    impl AgreementsOperation for AgreementsOperationMock {
        async fn get_agreements_by_user_account_id(
            &self,
            user_account_id: i64,
        ) -> Result<Vec<Agreement>, ErrResp> {
            if self.user_account_id != user_account_id {
                return Ok(vec![]);
            }
            Ok(self.agreements.clone())
        }
    }

    fn create_dummy_agreement1() -> Agreement {
        Agreement {
            email_address: "test@test.com".to_string(),
            version: 1,
            agreed_at: "2023-04-13T14:12:53.4242+09:00 ".to_string(),
        }
    }

    fn create_dummy_agreement2() -> Agreement {
        Agreement {
            email_address: "test@test.com".to_string(),
            version: 2,
            agreed_at: "2023-05-01T18:27:41.1221+09:00 ".to_string(),
        }
    }

    #[tokio::test]

    async fn get_agreements_by_user_account_id_internal_success_1_result() {
        let user_account_id = 64431;
        let agreement1 = create_dummy_agreement1();
        let op_mock = AgreementsOperationMock {
            user_account_id,
            agreements: vec![agreement1.clone()],
        };

        let result = get_agreements_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(vec![agreement1], resp.1 .0.agreements);
    }

    #[tokio::test]

    async fn get_agreements_by_user_account_id_internal_success_2_results() {
        let user_account_id = 64431;
        let agreement1 = create_dummy_agreement1();
        let agreement2 = create_dummy_agreement2();
        let op_mock = AgreementsOperationMock {
            user_account_id,
            agreements: vec![agreement1.clone(), agreement2.clone()],
        };

        let result = get_agreements_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(vec![agreement1, agreement2], resp.1 .0.agreements);
    }

    #[tokio::test]

    async fn get_agreements_by_user_account_id_internal_success_no_result() {
        let user_account_id = 64431;
        let op_mock = AgreementsOperationMock {
            user_account_id,
            agreements: vec![],
        };

        let result = get_agreements_by_user_account_id_internal(user_account_id, op_mock).await;

        let resp = result.expect("failed to get Ok");
        assert_eq!(StatusCode::OK, resp.0);
        assert_eq!(Vec::<Agreement>::with_capacity(0), resp.1 .0.agreements);
    }
}
