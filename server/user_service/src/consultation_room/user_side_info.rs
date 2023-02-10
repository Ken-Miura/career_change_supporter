// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use common::{ApiError, ErrResp, ErrRespStruct, RespResult, JAPANESE_TIME_ZONE};
use entity::consultation;
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, Set, TransactionError,
    TransactionTrait,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};
use crate::util;
use crate::util::available_user_account::UserAccount;
use crate::util::session::User;

use super::{
    create_sky_way_auth_token, create_sky_way_auth_token_payload,
    get_consultation_with_exclusive_lock, validate_consultation_id_is_positive, Consultation,
    SkyWayIdentification, LEEWAY_IN_MINUTES, VALID_TOKEN_DURATION_IN_SECONDS,
};

pub(crate) async fn get_user_side_info(
    User { account_id }: User,
    query: Query<UserSideInfoQuery>,
    State(pool): State<DatabaseConnection>,
) -> RespResult<UserSideInfoResult> {
    let consultation_id = query.0.consultation_id;
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let identification = SkyWayIdentification {
        application_id: "6668affc-5afa-4996-b65a-6afe2f72756b".to_string(),
        secret: "dummy".to_string(),
    };
    let token_id = "6668affc-5afa-4996-b65a-6afe2f72756b";
    let op = UserSideInfoOperationImpl { pool };
    handle_user_side_info(
        account_id,
        consultation_id,
        &current_date_time,
        identification,
        token_id,
        op,
    )
    .await
}

#[derive(Deserialize)]
pub(crate) struct UserSideInfoQuery {
    consultation_id: i64,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct UserSideInfoResult {
    token: String,
    room_name: String,
    member_name: String,
}

async fn handle_user_side_info(
    account_id: i64,
    consultation_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    identification: SkyWayIdentification,
    token_id: &str,
    op: impl UserSideInfoOperation,
) -> RespResult<UserSideInfoResult> {
    validate_consultation_id_is_positive(consultation_id)?;
    validate_identity_exists(account_id, &op).await?;
    let result = get_consultation_by_consultation_id(consultation_id, &op).await?;
    ensure_user_account_id_is_valid(result.user_account_id, account_id)?;
    let _ = get_consultant_if_available(result.consultant_id, &op).await?;
    let _ = get_user_account_if_available(result.user_account_id, &op).await?;
    let leeway = Duration::minutes(LEEWAY_IN_MINUTES);
    let criteria = result.consultation_date_time_in_jst - leeway;
    if *current_date_time < criteria {
        error!("consultation room has not opened yet (current_date_time: {}, consultation_date_time_in_jst: {}, leeway: {})", 
            current_date_time, result.consultation_date_time_in_jst, leeway);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultationRoomHasNotOpenedYet as u32,
            }),
        ));
    }

    let room_name = op
        .find_room_name_by_consultation_id(consultation_id)
        .await?;
    let room_name = room_name.ok_or_else(|| {
        error!(
            "failed to find room_name (consultation_id: {})",
            consultation_id
        );
        unexpected_err_resp()
    })?;

    let expiration_date_time =
        *current_date_time + Duration::seconds(VALID_TOKEN_DURATION_IN_SECONDS);
    let payload = create_sky_way_auth_token_payload(
        token_id.to_string(),
        *current_date_time,
        expiration_date_time,
        identification.application_id,
        room_name.clone(),
        account_id.to_string(),
    )?;
    let token = create_sky_way_auth_token(&payload, identification.secret.as_bytes())?;

    Ok((
        StatusCode::OK,
        Json(UserSideInfoResult {
            token,
            room_name,
            member_name: account_id.to_string(),
        }),
    ))
}

#[async_trait]
trait UserSideInfoOperation {
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

    async fn find_room_name_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<String>, ErrResp>;

    async fn update_user_account_entered_at_if_needed(
        &self,
        consultation_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp>;
}

struct UserSideInfoOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl UserSideInfoOperation for UserSideInfoOperationImpl {
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

    async fn find_room_name_by_consultation_id(
        &self,
        consultation_id: i64,
    ) -> Result<Option<String>, ErrResp> {
        super::find_room_name_by_consultation_id(&self.pool, consultation_id).await
    }

    async fn update_user_account_entered_at_if_needed(
        &self,
        consultation_id: i64,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<(), ErrResp> {
        self.pool
            .transaction::<_, (), ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let result = get_consultation_with_exclusive_lock(consultation_id, txn).await?;
                    if result.user_account_entered_at.is_some() {
                        return Ok(());
                    }
                    update_user_account_entered_at(current_date_time, result, txn).await?;
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
                        "failed to update_user_account_entered_at_if_needed: {}",
                        err_resp_struct
                    );
                    err_resp_struct.err_resp
                }
            })?;
        Ok(())
    }
}

async fn validate_identity_exists(
    account_id: i64,
    op: &impl UserSideInfoOperation,
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
    op: &impl UserSideInfoOperation,
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

fn ensure_user_account_id_is_valid(
    user_account_id_in_consultation: i64,
    user_account_id: i64,
) -> Result<(), ErrResp> {
    if user_account_id_in_consultation != user_account_id {
        error!(
            "user_account_id in consultation ({}) is not same as passed user_accound_id ({})",
            user_account_id_in_consultation, user_account_id
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
    op: &impl UserSideInfoOperation,
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
    op: &impl UserSideInfoOperation,
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

async fn update_user_account_entered_at(
    current_date_time: DateTime<FixedOffset>,
    model: consultation::Model,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    let consultation_id = model.consultation_id;
    let mut active_model: consultation::ActiveModel = model.into();
    active_model.user_account_entered_at = Set(Some(current_date_time));
    let _ = active_model.update(txn).await.map_err(|e| {
        error!("failed to update user_account_entered_at (consultation_id: {}, current_date_time: {}): {}",
        consultation_id, current_date_time, e);
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}
