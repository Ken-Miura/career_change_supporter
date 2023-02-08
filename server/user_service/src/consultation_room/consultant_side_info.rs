// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use common::util::validator::uuid_validator::validate_uuid;
use common::{ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
use entity::consultation;
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, Set, TransactionError,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use tracing::error;
use uuid::Uuid;

use crate::err::{unexpected_err_resp, Code};
use crate::util;
use crate::util::available_user_account::UserAccount;
use crate::util::session::User;

use super::{
    create_sky_way_credential, get_consultation_with_exclusive_lock,
    validate_consultation_id_is_positive, Consultation, SkyWayCredential,
    TIME_BEFORE_CONSULTATION_STARTS_IN_MINUTES,
};

pub(crate) async fn get_consultant_side_info(
    User { account_id }: User,
    query: Query<ConsultantSideInfoQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<ConsultantSideInfoResult> {
    let consultation_id = query.0.consultation_id;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let peer_id = Uuid::new_v4().simple().to_string();
    let op = ConsultantSideInfoOperationImpl { pool };
    handle_consultant_side_info(
        account_id,
        consultation_id,
        &current_date_time,
        peer_id.as_str(),
        op,
    )
    .await
}

#[derive(Deserialize)]
pub(crate) struct ConsultantSideInfoQuery {
    consultation_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultantSideInfoResult {
    consultant_peer_id: String,
    credential: SkyWayCredential,
    user_account_peer_id: Option<String>,
}

async fn handle_consultant_side_info(
    account_id: i64,
    consultation_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    peer_id: &str,
    op: impl ConsultantSideInfoOperation,
) -> RespResult<ConsultantSideInfoResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    validate_uuid(peer_id).map_err(|e| {
        error!("failed to validate {}: {}", peer_id, e);
        // peer_idは、ユーザーから渡されるものではなく、サーバで生成するものなので失敗はunexpected_err_resp
        unexpected_err_resp()
    })?;
    validate_identity_exists(account_id, &op).await?;
    let result = get_consultation_by_consultation_id(consultation_id, &op).await?;
    ensure_consultant_id_is_valid(result.consultant_id, account_id)?;
    let _ = get_consultant_if_available(result.consultant_id, &op).await?;
    let _ = get_user_account_if_available(result.user_account_id, &op).await?;
    let offset = Duration::minutes(TIME_BEFORE_CONSULTATION_STARTS_IN_MINUTES);
    let criteria = result.consultation_date_time_in_jst - offset;
    if *current_date_time < criteria {
        error!("consultation room has not opened yet (current_date_time: {}, consultation_date_time_in_jst: {}, offset: {})", 
            current_date_time, result.consultation_date_time_in_jst, offset);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultationRoomHasNotOpenedYet as u32,
            }),
        ));
    }

    let updated_result = op
        .update_consultation_if_needed(consultation_id, *current_date_time, peer_id.to_string())
        .await?;

    let consultant_peer_id = updated_result.consultant_peer_id.ok_or_else(|| {
        error!(
            "consultant_peer_id is None (consultation_id: {})",
            consultation_id
        );
        unexpected_err_resp()
    })?;

    let timestamp = current_date_time.timestamp();
    let credential = create_sky_way_credential(consultant_peer_id.as_str(), timestamp)?;

    Ok((
        StatusCode::OK,
        Json(ConsultantSideInfoResult {
            consultant_peer_id,
            credential,
            user_account_peer_id: updated_result.user_account_peer_id,
        }),
    ))
}

#[async_trait]
trait ConsultantSideInfoOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;

    async fn find_consultation_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<Consultation>, ErrResp>;

    /// コンサルタントが利用可能な場合（UserAccountが存在し、かつdisabled_atがNULLである場合）、[UserAccount]を返す
    async fn get_consultant_if_available(
        &self,
        consultant_id: i64,
    ) -> Result<Option<UserAccount>, ErrResp>;

    /// ユーザーが利用可能な場合（UserAccountが存在し、かつdisabled_atがNULLである場合）、[UserAccount]を返す
    async fn get_user_account_if_available(
        &self,
        user_account_id: i64,
    ) -> Result<Option<UserAccount>, ErrResp>;

    async fn update_consultation_if_needed(
        &self,
        consultation_id: i64,
        current_date_time: DateTime<FixedOffset>,
        peer_id: String,
    ) -> Result<Consultation, ErrResp>;
}

