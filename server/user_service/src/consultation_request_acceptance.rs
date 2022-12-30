// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, Datelike, Duration, FixedOffset, Timelike, Utc};
use common::smtp::{
    SmtpClient, INQUIRY_EMAIL_ADDRESS, SMTP_HOST, SMTP_PASSWORD, SMTP_PORT, SMTP_USERNAME,
    SYSTEM_EMAIL_ADDRESS,
};
use common::{smtp::SendMail, RespResult, JAPANESE_TIME_ZONE};
use common::{ApiError, ErrResp, ErrRespStruct, WEB_SITE_NAME};
use entity::prelude::ConsultationReq;
use entity::sea_orm::ActiveValue::NotSet;
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, QuerySelect, Set,
    TransactionError, TransactionTrait,
};
use entity::{consultant_rating, consultation, consultation_req, settlement, user_rating};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::err::{unexpected_err_resp, Code};
use crate::util::session::User;
use crate::util::{
    self, available_user_account::UserAccount, consultation_request::consultation_req_exists,
    consultation_request::ConsultationRequest,
    optional_env_var::MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE,
    validator::consultation_req_id_validator::validate_consultation_req_id_is_positive,
};

static CONSULTATION_REQ_ACCEPTANCE_MAIL_SUBJECT: Lazy<String> =
    Lazy::new(|| format!("[{}] 相談申し込み成立通知", WEB_SITE_NAME));

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

