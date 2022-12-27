// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, Duration, FixedOffset, Utc};
use common::smtp::{SmtpClient, SMTP_HOST, SMTP_PASSWORD, SMTP_PORT, SMTP_USERNAME};
use common::{smtp::SendMail, RespResult, JAPANESE_TIME_ZONE};
use common::{ApiError, ErrResp, ErrRespStruct};
use entity::sea_orm::ActiveValue::NotSet;
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QuerySelect, Set,
    TransactionError, TransactionTrait,
};
use entity::{consultation, consultation_req, user_rating};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::err::{unexpected_err_resp, Code};
use crate::util::session::User;
use crate::util::{
    self, available_user_account::UserAccount, consultation_request::consultation_req_exists,
    consultation_request::ConsultationRequest,
    optional_env_var::MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE,
    validator::consultation_req_id_validator::validate_consultation_req_id_is_positive,
};

pub(crate) async fn post_consultation_request_acceptance(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
    Json(param): Json<ConsultationRequestAcceptanceParam>,
) -> RespResult<ConsultationRequestAcceptanceResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = ConsultationRequestAcceptanceOperationImpl { pool };
    let smtp_client = SmtpClient::new(
        SMTP_HOST.to_string(),
        *SMTP_PORT,
        SMTP_USERNAME.to_string(),
        SMTP_PASSWORD.to_string(),
    );
    handle_consultation_request_acceptance(account_id, &param, &current_date_time, op, smtp_client)
        .await
}

#[derive(Deserialize)]
pub(crate) struct ConsultationRequestAcceptanceParam {
    pub(crate) consultation_req_id: i64,
    pub(crate) picked_candidate: u8,
    pub(crate) user_checked: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationRequestAcceptanceResult {}

async fn handle_consultation_request_acceptance(
    user_account_id: i64,
    param: &ConsultationRequestAcceptanceParam,
    current_date_time: &DateTime<FixedOffset>,
    op: impl ConsultationRequestAcceptanceOperation,
    send_mail: impl SendMail,
) -> RespResult<ConsultationRequestAcceptanceResult> {
    let picked_candidate = param.picked_candidate;
    validate_picked_candidate(picked_candidate)?;
    info!(
        "consultant (consultant id: {}) picked candidate: {}",
        user_account_id, picked_candidate
    );
    validate_user_checked_confirmation_items(param.user_checked, user_account_id)?;
    let consultation_req_id = param.consultation_req_id;
    validate_consultation_req_id_is_positive(consultation_req_id)?;
    validate_identity_exists(user_account_id, &op).await?;

    let req = op
        .find_consultation_req_by_consultation_req_id(consultation_req_id)
        .await?;
    let req = consultation_req_exists(req, consultation_req_id)?;
    validate_consultation_req_for_acceptance(&req, user_account_id, current_date_time)?;

    let consultant = get_consultant_if_available(req.consultant_id, &op).await?;
    let user = get_user_account_if_available(req.user_account_id, &op).await?;

    let _consultation = op
        .accept_consultation_req(consultation_req_id, picked_candidate)
        .await?;
    // TODO: メール送信
    println!("{:?}, {:?}", consultant.email_address, user.email_address);
    todo!()
}

#[async_trait]
trait ConsultationRequestAcceptanceOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;

    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp>;

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

    async fn accept_consultation_req(
        &self,
        consultation_req_id: i64,
        picked_candidate: u8,
    ) -> Result<Consultation, ErrResp>;
}

#[derive(Clone, Debug)]
struct Consultation {
    consultation_req_id: i64,
    fee_per_hour_in_yen: i32,
    consultation_date_time_in_jst: DateTime<FixedOffset>,
}

struct ConsultationRequestAcceptanceOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl ConsultationRequestAcceptanceOperation for ConsultationRequestAcceptanceOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::identity_checker::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp> {
        util::consultation_request::find_consultation_req_by_consultation_req_id(
            &self.pool,
            consultation_req_id,
        )
        .await
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

