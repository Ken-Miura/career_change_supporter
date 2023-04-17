// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::extract::State;
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use opensearch::OpenSearch;
use serde::Serialize;

use crate::util::session::user::User;

pub(crate) async fn delete_accounts(
    User {
        user_info: _user_info,
    }: User,
    State(_pool): State<DatabaseConnection>,
    State(_index_client): State<OpenSearch>,
) -> RespResult<DeleteAccountsResult> {
    todo!()
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
    op: &impl DeleteAccountsOperation,
) -> RespResult<DeleteAccountsResult> {
    // user_accountの排他ロック
    //   settlementに何かあればstopped_settlementへ移動
    //   opensearchから職歴の削除
    //   user_accountの削除（user_accountからdeleted_user_accountへ移動）
    // 削除成功のメール通知
    todo!()
}