struct ConsultantSideInfoOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultantSideInfoOperation for ConsultantSideInfoOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::identity_checker::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn find_consultation_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<Consultation>, ErrResp> {
        super::find_consultation_by_consultation_id(consultation_id, &self.pool).await
    }

    async fn get_consultant_if_available(
        &self,
        consultant_id: i64,
    ) -> Result<Option<UserAccount>, ErrResp> {
        util::available_user_account::get_if_user_account_is_available(&self.pool, consultant_id)
            .await
    }

    async fn get_user_account_if_available(
        &self,
        user_account_id: i64,
    ) -> Result<Option<UserAccount>, ErrResp> {
        util::available_user_account::get_if_user_account_is_available(&self.pool, user_account_id)
            .await
    }

    async fn update_consultation_if_needed(
        &self,
        consultation_id: i64,
        current_date_time: DateTime<FixedOffset>,
        peer_id: String,
    ) -> Result<Consultation, ErrResp> {
        let updated_result = self
            .pool
            .transaction::<_, Consultation, ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let result = get_consultation_with_exclusive_lock(consultation_id, txn).await?;

                    // if result.consultant_peer_id.is_some() {
                    //     return Ok(Consultation {
                    //         user_account_id: result.user_account_id,
                    //         consultant_id: result.consultant_id,
                    //         consultation_date_time_in_jst: result
                    //             .meeting_at
                    //             .with_timezone(&(*JAPANESE_TIME_ZONE)),
                    //         user_account_peer_id: result.user_account_peer_id,
                    //         consultant_peer_id: result.consultant_peer_id,
                    //     });
                    // }

                    let updated_result =
                        update_consultant_side_info(peer_id, current_date_time, result, txn)
                            .await?;

                    todo!()
                    // Ok(Consultation {
                    //     user_account_id: updated_result.user_account_id,
                    //     consultant_id: updated_result.consultant_id,
                    //     consultation_date_time_in_jst: updated_result
                    //         .meeting_at
                    //         .with_timezone(&(*JAPANESE_TIME_ZONE)),
                    //     user_account_peer_id: updated_result.user_account_peer_id,
                    //     consultant_peer_id: updated_result.consultant_peer_id,
                    // })
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
                        "failed to update_consultation_if_needed: {}",
                        err_resp_struct
                    );
                    err_resp_struct.err_resp
                }
            })?;
        Ok(updated_result)
    }
}

async fn validate_identity_exists(
    account_id: i64,
    op: &impl ConsultantSideInfoOperation,
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

async fn get_consultation_by_consultation_id(
    consultation_id: i64,
    op: &impl ConsultantSideInfoOperation,
) -> Result<Consultation, ErrResp> {
    let consultation_option = op
        .find_consultation_by_consultation_id(consultation_id)
        .await?;
    if let Some(c) = consultation_option {
        Ok(c)
    } else {
        error!(
            "no consultation (consultation_id: {}) found",
            consultation_id
        );
        Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationFound as u32,
            }),
        ))
    }
}

fn ensure_consultant_id_is_valid(
    consultant_id_in_consultation: i64,
    consultant_id: i64,
) -> Result<(), ErrResp> {
    if consultant_id_in_consultation != consultant_id {
        error!(
            "consultant_id in consultation ({}) is not same as passed consultant_id ({})",
            consultant_id_in_consultation, consultant_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationFound as u32,
            }),
        ));
    }
    Ok(())
}

async fn get_consultant_if_available(
    consultant_id: i64,
    op: &impl ConsultantSideInfoOperation,
) -> Result<UserAccount, ErrResp> {
    let consultant = op.get_consultant_if_available(consultant_id).await?;
    consultant.ok_or_else(|| {
        error!("consultant ({}) is not available", consultant_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultantIsNotAvailableOnConsultationRoom as u32,
            }),
        )
    })
}

async fn get_user_account_if_available(
    user_account_id: i64,
    op: &impl ConsultantSideInfoOperation,
) -> Result<UserAccount, ErrResp> {
    let user = op.get_user_account_if_available(user_account_id).await?;
    user.ok_or_else(|| {
        error!("user ({}) is not available", user_account_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::UserIsNotAvailableOnConsultationRoom as u32,
            }),
        )
    })
}

async fn update_consultant_side_info(
    peer_id: String,
    current_date_time: DateTime<FixedOffset>,
    model: consultation::Model,
    txn: &DatabaseTransaction,
) -> Result<consultation::Model, ErrRespStruct> {
    // let consultation_id = model.consultation_id;
    // let mut active_model: consultation::ActiveModel = model.into();
    // active_model.consultant_peer_id = Set(Some(peer_id.clone()));
    // active_model.consultant_peer_opend_at = Set(Some(current_date_time));
    // let updated_result = active_model.update(txn).await.map_err(|e| {
    //     error!("failed to update consultation (consultation_id: {}, peer_id: {}, current_date_time: {}): {}",
    //     consultation_id, peer_id, current_date_time, e);
    //     ErrRespStruct {
    //         err_resp: unexpected_err_resp(),
    //     }
    // })?;
    // Ok(updated_result)
    todo!()
}