    async fn accept_consultation_req(
        &self,
        consultation_req_id: i64,
        picked_candidate: u8,
    ) -> Result<Consultation, ErrResp> {
        let consultation = self
            .pool
            .transaction::<_, Consultation, ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let req =
                        get_consultation_req_with_exclusive_lock(consultation_req_id, txn).await?;

                    let meeting_date_time = select_meeting_date_time(
                        &req.first_candidate_date_time,
                        &req.second_candidate_date_time,
                        &req.third_candidate_date_time,
                        picked_candidate,
                    )?;

                    create_consultation(&req, &meeting_date_time, txn).await?;
                    create_user_rating(&req, &meeting_date_time, txn).await?;
                    create_settlement(&req, &meeting_date_time, txn).await?;
                    // consultant_ratingをinsert
                    // consultation_reqをdelete

                    Ok(Consultation {
                        consultation_req_id: req.consultation_req_id,
                        fee_per_hour_in_yen: req.fee_per_hour_in_yen,
                        consultation_date_time_in_jst: meeting_date_time
                            .with_timezone(&(*JAPANESE_TIME_ZONE)),
                    })
                })
            })
            .await
            .map_err(|e| match e {
                TransactionError::Connection(db_err) => {
                    error!("connection error: {}", db_err);
                    unexpected_err_resp()
                }
                TransactionError::Transaction(err_resp_struct) => {
                    error!("failed to accept_consultation_req: {}", err_resp_struct);
                    err_resp_struct.err_resp
                }
            })?;
        Ok(consultation)
    }
}

