// Copyright 2023 Ken Miura

use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use chrono::{DateTime, FixedOffset};
use common::{ErrResp, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use opensearch::OpenSearch;
use serde::Serialize;
use tracing::error;

use crate::{err::unexpected_err_resp, util::session::user::User};

pub(crate) async fn delete_accounts(
    User { user_info }: User,
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
) -> RespResult<DeleteAccountsResult> {
    let current_date_time = chrono::Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = DeleteAccountsOperationImpl { pool, index_client };
    handle_delete_accounts(
        user_info.account_id,
        user_info.email_address,
        current_date_time,
        &op,
    )
    .await
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct DeleteAccountsResult {}

#[async_trait]
trait DeleteAccountsOperation {
    async fn get_settlement_ids(&self, consultant_id: i64) -> Result<Vec<i64>, ErrResp>;
}

struct DeleteAccountsOperationImpl {
    pool: DatabaseConnection,
    index_client: OpenSearch,
}

#[async_trait]
impl DeleteAccountsOperation for DeleteAccountsOperationImpl {
    async fn get_settlement_ids(&self, consultant_id: i64) -> Result<Vec<i64>, ErrResp> {
        // FROM settlement LEFT JOIN consultationの形となる
        // consultationの数は多量になる可能性がある。一方で、settlementの数はたかがしれているので
        // settlementを起点にデータを取ってくるこのケースでは複数回分けて取得するような操作は必要ない
        let models = entity::settlement::Entity::find()
            .find_also_related(entity::consultation::Entity)
            .filter(entity::consultation::Column::ConsultantId.eq(consultant_id))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter settlement (consultant_id: {}): {}",
                    consultant_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(models
            .into_iter()
            .map(|m| m.0.settlement_id)
            .collect::<Vec<i64>>())
    }
}

async fn handle_delete_accounts(
    account_id: i64,
    email_address: String,
    current_date_time: DateTime<FixedOffset>,
    op: &impl DeleteAccountsOperation,
) -> RespResult<DeleteAccountsResult> {
    let ids = op.get_settlement_ids(account_id).await?;
    println!("{:?}", ids);
    // settlement一覧取得
    // settlement一覧をイテレート
    //   settlementの排他ロックを取得し、stopped_settlementへ移動
    // user_accountの排他ロック
    //   opensearchから職歴の削除 (documentのデータは後の定期処理で削除する)
    //   user_accountの削除（user_accountからdeleted_user_accountへ移動）
    // 削除成功のメール通知
    Ok((StatusCode::OK, Json(DeleteAccountsResult {})))
}
