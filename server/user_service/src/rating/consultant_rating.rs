// Copyright 2023 Ken Miura

use async_session::serde_json::json;
use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, FixedOffset, Utc};
use common::opensearch::{update_document, INDEX_NAME};
use common::{ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
use entity::prelude::{ConsultantRating, Consultation};
use entity::sea_orm::ActiveValue::NotSet;
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait,
    QueryFilter, QuerySelect, Set, TransactionError, TransactionTrait,
};
use entity::{consultant_rating, consultation, receipt, settlement};
use opensearch::OpenSearch;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::err::{unexpected_err_resp, Code};
use crate::util::disabled_check::DisabledCheckOperationImpl;
use crate::util::document_operation::find_document_model_by_user_account_id_with_shared_lock;
use crate::util::session::User;
use crate::util::{self, find_user_account_by_user_account_id_with_exclusive_lock};

use super::{
    ensure_end_of_consultation_date_time_has_passed, ensure_rating_id_is_positive,
    ensure_rating_is_in_valid_range, ConsultationInfo,
};

pub(crate) async fn post_consultant_rating(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
    State(index_client): State<OpenSearch>,
    Json(req): Json<ConsultantRatingParam>,
) -> RespResult<ConsultantRatingResult> {
    let op = ConsultantRatingOperationImpl { pool, index_client };
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    handle_consultant_rating(
        account_id,
        req.consultant_rating_id,
        req.rating,
        &current_date_time,
        op,
    )
    .await
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ConsultantRatingParam {
    consultant_rating_id: i64,
    rating: i16,
}

#[derive(Serialize, Debug, PartialEq)]
pub(crate) struct ConsultantRatingResult {}

#[async_trait]
trait ConsultantRatingOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;

    async fn check_if_user_account_is_available(
        &self,
        user_account_id: i64,
    ) -> Result<bool, ErrResp>;

    async fn find_consultation_info_from_consultant_rating(
        &self,
        consultant_rating_id: i64,
    ) -> Result<Option<ConsultationInfo>, ErrResp>;

    async fn update_consultant_rating(
        &self,
        consultant_id: i64,
        consultant_rating_id: i64,
        rating: i16,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;

    async fn filter_consultant_rating_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Vec<i16>, ErrResp>;

    async fn update_rating_on_document_if_not_disabled(
        &self,
        consultant_id: i64,
        averate_rating: f64,
        num_of_rated: i32,
    ) -> Result<(), ErrResp>;

    async fn make_payment_if_needed(
        &self,
        consultation_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct ConsultantRatingOperationImpl {
    pool: DatabaseConnection,
    index_client: OpenSearch,
}

#[async_trait]
impl ConsultantRatingOperation for ConsultantRatingOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::identity_checker::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn check_if_user_account_is_available(
        &self,
        user_account_id: i64,
    ) -> Result<bool, ErrResp> {
        let op = DisabledCheckOperationImpl::new(&self.pool);
        util::disabled_check::check_if_user_account_is_available(user_account_id, op).await
    }

    async fn find_consultation_info_from_consultant_rating(
        &self,
        consultant_rating_id: i64,
    ) -> Result<Option<ConsultationInfo>, ErrResp> {
        let model = entity::consultant_rating::Entity::find_by_id(consultant_rating_id)
            .find_also_related(Consultation)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find consultant_rating and consultation (consultant_rating_id: {}): {}",
                    consultant_rating_id, e
                );
                unexpected_err_resp()
            })?;
        let converted_result = model.map(|m| {
            let c = m.1.ok_or_else(|| {
                error!(
                    "failed to find consultation (consultant_rating_id: {}, consultation_id: {})",
                    consultant_rating_id, m.0.consultation_id
                );
                unexpected_err_resp()
            })?;
            Ok(ConsultationInfo {
                consultation_id: c.consultation_id,
                user_account_id: c.user_account_id,
                consultant_id: c.consultant_id,
                consultation_date_time_in_jst: c.meeting_at.with_timezone(&(*JAPANESE_TIME_ZONE)),
            })
        });
        Ok(match converted_result {
            Some(r) => Some(r?),
            None => None,
        })
    }

    async fn update_consultant_rating(
        &self,
        consultant_id: i64,
        consultant_rating_id: i64,
        rating: i16,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    // 同じコンサルタントに対する複数のconsultant_ratingの更新が来た場合に備えて
                    // また、consultant_rating更新中にコンサルタントが自身のアカウントを削除する場合に備えてuser_accountで排他ロックを取得しておく
                    let consultant_option =
                        find_user_account_by_user_account_id_with_exclusive_lock(
                            txn,
                            consultant_id,
                        )
                        .await?;
                    if consultant_option.is_none() {
                        info!(
                            "no consultant (consultant_id: {}) found on rating",
                            consultant_id
                        );
                        return Ok(());
                    }
                    let model_option =
                        entity::consultant_rating::Entity::find_by_id(consultant_rating_id)
                            .one(txn)
                            .await
                            .map_err(|e| {
                                error!(
                                "failed to find consultant_rating (consultant_rating_id: {}): {}",
                                consultant_rating_id, e
                            );
                                ErrRespStruct {
                                    err_resp: unexpected_err_resp(),
                                }
                            })?;
                    let model = match model_option {
                        Some(m) => m,
                        None => {
                            error!(
                                "no consultant_rating (consultant_rating_id: {}) found on rating",
                                consultant_rating_id
                            );
                            return Err(ErrRespStruct {
                                err_resp: unexpected_err_resp(),
                            });
                        }
                    };
                    if model.rating.is_some() {
                        return Err(ErrRespStruct {
                            err_resp: (
                                StatusCode::BAD_REQUEST,
                                Json(ApiError {
                                    code: Code::ConsultantHasAlreadyBeenRated as u32,
                                }),
                            ),
                        });
                    }
                    update_consultant_rating(model, txn, rating, current_date_time).await?;
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
                    error!("failed to update_consultant_rating: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }

    async fn filter_consultant_rating_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Vec<i16>, ErrResp> {
        let models = consultation::Entity::find()
            .filter(consultation::Column::ConsultantId.eq(consultant_id))
            .find_with_related(ConsultantRating)
            .filter(consultant_rating::Column::Rating.is_not_null())
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter consultant_rating (consultant_id: {}): {}",
                    consultant_id, e
                );
                unexpected_err_resp()
            })?;
        models
            .into_iter()
            .map(|m| {
                // consultationとconsultant_ratingは1対1の設計なので取れない場合は想定外エラーとして扱う
                let cr = m.1.get(0).ok_or_else(|| {
                    error!(
                        "failed to find consultant_rating (consultation_id: {})",
                        m.0.consultation_id
                    );
                    unexpected_err_resp()
                })?;
                let r = cr.rating.ok_or_else(|| {
                    error!(
                        "rating is null (consultant_rating_id: {}, consultant_id: {})",
                        cr.consultant_rating_id, m.0.consultant_id
                    );
                    unexpected_err_resp()
                })?;
                Ok(r)
            })
            .collect::<Result<Vec<i16>, ErrResp>>()
    }

    async fn update_rating_on_document_if_not_disabled(
        &self,
        consultant_id: i64,
        averate_rating: f64,
        num_of_rated: i32,
    ) -> Result<(), ErrResp> {
        let index_client = self.index_client.clone();
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    // 管理者がコンサルタントをDisabledにしている途中に
                    // ユーザーがコンサルタントのratingの更新をした場合に備えて、user_accountで排他ロックを取得しておく
                    let consultant_option =
                        find_user_account_by_user_account_id_with_exclusive_lock(
                            txn,
                            consultant_id,
                        )
                        .await?;
                    let consultant = match consultant_option {
                        Some(c) => c,
                        None => {
                            info!(
                                "no consultant (consultant_id: {}) found on rating",
                                consultant_id
                            );
                            return Ok(());
                        }
                    };
                    if consultant.disabled_at.is_some() {
                        info!("do not update rating on document because consultant (consultant_id: {}) is disabled", consultant_id);
                        return Ok(());
                    }

                    let doc_option = find_document_model_by_user_account_id_with_shared_lock(txn, consultant_id).await?;
                    let doc = match doc_option {
                        Some(d) => d,
                        None => {
                            // アカウントを排他ロックし、Disabledでないことを確認済みのため、documentが存在しないことはないはず。従ってエラーログとして記録する。
                            // 一方で、ユーザーにまでこのエラーを返すのは適切でないため、Okとして処理する。
                            // エラーを返すのが適切ではないと考えたのは次の通り
                            // - このエラーを解消しないとユーザーは正しく操作を終了できないわけではない
                            // - 偶発的に起きた問題の場合、次回の評価時に正しく平均評価が反映される
                            error!("no document found on rate update(consultant_id: {})", consultant_id);
                            return Ok(());
                        }
                    };
                    update_rating_info_on_document(INDEX_NAME, doc.document_id.to_string().as_str(), averate_rating, num_of_rated, index_client).await?;
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
                    error!(
                        "failed to update_rating_on_document_if_not_disabled: {}",
                        err_resp_struct
                    );
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }

    async fn make_payment_if_needed(
        &self,
        consultation_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let settlement_option = find_settlement_by_consultation_id_with_exclusive_lock(
                        txn,
                        consultation_id,
                    )
                    .await?;
                    let stl = match settlement_option {
                        Some(s) => s,
                        None => {
                            // 特定の期間ユーザーが評価をせず、定期実行ツールによって自動決済されている場合、既にsettlementはない
                            info!(
                                "no settlement found (consultation_id: {}) on rating",
                                consultation_id
                            );
                            return Ok(());
                        }
                    };
                    let charge_id = stl.charge_id.clone();
                    insert_receipt(txn, stl, current_date_time).await?;
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
                    error!("failed to make_payment_if_needed: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

async fn update_consultant_rating(
    model: entity::consultant_rating::Model,
    txn: &DatabaseTransaction,
    rating: i16,
    current_date_time: DateTime<FixedOffset>,
) -> Result<(), ErrRespStruct> {
    let consultant_rating_id = model.consultant_rating_id;
    let mut active_model: entity::consultant_rating::ActiveModel = model.into();
    active_model.rating = Set(Some(rating));
    active_model.rated_at = Set(Some(current_date_time));
    let _ = active_model.update(txn).await.map_err(|e| {
        error!(
            "failed to update consultant_rating (consultant_rating_id: {}, rating: {}, current_date_time: {}): {}",
            consultant_rating_id, rating, current_date_time, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

async fn update_rating_info_on_document(
    index_name: &str,
    document_id: &str,
    averate_rating: f64,
    num_of_rated: i32,
    client: OpenSearch,
) -> Result<(), ErrRespStruct> {
    let script = json!({
        "doc": {
            "rating": averate_rating,
            "num_of_rated": num_of_rated
        }
    });
    update_document(index_name, document_id, &script, &client)
        .await
        .map_err(|e| {
            error!(
                "failed to update rating info into document (document_id: {}, averate_rating: {}, num_of_rated: {})",
                document_id, averate_rating, num_of_rated
            );
            ErrRespStruct { err_resp: e }
        })?;
    Ok(())
}

async fn find_settlement_by_consultation_id_with_exclusive_lock(
    txn: &DatabaseTransaction,
    consultation_id: i64,
) -> Result<Option<settlement::Model>, ErrRespStruct> {
    let model = entity::prelude::Settlement::find_by_id(consultation_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find settlement (consultation_id): {}): {}",
                consultation_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(model)
}

async fn insert_receipt(
    txn: &DatabaseTransaction,
    model: settlement::Model,
    current_date_time: DateTime<FixedOffset>,
) -> Result<(), ErrRespStruct> {
    let consultation_id = model.consultation_id;
    let active_model = receipt::ActiveModel {
        receipt_id: NotSet,
        consultation_id: Set(model.consultation_id),
        charge_id: Set(model.charge_id),
        fee_per_hour_in_yen: Set(model.fee_per_hour_in_yen),
        platform_fee_rate_in_percentage: Set(model.platform_fee_rate_in_percentage),
        settled_at: Set(current_date_time),
    };
    let _ = active_model.insert(txn).await.map_err(|e| {
        error!(
            "failed to insert receipt (consultation_id: {}): {}",
            consultation_id, e
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

async fn handle_consultant_rating(
    account_id: i64,
    consultant_rating_id: i64,
    rating: i16,
    current_date_time: &DateTime<FixedOffset>,
    op: impl ConsultantRatingOperation,
) -> RespResult<ConsultantRatingResult> {
    ensure_rating_id_is_positive(consultant_rating_id)?;
    ensure_rating_is_in_valid_range(rating)?;
    ensure_identity_exists(account_id, &op).await?;
    ensure_user_account_is_available(account_id, &op).await?;

    let cl = get_consultation_info_from_consultation_rating(consultant_rating_id, &op).await?;
    ensure_user_account_ids_are_same(account_id, cl.user_account_id)?;
    ensure_end_of_consultation_date_time_has_passed(
        &cl.consultation_date_time_in_jst,
        current_date_time,
    )?;

    op.update_consultant_rating(
        cl.consultant_id,
        consultant_rating_id,
        rating,
        *current_date_time,
    )
    .await?;

    let ratings = op
        .filter_consultant_rating_by_consultant_id(cl.consultant_id)
        .await?;
    let num_of_rated = ratings.len() as i32;
    let average_rating = calculate_average_rating(ratings);
    op.update_rating_on_document_if_not_disabled(cl.consultant_id, average_rating, num_of_rated)
        .await?;

    op.make_payment_if_needed(cl.consultation_id, *current_date_time)
        .await?;

    Ok((StatusCode::OK, Json(ConsultantRatingResult {})))
}

async fn ensure_identity_exists(
    account_id: i64,
    op: &impl ConsultantRatingOperation,
) -> Result<(), ErrResp> {
    let identity_exists = op.check_if_identity_exists(account_id).await?;
    if !identity_exists {
        error!("identity is not registered (account_id: {})", account_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoIdentityRegistered as u32,
            }),
        ));
    }
    Ok(())
}

async fn ensure_user_account_is_available(
    user_account_id: i64,
    op: &impl ConsultantRatingOperation,
) -> Result<(), ErrResp> {
    let available = op
        .check_if_user_account_is_available(user_account_id)
        .await?;
    if !available {
        error!(
            "user account is not available (user_account_id: {})",
            user_account_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::UserIsNotAvailable as u32,
            }),
        ));
    }
    Ok(())
}

async fn get_consultation_info_from_consultation_rating(
    consultation_rating_id: i64,
    op: &impl ConsultantRatingOperation,
) -> Result<ConsultationInfo, ErrResp> {
    let cl = op
        .find_consultation_info_from_consultant_rating(consultation_rating_id)
        .await?;
    match cl {
        Some(c) => Ok(c),
        None => Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationRatingFound as u32,
            }),
        )),
    }
}

fn ensure_user_account_ids_are_same(
    user_account_id: i64,
    user_account_id_in_consultation_info: i64,
) -> Result<(), ErrResp> {
    if user_account_id != user_account_id_in_consultation_info {
        error!(
            "user_account_id ({}) and user_account_id_in_consultation_info ({}) are not same",
            user_account_id, user_account_id_in_consultation_info
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationRatingFound as u32,
            }),
        ));
    }
    Ok(())
}

fn calculate_average_rating(ratings: Vec<i16>) -> f64 {
    let size = ratings.len();
    let mut sum = 0;
    for rating in ratings {
        sum += rating as usize
    }
    (sum / size) as f64
}