async fn get_consultation_req_with_exclusive_lock(
    consultation_req_id: i64,
    txn: &DatabaseTransaction,
) -> Result<consultation_req::Model, ErrRespStruct> {
    let req = consultation_req::Entity::find_by_id(consultation_req_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find consultation_req (consultation_req_id: {}): {}",
                consultation_req_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    req.ok_or_else(|| {
        error!(
            "failed to get consultation_req (consultation_req_id: {})",
            consultation_req_id
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })
}

fn select_meeting_date_time(
    first_candidate_date_time: &DateTime<FixedOffset>,
    second_candidate_date_time: &DateTime<FixedOffset>,
    third_candidate_date_time: &DateTime<FixedOffset>,
    picked_candidate: u8,
) -> Result<DateTime<FixedOffset>, ErrRespStruct> {
    if picked_candidate == 1 {
        Ok(*first_candidate_date_time)
    } else if picked_candidate == 2 {
        Ok(*second_candidate_date_time)
    } else if picked_candidate == 3 {
        Ok(*third_candidate_date_time)
    } else {
        error!("invalid picked_candidate ({})", picked_candidate);
        Err(ErrRespStruct {
            err_resp: unexpected_err_resp(),
        })
    }
}

async fn create_consultation(
    req: &consultation_req::Model,
    meeting_date_time: &DateTime<FixedOffset>,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    let active_model = consultation::ActiveModel {
        consultation_id: NotSet,
        user_account_id: Set(req.user_account_id),
        consultant_id: Set(req.consultant_id),
        meeting_at: Set(*meeting_date_time),
        charge_id: Set(req.charge_id.clone()),
        user_account_peer_id: NotSet,
        user_account_peer_opened_at: NotSet,
        consultant_peer_id: NotSet,
        consultant_peer_opend_at: NotSet,
        consultation_started_at: NotSet,
    };
    let _ = active_model.insert(txn).await.map_err(|e| {
        error!("failed to insert consultation (user_account_id: {}, consultant_id: {}, meeting_at: {}, charge_id: {}): {}", 
            req.user_account_id, req.consultant_id, meeting_date_time, req.charge_id, e);
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

async fn create_user_rating(
    req: &consultation_req::Model,
    meeting_date_time: &DateTime<FixedOffset>,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    let active_model = user_rating::ActiveModel {
        user_rating_id: NotSet,
        user_account_id: Set(req.user_account_id),
        consultant_id: Set(req.consultant_id),
        meeting_at: Set(*meeting_date_time),
        charge_id: Set(req.charge_id.clone()),
        rating: NotSet,
        rated_at: NotSet,
    };
    let _ = active_model.insert(txn).await.map_err(|e| {
        error!("failed to insert user_rating (user_account_id: {}, consultant_id: {}, meeting_at: {}, charge_id: {}): {}", 
            req.user_account_id, req.consultant_id, meeting_date_time, req.charge_id, e);
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

async fn create_settlement(
    req: &consultation_req::Model,
    meeting_date_time: &DateTime<FixedOffset>,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    // let active_model = settlement::ActiveModel {
    //     settlement_id: NotSet,
    //     user_account_id: Set(req.user_account_id),
    //     consultant_id: Set(req.consultant_id),
    //     meeting_at: Set(*meeting_date_time),
    //     charge_id: Set(req.charge_id.clone()),
    //     settled: Set(false),
    //     stop_settlement: Set(false),
    //     expired_at: Set(req.latest_candidate_date_time),
    // };
    // let _ = active_model.insert(txn).await.map_err(|e| {
    //     error!("failed to insert settlement (user_account_id: {}, consultant_id: {}, meeting_at: {}, charge_id: {}): {}",
    //         req.user_account_id, req.consultant_id, meeting_date_time, req.charge_id, e);
    //     ErrRespStruct {
    //         err_resp: unexpected_err_resp(),
    //     }
    // })?;
    // Ok(())
    todo!()
}

fn validate_picked_candidate(picked_candidate: u8) -> Result<(), ErrResp> {
    if !(1..=3).contains(&picked_candidate) {
        error!("invalid candidate ({})", picked_candidate);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidCandidate as u32,
            }),
        ));
    }
    Ok(())
}

fn validate_user_checked_confirmation_items(
    user_checked: bool,
    user_account_id: i64,
) -> Result<(), ErrResp> {
    if !user_checked {
        error!(
            "user ({}) did not check confirmation items",
            user_account_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::UerDoesNotCheckConfirmationItems as u32,
            }),
        ));
    };
    Ok(())
}

async fn validate_identity_exists(
    account_id: i64,
    op: &impl ConsultationRequestAcceptanceOperation,
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

fn validate_consultation_req_for_acceptance(
    consultation_req: &ConsultationRequest,
    consultant_id: i64,
    current_date_time: &DateTime<FixedOffset>,
) -> Result<(), ErrResp> {
    if consultation_req.consultant_id != consultant_id {
        error!(
            "consultant_id ({}) does not match consultation_req.consultant_id ({})",
            consultant_id, consultation_req.consultant_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationReqFound as u32,
            }),
        ));
    }
    let criteria = *current_date_time
        + Duration::hours(*MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE as i64);
    if consultation_req.latest_candidate_date_time_in_jst <= criteria {
        error!(
            "latest candidate ({}) is not over criteria ({})",
            consultation_req.latest_candidate_date_time_in_jst, criteria
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationReqFound as u32,
            }),
        ));
    }
    Ok(())
}

async fn get_consultant_if_available(
    consultant_id: i64,
    op: &impl ConsultationRequestAcceptanceOperation,
) -> Result<UserAccount, ErrResp> {
    let consultant = op.get_consultant_if_available(consultant_id).await?;
    consultant.ok_or_else(|| {
        error!("consultant ({}) is not available", consultant_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultantIsNotAvailableOnConsultationAcceptance as u32,
            }),
        )
    })
}

async fn get_user_account_if_available(
    user_account_id: i64,
    op: &impl ConsultationRequestAcceptanceOperation,
) -> Result<UserAccount, ErrResp> {
    let user = op.get_user_account_if_available(user_account_id).await?;
    user.ok_or_else(|| {
        error!("user ({}) is not available", user_account_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::UserIsNotAvailableOnConsultationAcceptance as u32,
            }),
        )
    })
}
