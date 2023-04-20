// Copyright 2023 Ken Miura

use async_fred_session::RedisSessionStore;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{async_trait, Json};
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::SignedCookieJar;
use chrono::{DateTime, FixedOffset};
use common::{ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
use entity::sea_orm::ActiveValue::NotSet;
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    ModelTrait, QueryFilter, QuerySelect, Set, TransactionError, TransactionTrait,
};
use opensearch::OpenSearch;
use serde::Serialize;
use tracing::{error, warn};

use crate::err::unexpected_err_resp;
use crate::util::find_user_account_by_user_account_id_with_exclusive_lock;
use crate::util::session::{
    destroy_session_if_exists, get_user_info_from_cookie, SESSION_ID_COOKIE_NAME,
};

pub(crate) async fn delete_accounts(
    jar: SignedCookieJar,
    State(store): State<RedisSessionStore>,
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
) -> Result<(StatusCode, SignedCookieJar), ErrResp> {
    let option_cookie = jar.get(SESSION_ID_COOKIE_NAME);
    let user_info = get_user_info_from_cookie(option_cookie.clone(), &store, &pool).await?;

    let current_date_time = chrono::Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = DeleteAccountsOperationImpl { pool, index_client };
    let _ = handle_delete_accounts(
        user_info.account_id,
        user_info.email_address,
        current_date_time,
        &op,
    )
    .await?;

    if let Some(session_id) = option_cookie {
        destroy_session_if_exists(session_id.value(), &store).await?;
    }
    Ok((
        StatusCode::OK,
        jar.remove(Cookie::named(SESSION_ID_COOKIE_NAME)),
    ))
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct DeleteAccountsResult {}

#[async_trait]
trait DeleteAccountsOperation {
    async fn get_settlement_ids(&self, consultant_id: i64) -> Result<Vec<i64>, ErrResp>;

    /// 役務の提供（ユーザーとの相談）が未実施の支払いに関して、コンサルタント（この削除するアカウント）が受け取るのを止める
    async fn stop_payment(
        &self,
        settlement_id: i64,
        stopped_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;

    async fn delete_user_account(
        &self,
        account_id: i64,
        deleted_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
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

    async fn stop_payment(
        &self,
        settlement_id: i64,
        stopped_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let s_option =
                        find_settlement_by_settlement_id_with_exclusive_lock(txn, settlement_id)
                            .await?;
                    let s = match s_option {
                        Some(s) => s,
                        None => {
                            // 一度リストアップしたsettlementをイテレートしている途中のため、存在しない確率は低い
                            // （確率が低いだけで正常なケースもありえる。例えば、リストアップされた後、ここに到達するまでの間にユーザーがコンサルタントの評価（決済）を行った等）
                            // 正常なケースも考えられるが、低確率のためwarnでログに記録しておく
                            warn!(
                                "no settelment (settelment_id: {}) found when stopping payment",
                                settlement_id
                            );
                            return Ok(());
                        }
                    };

                    insert_stopped_settlement(txn, s, stopped_date_time).await?;

                    let _ = entity::settlement::Entity::delete_by_id(settlement_id)
                        .exec(txn)
                        .await
                        .map_err(|e| {
                            error!(
                                "failed to delete settlement (settlement_id: {}): {}",
                                settlement_id, e
                            );
                            ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            }
                        })?;

                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(db_err) => {
                    error!("connection error: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    error!("failed to stop_payment: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }

    async fn delete_user_account(
        &self,
        account_id: i64,
        deleted_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let user_option =
                        find_user_account_by_user_account_id_with_exclusive_lock(txn, account_id)
                            .await?;
                    let user = user_option.ok_or_else(|| {
                        error!(
                            "failed to find user_account (user_account_id: {})",
                            account_id
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    insert_deleted_user_account(txn, &user, deleted_date_time).await?;

                    let _ = user.delete(txn).await.map_err(|e| {
                        error!(
                            "failed to delete user_account (user_account_id: {}): {}",
                            account_id, e
                        );
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                    //   opensearchから職歴の削除 (documentのデータは後の定期処理で削除する)
                    Ok(())
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(db_err) => {
                    error!("connection error: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    error!("failed to delete_user_account: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

async fn find_settlement_by_settlement_id_with_exclusive_lock(
    txn: &DatabaseTransaction,
    settlement_id: i64,
) -> Result<Option<entity::settlement::Model>, ErrRespStruct> {
    let model = entity::settlement::Entity::find_by_id(settlement_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find settlement (settlement_id): {}): {}",
                settlement_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(model)
}

async fn insert_stopped_settlement(
    txn: &DatabaseTransaction,
    model: entity::settlement::Model,
    stopped_date_time: DateTime<FixedOffset>,
) -> Result<(), ErrRespStruct> {
    let active_model = entity::stopped_settlement::ActiveModel {
        stopped_settlement_id: NotSet,
        consultation_id: Set(model.consultation_id),
        charge_id: Set(model.charge_id.clone()),
        fee_per_hour_in_yen: Set(model.fee_per_hour_in_yen),
        platform_fee_rate_in_percentage: Set(model.platform_fee_rate_in_percentage.clone()),
        credit_facilities_expired_at: Set(model.credit_facilities_expired_at),
        stopped_at: Set(stopped_date_time),
    };
    let _ = active_model.insert(txn).await.map_err(|e| {
        error!(
            "failed to insert stopped_settlement (settlement: {:?}, stopped_date_time: {}): {}",
            model, stopped_date_time, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

async fn insert_deleted_user_account(
    txn: &DatabaseTransaction,
    model: &entity::user_account::Model,
    deleted_date_time: DateTime<FixedOffset>,
) -> Result<(), ErrRespStruct> {
    let active_model = entity::deleted_user_account::ActiveModel {
        user_account_id: Set(model.user_account_id),
        email_address: Set(model.email_address.clone()),
        hashed_password: Set(model.hashed_password.clone()),
        last_login_time: Set(model.last_login_time),
        created_at: Set(model.created_at),
        mfa_enabled_at: Set(model.mfa_enabled_at),
        disabled_at: Set(model.disabled_at),
        deleted_at: Set(deleted_date_time),
    };
    let _ = active_model.insert(txn).await.map_err(|e| {
        error!(
            "failed to insert deleted_user_account (user_account: {:?}, deleted_date_time: {}): {}",
            model, deleted_date_time, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

async fn handle_delete_accounts(
    account_id: i64,
    email_address: String,
    current_date_time: DateTime<FixedOffset>,
    op: &impl DeleteAccountsOperation,
) -> RespResult<DeleteAccountsResult> {
    let settlement_ids = op.get_settlement_ids(account_id).await?;
    for s_id in settlement_ids {
        op.stop_payment(s_id, current_date_time).await?;
    }

    // 削除成功のメール通知
    Ok((StatusCode::OK, Json(DeleteAccountsResult {})))
}
