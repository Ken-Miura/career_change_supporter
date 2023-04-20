// Copyright 2022 Ken Miura

use common::ErrRespStruct;
use entity::{
    document,
    sea_orm::{ActiveModelTrait, DatabaseTransaction, EntityTrait, QuerySelect, Set},
};
use tracing::error;

use crate::err::unexpected_err_resp;

/// 共有ロックを行い、documentテーブルからドキュメントIDを取得する
///
/// opensearch呼び出しとセットで利用するため、トランザクション内で利用することが前提となる
pub(crate) async fn find_document_model_by_user_account_id_with_shared_lock(
    txn: &DatabaseTransaction,
    user_account_id: i64,
) -> Result<Option<document::Model>, ErrRespStruct> {
    let doc_option = document::Entity::find_by_id(user_account_id)
        .lock_shared()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find document (user_account_id: {}): {}",
                user_account_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(doc_option)
}

/// 排他ロックを行い、documentテーブルからドキュメントIDを取得する
///
/// opensearch呼び出しとセットで利用するため、トランザクション内で利用することが前提となる
pub(crate) async fn find_document_model_by_user_account_id_with_exclusive_lock(
    txn: &DatabaseTransaction,
    user_account_id: i64,
) -> Result<Option<document::Model>, ErrRespStruct> {
    let doc_option = document::Entity::find_by_id(user_account_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find document (user_account_id: {}): {}",
                user_account_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(doc_option)
}

/// documentテーブルにドキュメントIDを挿入する
///
/// opensearchにドキュメントを登録する際、そのドキュメントIDをDBに保管しておくために利用する<br>
/// opensearch呼び出しとセットで利用するため、トランザクション内で利用することが前提となる
pub(crate) async fn insert_document(
    txn: &DatabaseTransaction,
    user_account_id: i64,
    document_id: i64,
) -> Result<(), ErrRespStruct> {
    let document = document::ActiveModel {
        user_account_id: Set(user_account_id),
        document_id: Set(document_id),
    };
    let _ = document.insert(txn).await.map_err(|e| {
        error!(
            "failed to insert document (user_account_id: {}, document_id: {}): {}",
            user_account_id, document_id, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}
