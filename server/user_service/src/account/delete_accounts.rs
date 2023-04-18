// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::extract::State;
use chrono::{DateTime, FixedOffset};
use common::{RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::DatabaseConnection;
use opensearch::OpenSearch;
use serde::Serialize;

use crate::util::session::user::User;

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
trait DeleteAccountsOperation {}

struct DeleteAccountsOperationImpl {
    pool: DatabaseConnection,
    index_client: OpenSearch,
}

#[async_trait]
impl DeleteAccountsOperation for DeleteAccountsOperationImpl {}

async fn handle_delete_accounts(
    account_id: i64,
    email_address: String,
    current_date_time: DateTime<FixedOffset>,
    op: &impl DeleteAccountsOperation,
) -> RespResult<DeleteAccountsResult> {
    // settlement一覧取得
    // settlement一覧をイテレート
    //   settlementの排他ロックを取得し、stopped_settlementへ移動
    // user_accountの排他ロック
    //   opensearchから職歴の削除 (documentのデータは後の定期処理で削除する)
    //   user_accountの削除（user_accountからdeleted_user_accountへ移動）
    // 削除成功のメール通知
    todo!()
}