#[derive(Debug, Deserialize)]
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

    let consultation = op
        .accept_consultation_req(consultation_req_id, picked_candidate)
        .await?;

    let result = send_mail_to_user(
        req.consultation_req_id,
        &consultation,
        user.email_address.as_str(),
        &send_mail,
    )
    .await;
    // 相談受け付け処理（DBのトランザクション）は完了しているため、万が一通知メールが失敗しても処理自体はエラーとしない
    if result.is_err() {
        warn!(
            "failed to send email to user (consultation_req_id: {}, consultation: {:?}, email_address: {}, result: {:?})",
            req.consultation_req_id, consultation, user.email_address, result
        );
    }

    let result = send_mail_to_consultant(
        req.consultation_req_id,
        &consultation,
        consultant.email_address.as_str(),
        &send_mail,
    )
    .await;
    // 相談受け付け処理（DBのトランザクション）は完了しているため、万が一通知メールが失敗しても処理自体はエラーとしない
    if result.is_err() {
        warn!(
            "failed to send email to consultant (consultation_req_id: {}, consultation: {:?}, email_address: {}, result: {:?})",
            req.consultation_req_id, consultation, consultant.email_address, result
        );
    }

    Ok((StatusCode::OK, Json(ConsultationRequestAcceptanceResult {})))
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
    user_account_id: i64,
    consultant_id: i64,
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
                    create_consultant_rating(&req, &meeting_date_time, txn).await?;

                    delete_consultation_req_by_consultation_req_id(req.consultation_req_id, txn)
                        .await?;

                    Ok(Consultation {
                        user_account_id: req.user_account_id,
                        consultant_id: req.consultant_id,
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
    let active_model = settlement::ActiveModel {
        settlement_id: NotSet,
        user_account_id: Set(req.user_account_id),
        consultant_id: Set(req.consultant_id),
        meeting_at: Set(*meeting_date_time),
        charge_id: Set(req.charge_id.clone()),
        fee_per_hour_in_yen: Set(req.fee_per_hour_in_yen),
        platform_fee_rate_in_percentage: Set(req.platform_fee_rate_in_percentage.clone()),
        credit_facilities_expired_at: Set(req.credit_facilities_expired_at),
    };
    let _ = active_model.insert(txn).await.map_err(|e| {
        error!("failed to insert settlement (user_account_id: {}, consultant_id: {}, meeting_at: {}, req: {:?}): {}",
            req.user_account_id, req.consultant_id, meeting_date_time, req, e);
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

async fn create_consultant_rating(
    req: &consultation_req::Model,
    meeting_date_time: &DateTime<FixedOffset>,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    let active_model = consultant_rating::ActiveModel {
        consultant_rating_id: NotSet,
        user_account_id: Set(req.user_account_id),
        consultant_id: Set(req.consultant_id),
        meeting_at: Set(*meeting_date_time),
        charge_id: Set(req.charge_id.clone()),
        rating: NotSet,
        rated_at: NotSet,
    };
    let _ = active_model.insert(txn).await.map_err(|e| {
        error!("failed to insert consultant_rating (user_account_id: {}, consultant_id: {}, meeting_at: {}, charge_id: {}): {}", 
            req.user_account_id, req.consultant_id, meeting_date_time, req.charge_id, e);
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(())
}

async fn delete_consultation_req_by_consultation_req_id(
    consultation_req_id: i64,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    ConsultationReq::delete_by_id(consultation_req_id)
        .exec(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to delete consultantion_req (consultation_req_id: {}): {}",
                consultation_req_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    Ok(())
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

async fn send_mail_to_user(
    consultation_req_id: i64,
    consultation: &Consultation,
    email_address: &str,
    send_mail: &impl SendMail,
) -> Result<(), ErrResp> {
    let date_time =
        create_japanese_date_time_expression(&consultation.consultation_date_time_in_jst);
    let text = create_text_for_user(
        consultation_req_id,
        consultation.consultant_id,
        consultation.fee_per_hour_in_yen,
        date_time.as_str(),
    );
    send_mail
        .send_mail(
            email_address,
            SYSTEM_EMAIL_ADDRESS,
            CONSULTATION_REQ_ACCEPTANCE_MAIL_SUBJECT.as_str(),
            text.as_str(),
        )
        .await?;
    Ok(())
}

fn create_japanese_date_time_expression(date_time: &DateTime<FixedOffset>) -> String {
    let year = date_time.year();
    let month = date_time.month();
    let day = date_time.day();
    let hour = date_time.hour();
    format!("{}年 {}月 {}日 {}時00分", year, month, day, hour)
}

fn create_text_for_user(
    consultation_req_id: i64,
    consultant_id: i64,
    fee_per_hour_in_yen: i32,
    consultation_date_time: &str,
) -> String {
    // TODO: 文面の調整
    format!(
        r"相談申し込み（相談申し込み番号: {}）が成立しました。下記に成立した相談申し込みの詳細を記載いたします。

【相談相手】
  コンサルタントID: {}

【相談料金】
  {} 円

【相談開始日時】
  {}

【お問い合わせ先】
Email: {}",
        consultation_req_id,
        consultant_id,
        fee_per_hour_in_yen,
        consultation_date_time,
        INQUIRY_EMAIL_ADDRESS
    )
}

async fn send_mail_to_consultant(
    consultation_req_id: i64,
    consultation: &Consultation,
    email_address: &str,
    send_mail: &impl SendMail,
) -> Result<(), ErrResp> {
    let date_time =
        create_japanese_date_time_expression(&consultation.consultation_date_time_in_jst);
    let text = create_text_for_consultant(
        consultation_req_id,
        consultation.user_account_id,
        consultation.fee_per_hour_in_yen,
        date_time.as_str(),
    );
    send_mail
        .send_mail(
            email_address,
            SYSTEM_EMAIL_ADDRESS,
            CONSULTATION_REQ_ACCEPTANCE_MAIL_SUBJECT.as_str(),
            text.as_str(),
        )
        .await?;
    Ok(())
}

fn create_text_for_consultant(
    consultation_req_id: i64,
    user_account_id: i64,
    fee_per_hour_in_yen: i32,
    consultation_date_time: &str,
) -> String {
    // TODO: 文面の調整
    format!(
        r"相談申し込み（相談申し込み番号: {}）が成立しました。下記に成立した相談申し込みの詳細を記載いたします。

【相談申し込み者】
  ユーザーID: {}

【相談料金】
  {} 円

【相談開始日時】
  {}

【お問い合わせ先】
Email: {}",
        consultation_req_id,
        user_account_id,
        fee_per_hour_in_yen,
        consultation_date_time,
        INQUIRY_EMAIL_ADDRESS
    )
}

#[cfg(test)]
mod tests {
    use axum::async_trait;
    use chrono::{DateTime, FixedOffset};
    use common::{ErrResp, RespResult};
    use once_cell::sync::Lazy;

    use crate::util::{
        available_user_account::UserAccount, consultation_request::ConsultationRequest,
        tests::SendMailMock,
    };

    use super::{
        Consultation, ConsultationRequestAcceptanceOperation, ConsultationRequestAcceptanceParam,
        ConsultationRequestAcceptanceResult,
    };

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<ConsultationRequestAcceptanceResult>,
    }

    #[derive(Debug)]
    struct Input {
        user_account_id: i64,
        param: ConsultationRequestAcceptanceParam,
        current_date_time: DateTime<FixedOffset>,
        op: ConsultationRequestAcceptanceOperationMock,
        send_mail: SendMailMock,
    }

    #[derive(Debug)]
    struct ConsultationRequestAcceptanceOperationMock {}

    #[async_trait]
    impl ConsultationRequestAcceptanceOperation for ConsultationRequestAcceptanceOperationMock {
        async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
            todo!()
        }

        async fn find_consultation_req_by_consultation_req_id(
            &self,
            consultation_req_id: i64,
        ) -> Result<Option<ConsultationRequest>, ErrResp> {
            todo!()
        }

        async fn get_consultant_if_available(
            &self,
            consultant_id: i64,
        ) -> Result<Option<UserAccount>, ErrResp> {
            todo!()
        }

        async fn get_user_account_if_available(
            &self,
            user_account_id: i64,
        ) -> Result<Option<UserAccount>, ErrResp> {
            todo!()
        }

        async fn accept_consultation_req(
            &self,
            consultation_req_id: i64,
            picked_candidate: u8,
        ) -> Result<Consultation, ErrResp> {
            todo!()
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| vec![]);

    #[tokio::test]
    async fn handle_consultation_request_acceptance_tests() {
        for test_case in TEST_CASE_SET.iter() {}
    }
}
