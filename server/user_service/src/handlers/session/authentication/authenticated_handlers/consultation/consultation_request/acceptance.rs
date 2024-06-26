// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, Datelike, Duration, FixedOffset, Timelike, Utc};
use common::smtp::{SmtpClient, INQUIRY_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS};
use common::util::validator::uuid_validator::validate_uuid;
use common::util::Maintenance;
use common::{smtp::SendMail, RespResult, JAPANESE_TIME_ZONE};
use common::{ApiError, ErrResp, ErrRespStruct, WEB_SITE_NAME};
use entity::prelude::ConsultationReq;
use entity::sea_orm::ActiveValue::NotSet;
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    PaginatorTrait, QueryFilter, QuerySelect, Set, TransactionError, TransactionTrait,
};
use entity::{consultation, consultation_req, maintenance};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};
use uuid::Uuid;

use super::validate_consultation_req_id_is_positive;
use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::authentication::LENGTH_OF_MEETING_IN_MINUTE;
use crate::handlers::session::authentication::authenticated_handlers::authenticated_users::verified_user::VerifiedUser;
use crate::handlers::session::authentication::authenticated_handlers::consultation::{
    consultation_req_exists, ConsultationRequest,
};
use crate::handlers::session::authentication::user_operation::{FindUserInfoOperationImpl, UserInfo};
use crate::optional_env_var::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS;

static CONSULTATION_REQ_ACCEPTANCE_MAIL_SUBJECT: Lazy<String> =
    Lazy::new(|| format!("[{}] 相談申し込み成立通知", WEB_SITE_NAME));

pub(crate) const KEY_TO_BANK_CODE: &str = "BANK_CODE";
/// 収納代行として相談料金を預かる口座の銀行コード
static BANK_CODE: Lazy<String> = Lazy::new(|| {
    std::env::var(KEY_TO_BANK_CODE).unwrap_or_else(|_| {
        // UTのためにダミー用の初期値を設定しているだけ
        // 実行時には環境変数を指定して適切なものに入れ替える
        "xxxx".to_string()
    })
});

pub(crate) const KEY_TO_BANK_NAME: &str = "BANK_NAME";
/// 収納代行として相談料金を預かる口座の銀行名
static BANK_NAME: Lazy<String> = Lazy::new(|| {
    std::env::var(KEY_TO_BANK_NAME).unwrap_or_else(|_| {
        // UTのためにダミー用の初期値を設定しているだけ
        // 実行時には環境変数を指定して適切なものに入れ替える
        "XYZ銀行".to_string()
    })
});

pub(crate) const KEY_TO_BANK_BRANCH_CODE: &str = "BANK_BRANCH_CODE";
/// 収納代行として相談料金を預かる口座の銀行支店コード
static BANK_BRANCH_CODE: Lazy<String> = Lazy::new(|| {
    std::env::var(KEY_TO_BANK_BRANCH_CODE).unwrap_or_else(|_| {
        // UTのためにダミー用の初期値を設定しているだけ
        // 実行時には環境変数を指定して適切なものに入れ替える
        "yyy".to_string()
    })
});

pub(crate) const KEY_TO_BANK_BRANCH_NAME: &str = "BANK_BRANCH_NAME";
/// 収納代行として相談料金を預かる口座の銀行支店名
static BANK_BRANCH_NAME: Lazy<String> = Lazy::new(|| {
    std::env::var(KEY_TO_BANK_BRANCH_NAME).unwrap_or_else(|_| {
        // UTのためにダミー用の初期値を設定しているだけ
        // 実行時には環境変数を指定して適切なものに入れ替える
        "ABC支店".to_string()
    })
});

// 普通のみ利用する想定。普通以外が必要になったときに環境変数として作成し直す。
const BANK_ACCOUNT_TYPE: &str = "普通";

pub(crate) const KEY_TO_BANK_ACCOUNT_NUMBER: &str = "BANK_ACCOUNT_NUMBER";
/// 収納代行として相談料金を預かる口座の銀行口座番号
static BANK_ACCOUNT_NUMBER: Lazy<String> = Lazy::new(|| {
    std::env::var(KEY_TO_BANK_ACCOUNT_NUMBER).unwrap_or_else(|_| {
        // UTのためにダミー用の初期値を設定しているだけ
        // 実行時には環境変数を指定して適切なものに入れ替える
        "zzzzzzz".to_string()
    })
});

pub(crate) const KEY_TO_BANK_ACCOUNT_HOLDER_NAME: &str = "BANK_ACCOUNT_HOLDER_NAME";
/// 収納代行として相談料金を預かる口座の銀行口座名義名
static BANK_ACCOUNT_HOLDER_NAME: Lazy<String> = Lazy::new(|| {
    std::env::var(KEY_TO_BANK_ACCOUNT_HOLDER_NAME).unwrap_or_else(|_| {
        // UTのためにダミー用の初期値を設定しているだけ
        // 実行時には環境変数を指定して適切なものに入れ替える
        WEB_SITE_NAME.to_string()
    })
});

/// 収納代行の口座への入金期限（日）
///
/// [crate::optional_env_var::MIN_DURATION_BEFORE_CONSULTATION_IN_SECONDS]と[crate::optional_env_var::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS]を関連する値だが、
/// この値自体は、メールの文面に記載するだけでプログラム的に何か制約を課すわけでないため関係性を記載せず単独の定数として宣言している。
/// これはユーザーに対して入金の目安を指示しているのみで、この期限を過ぎたから入金しても無駄であるということを意味するわけではない。
/// 相談日時までに管理者が余裕を持って入金を確認したいため、ユーザーにそれまでに入金してほしいという意図で使っている。
const DEADLINE_OF_PAYMENT_IN_DAYS: usize = 3;

pub(crate) async fn post_consultation_request_acceptance(
    VerifiedUser { user_info }: VerifiedUser,
    State(smtp_client): State<SmtpClient>,
    State(pool): State<DatabaseConnection>,
    Json(param): Json<ConsultationRequestAcceptanceParam>,
) -> RespResult<ConsultationRequestAcceptanceResult> {
    let room_name = Uuid::new_v4().simple().to_string();
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = ConsultationRequestAcceptanceOperationImpl { pool };
    handle_consultation_request_acceptance(
        user_info.account_id,
        user_info.email_address,
        &param,
        &current_date_time,
        room_name,
        op,
        smtp_client,
    )
    .await
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct ConsultationRequestAcceptanceParam {
    consultation_req_id: i64,
    picked_candidate: u8,
    user_checked: bool,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct ConsultationRequestAcceptanceResult {}

async fn handle_consultation_request_acceptance(
    consultant_id: i64,
    consultant_email_address: String,
    param: &ConsultationRequestAcceptanceParam,
    current_date_time: &DateTime<FixedOffset>,
    room_name: String,
    op: impl ConsultationRequestAcceptanceOperation,
    send_mail: impl SendMail,
) -> RespResult<ConsultationRequestAcceptanceResult> {
    validate_uuid(room_name.as_str()).map_err(|e| {
        error!("failed to validate {}: {}", room_name, e);
        // peer_room_nameidは、ユーザーから渡されるものではなく、サーバで生成するものなので失敗はunexpected_err_resp
        unexpected_err_resp()
    })?;
    let picked_candidate = param.picked_candidate;
    validate_picked_candidate(picked_candidate)?;
    info!(
        "consultant (consultant id: {}) picked candidate: {}",
        consultant_id, picked_candidate
    );
    validate_user_checked_confirmation_items(param.user_checked, consultant_id)?;
    let consultation_req_id = param.consultation_req_id;
    validate_consultation_req_id_is_positive(consultation_req_id)?;

    let req = op
        .find_consultation_req_by_consultation_req_id(consultation_req_id)
        .await?;
    let req = consultation_req_exists(req, consultation_req_id)?;
    validate_consultation_req_for_acceptance(&req, consultant_id, current_date_time)?;

    // 操作者（コンサルタント）のアカウントが無効化されているかどうかは個々のURLを示すハンドラに来る前の共通箇所でチェックする
    // 従って、アカウントが無効化されているかどうかはユーザーのみ確認する
    let user = get_user_account_if_available(req.user_account_id, &op).await?;

    let meeting_date_time = select_meeting_date_time(
        &req.first_candidate_date_time_in_jst,
        &req.second_candidate_date_time_in_jst,
        &req.third_candidate_date_time_in_jst,
        picked_candidate,
    )?;

    ensure_there_is_enough_spare_time_before_meeting(meeting_date_time, *current_date_time)?;

    ensure_consultant_has_no_same_meeting_date_time(req.consultant_id, meeting_date_time, &op)
        .await?;
    ensure_user_has_no_same_meeting_date_time(req.user_account_id, meeting_date_time, &op).await?;

    ensure_meeting_date_time_does_not_overlap_maintenance(
        *current_date_time,
        meeting_date_time,
        &op,
    )
    .await?;

    let consultation = op
        .accept_consultation_req(
            consultation_req_id,
            meeting_date_time,
            room_name,
            *current_date_time,
            req.fee_per_hour_in_yen,
        )
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
        consultant_email_address.as_str(),
        &send_mail,
    )
    .await;
    // 相談受け付け処理（DBのトランザクション）は完了しているため、万が一通知メールが失敗しても処理自体はエラーとしない
    if result.is_err() {
        warn!(
            "failed to send email to consultant (consultation_req_id: {}, consultation: {:?}, email_address: {}, result: {:?})",
            req.consultation_req_id, consultation, consultant_email_address, result
        );
    }

    Ok((StatusCode::OK, Json(ConsultationRequestAcceptanceResult {})))
}

#[async_trait]
trait ConsultationRequestAcceptanceOperation {
    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp>;

    async fn get_user_account_if_available(
        &self,
        user_account_id: i64,
    ) -> Result<Option<UserInfo>, ErrResp>;

    async fn count_user_side_consultation_by_user_account_id(
        &self,
        user_account_id: i64,
        meeting_date_time: DateTime<FixedOffset>,
    ) -> Result<u64, ErrResp>;

    async fn count_consultant_side_consultation_by_user_account_id(
        &self,
        user_account_id: i64,
        meeting_date_time: DateTime<FixedOffset>,
    ) -> Result<u64, ErrResp>;

    async fn count_consultant_side_consultation_by_consultant_id(
        &self,
        consultant_id: i64,
        meeting_date_time: DateTime<FixedOffset>,
    ) -> Result<u64, ErrResp>;

    async fn count_user_side_consultation_by_consultant_id(
        &self,
        consultant_id: i64,
        meeting_date_time: DateTime<FixedOffset>,
    ) -> Result<u64, ErrResp>;

    async fn filter_maintenance_by_maintenance_end_at(
        &self,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<Maintenance>, ErrResp>;

    async fn accept_consultation_req(
        &self,
        consultation_req_id: i64,
        meeting_date_time: DateTime<FixedOffset>,
        room_name: String,
        current_date_time: DateTime<FixedOffset>,
        fee_per_hour_in_yen: i32,
    ) -> Result<AcceptedConsultation, ErrResp>;
}

#[derive(Clone, Debug)]
struct AcceptedConsultation {
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
    async fn find_consultation_req_by_consultation_req_id(
        &self,
        consultation_req_id: i64,
    ) -> Result<Option<ConsultationRequest>, ErrResp> {
        super::super::find_consultation_req_by_consultation_req_id(&self.pool, consultation_req_id)
            .await
    }

    async fn get_user_account_if_available(
        &self,
        user_account_id: i64,
    ) -> Result<Option<UserInfo>, ErrResp> {
        let op = FindUserInfoOperationImpl::new(&self.pool);
        super::super::find_user_info_if_available(user_account_id, &op).await
    }

    async fn count_user_side_consultation_by_user_account_id(
        &self,
        user_account_id: i64,
        meeting_date_time: DateTime<FixedOffset>,
    ) -> Result<u64, ErrResp> {
        let cnt = count_consultation_filtered_by_user_account_id_and_meeting_at(
            user_account_id,
            meeting_date_time,
            &self.pool,
        )
        .await
        .map_err(|e| {
            error!(
                "failed to count user side consultation (user_account_id: {}, meeting_date_time: {}): {}",
                user_account_id, meeting_date_time, e
            );
            unexpected_err_resp()
        })?;
        Ok(cnt)
    }

    async fn count_consultant_side_consultation_by_user_account_id(
        &self,
        user_account_id: i64,
        meeting_date_time: DateTime<FixedOffset>,
    ) -> Result<u64, ErrResp> {
        let cnt = count_consultation_filtered_by_consultant_id_and_meeting_at(
            user_account_id,
            meeting_date_time,
            &self.pool,
        )
        .await
        .map_err(|e| {
            error!(
                "failed to count consultant side consultation (user_account_id: {}, meeting_date_time: {}): {}",
                user_account_id, meeting_date_time, e
            );
            unexpected_err_resp()
        })?;
        Ok(cnt)
    }

    async fn count_consultant_side_consultation_by_consultant_id(
        &self,
        consultant_id: i64,
        meeting_date_time: DateTime<FixedOffset>,
    ) -> Result<u64, ErrResp> {
        let cnt = count_consultation_filtered_by_consultant_id_and_meeting_at(
            consultant_id,
            meeting_date_time,
            &self.pool,
        )
        .await
        .map_err(|e| {
            error!(
                "failed to count consultant side consultation (consultant_id: {}, meeting_date_time: {}): {}",
                consultant_id, meeting_date_time, e
            );
            unexpected_err_resp()
        })?;
        Ok(cnt)
    }

    async fn count_user_side_consultation_by_consultant_id(
        &self,
        consultant_id: i64,
        meeting_date_time: DateTime<FixedOffset>,
    ) -> Result<u64, ErrResp> {
        let cnt = count_consultation_filtered_by_user_account_id_and_meeting_at(
            consultant_id,
            meeting_date_time,
            &self.pool,
        )
        .await
        .map_err(|e| {
            error!(
                "failed to count user side consultation (consultant_id: {}, meeting_date_time: {}): {}",
                consultant_id, meeting_date_time, e
            );
            unexpected_err_resp()
        })?;
        Ok(cnt)
    }

    async fn filter_maintenance_by_maintenance_end_at(
        &self,
        current_date_time: DateTime<FixedOffset>,
    ) -> Result<Vec<Maintenance>, ErrResp> {
        let maintenances = maintenance::Entity::find()
            .filter(maintenance::Column::MaintenanceEndAt.gte(current_date_time))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter maintenance (current_date_time: {}): {}",
                    current_date_time, e
                );
                unexpected_err_resp()
            })?;
        Ok(maintenances
            .into_iter()
            .map(|m| Maintenance {
                maintenance_id: m.maintenance_id,
                maintenance_start_at_in_jst: m
                    .maintenance_start_at
                    .with_timezone(&*JAPANESE_TIME_ZONE),
                maintenance_end_at_in_jst: m.maintenance_end_at.with_timezone(&*JAPANESE_TIME_ZONE),
            })
            .collect::<Vec<Maintenance>>())
    }

    async fn accept_consultation_req(
        &self,
        consultation_req_id: i64,
        meeting_date_time: DateTime<FixedOffset>,
        room_name: String,
        current_date_time: DateTime<FixedOffset>,
        fee_per_hour_in_yen: i32,
    ) -> Result<AcceptedConsultation, ErrResp> {
        let consultation = self
            .pool
            .transaction::<_, AcceptedConsultation, ErrRespStruct>(|txn| {
                Box::pin(async move {
                    let req =
                        get_consultation_req_with_exclusive_lock(consultation_req_id, txn).await?;

                    let c = create_consultation(&req, &meeting_date_time, room_name.as_str(), txn)
                        .await?;
                    create_awaiting_payment(
                        c.consultation_id,
                        c.user_account_id,
                        c.consultant_id,
                        c.meeting_at,
                        current_date_time,
                        fee_per_hour_in_yen,
                        txn,
                    )
                    .await?;

                    delete_consultation_req_by_consultation_req_id(req.consultation_req_id, txn)
                        .await?;

                    Ok(AcceptedConsultation {
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

async fn count_consultation_filtered_by_consultant_id_and_meeting_at(
    consultant_id: i64,
    meeting_date_time: DateTime<FixedOffset>,
    pool: &DatabaseConnection,
) -> Result<u64, DbErr> {
    let cnt = consultation::Entity::find()
        .filter(consultation::Column::MeetingAt.eq(meeting_date_time))
        .filter(consultation::Column::ConsultantId.eq(consultant_id))
        .count(pool)
        .await?;
    Ok(cnt)
}

async fn count_consultation_filtered_by_user_account_id_and_meeting_at(
    user_account_id: i64,
    meeting_date_time: DateTime<FixedOffset>,
    pool: &DatabaseConnection,
) -> Result<u64, DbErr> {
    let cnt = consultation::Entity::find()
        .filter(consultation::Column::MeetingAt.eq(meeting_date_time))
        .filter(consultation::Column::UserAccountId.eq(user_account_id))
        .count(pool)
        .await?;
    Ok(cnt)
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

async fn create_consultation(
    req: &consultation_req::Model,
    meeting_date_time: &DateTime<FixedOffset>,
    room_name: &str,
    txn: &DatabaseTransaction,
) -> Result<entity::consultation::Model, ErrRespStruct> {
    let active_model = consultation::ActiveModel {
        consultation_id: NotSet,
        user_account_id: Set(req.user_account_id),
        consultant_id: Set(req.consultant_id),
        meeting_at: Set(*meeting_date_time),
        room_name: Set(room_name.to_string()),
        user_account_entered_at: NotSet,
        consultant_entered_at: NotSet,
    };
    let result = active_model.insert(txn).await.map_err(|e| {
        error!("failed to insert consultation (user_account_id: {}, consultant_id: {}, meeting_at: {}, room_name: {}, charge_id: {}): {}", 
            req.user_account_id, req.consultant_id, meeting_date_time, room_name, "charge_id", e);
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })?;
    Ok(result)
}

async fn create_awaiting_payment(
    consultation_id: i64,
    user_account_id: i64,
    consultant_id: i64,
    meeting_at: DateTime<FixedOffset>,
    current_date_time: DateTime<FixedOffset>,
    fee_per_hour_in_yen: i32,
    txn: &DatabaseTransaction,
) -> Result<(), ErrRespStruct> {
    let active_model = entity::awaiting_payment::ActiveModel {
        consultation_id: Set(consultation_id),
        user_account_id: Set(user_account_id),
        consultant_id: Set(consultant_id),
        meeting_at: Set(meeting_at),
        fee_per_hour_in_yen: Set(fee_per_hour_in_yen),
        created_at: Set(current_date_time),
    };
    let _ = active_model.insert(txn).await.map_err(|e|{
        error!("failed to insert awaiting_payment (consultation_id: {}, current_date_time: {}, fee_per_hour_in_yen: {}): {}", 
            consultation_id, current_date_time, fee_per_hour_in_yen, e);
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
                code: Code::UserDoesNotCheckConfirmationItems as u32,
            }),
        ));
    };
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
        + Duration::seconds(*MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64);
    if consultation_req.latest_candidate_date_time_in_jst <= criteria {
        error!(
            "there is no enough spare time before latest candidate (latest candidate: {}, criteria: {})",
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

async fn get_user_account_if_available(
    user_account_id: i64,
    op: &impl ConsultationRequestAcceptanceOperation,
) -> Result<UserInfo, ErrResp> {
    let user = op.get_user_account_if_available(user_account_id).await?;
    user.ok_or_else(|| {
        error!("user ({}) is not available", user_account_id);
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::TheOtherPersonAccountIsNotAvailable as u32,
            }),
        )
    })
}

fn select_meeting_date_time(
    first_candidate_date_time: &DateTime<FixedOffset>,
    second_candidate_date_time: &DateTime<FixedOffset>,
    third_candidate_date_time: &DateTime<FixedOffset>,
    picked_candidate: u8,
) -> Result<DateTime<FixedOffset>, ErrResp> {
    if picked_candidate == 1 {
        Ok(*first_candidate_date_time)
    } else if picked_candidate == 2 {
        Ok(*second_candidate_date_time)
    } else if picked_candidate == 3 {
        Ok(*third_candidate_date_time)
    } else {
        error!("invalid picked_candidate ({})", picked_candidate);
        Err(unexpected_err_resp())
    }
}

fn ensure_there_is_enough_spare_time_before_meeting(
    meeting_date_time: DateTime<FixedOffset>,
    current_date_time: DateTime<FixedOffset>,
) -> Result<(), ErrResp> {
    let criteria = current_date_time
        + Duration::seconds(*MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64);
    if meeting_date_time <= criteria {
        error!(
            "there is no enough spare time before meeting (meeting date time: {}, criteria: {})",
            meeting_date_time, criteria
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoEnoughSpareTimeBeforeMeeting as u32,
            }),
        ));
    }
    Ok(())
}

async fn ensure_consultant_has_no_same_meeting_date_time(
    consultant_id: i64,
    meeting_date_time: DateTime<FixedOffset>,
    op: &impl ConsultationRequestAcceptanceOperation,
) -> Result<(), ErrResp> {
    // コンサルタントが、相談相手として同じ日時の相談を持っているかどうか
    let cnt = op
        .count_consultant_side_consultation_by_consultant_id(consultant_id, meeting_date_time)
        .await?;
    if cnt != 0 {
        error!(
            "same meeting date time (as consultant) found (cnt: {}, consultant_id: {}, meeting_date_time: {})",
            cnt, consultant_id, meeting_date_time
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultantHasSameMeetingDateTime as u32,
            }),
        ));
    }
    // コンサルタントが、相談申し込み者として同じ日時の相談を持っているかどうか
    let cnt = op
        .count_user_side_consultation_by_consultant_id(consultant_id, meeting_date_time)
        .await?;
    if cnt != 0 {
        error!(
            "same meeting date time  (as user)  found (cnt: {}, consultant_id: {}, meeting_date_time: {})",
            cnt, consultant_id, meeting_date_time
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultantHasSameMeetingDateTime as u32,
            }),
        ));
    }
    Ok(())
}

async fn ensure_user_has_no_same_meeting_date_time(
    user_account_id: i64,
    meeting_date_time: DateTime<FixedOffset>,
    op: &impl ConsultationRequestAcceptanceOperation,
) -> Result<(), ErrResp> {
    // ユーザーが、相談申し込み者として同じ日時の相談を持っているかどうか
    let cnt = op
        .count_user_side_consultation_by_user_account_id(user_account_id, meeting_date_time)
        .await?;
    if cnt != 0 {
        error!(
            "same meeting date time (as user) found (cnt: {}, user_account_id: {}, meeting_date_time: {})",
            cnt, user_account_id, meeting_date_time
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::UserHasSameMeetingDateTime as u32,
            }),
        ));
    }
    // ユーザーが、相談相手として同じ日時の相談を持っているかどうか
    let cnt = op
        .count_consultant_side_consultation_by_user_account_id(user_account_id, meeting_date_time)
        .await?;
    if cnt != 0 {
        error!(
            "same meeting date time (as consultant) found (cnt: {}, user_account_id: {}, meeting_date_time: {})",
            cnt, user_account_id, meeting_date_time
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::UserHasSameMeetingDateTime as u32,
            }),
        ));
    }
    Ok(())
}

async fn ensure_meeting_date_time_does_not_overlap_maintenance(
    current_date_time: DateTime<FixedOffset>,
    meeting_date_time: DateTime<FixedOffset>,
    op: &impl ConsultationRequestAcceptanceOperation,
) -> Result<(), ErrResp> {
    let results = op
        .filter_maintenance_by_maintenance_end_at(current_date_time)
        .await?;
    let meeting_start_time = meeting_date_time;
    let meeting_end_time =
        meeting_date_time + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64);
    for result in results {
        // ２つの時間帯が重なる条件（重ならない条件をド・モルガンの法則で反転）
        // 参考: https://yucatio.hatenablog.com/entry/2018/08/16/175914
        if result.maintenance_end_at_in_jst > meeting_start_time
            && meeting_end_time > result.maintenance_start_at_in_jst
        {
            error!(
                "meeting date time ({}) overlaps maintenance ({:?}) (current date time: {})",
                meeting_date_time, result, current_date_time
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: Code::MeetingDateTimeOverlapsMaintenance as u32,
                }),
            ));
        }
    }
    Ok(())
}

async fn send_mail_to_user(
    consultation_req_id: i64,
    consultation: &AcceptedConsultation,
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
            SYSTEM_EMAIL_ADDRESS.as_str(),
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
    format!(
        r"相談申し込み（相談申し込み番号: {}）が成立しました。ログイン後、スケジュールに相談室が作られていることをご確認下さい。下記に成立した相談申し込みの詳細を記載いたします。

相談相手
  コンサルタントID: {}

相談料金
  {} 円

相談開始日時
  {}

相談開始日時の{}日前までに下記の口座に入金をお願いいたします（入金の際にかかる振込手数料はユーザーのご負担となります）入金が確認できない場合、相談を行うことは出来ないため、期日に余裕を持って入金をするようお願いいたします。

  銀行名: {} (銀行コード: {})
  支店名: {} (支店コード: {})
  口座種別: {}
  口座番号: {}
  口座名義人: {}

入金の際、依頼人名は姓名、空白、相談日時（6桁の数字）として下さい（例 姓名がタナカ　タロウ、相談日時が9月29日19時のとき、依頼人名は「タナカ　タロウ　０９２９１９」となります（※））お名前に加えて相談日時を確認できない場合、正しく入金確認出来ないことがありますので必ず前述の通りご入力をお願いします。

（※）依頼人名に入力可能な文字数制限に達して全て入力出来ない場合、可能なところまで入力して振り込みを行って下さい。

【お問い合わせ先】
Email: {}",
        consultation_req_id,
        consultant_id,
        fee_per_hour_in_yen,
        consultation_date_time,
        DEADLINE_OF_PAYMENT_IN_DAYS,
        *BANK_NAME,
        *BANK_CODE,
        *BANK_BRANCH_NAME,
        *BANK_BRANCH_CODE,
        BANK_ACCOUNT_TYPE,
        *BANK_ACCOUNT_NUMBER,
        *BANK_ACCOUNT_HOLDER_NAME,
        INQUIRY_EMAIL_ADDRESS.as_str()
    )
}

async fn send_mail_to_consultant(
    consultation_req_id: i64,
    consultation: &AcceptedConsultation,
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
            SYSTEM_EMAIL_ADDRESS.as_str(),
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
    format!(
        r"相談申し込み（相談申し込み番号: {}）が成立しました。ログイン後、スケジュールに相談室が作られていることをご確認下さい。下記に成立した相談申し込みの詳細を記載いたします。

相談申し込み者
  ユーザーID: {}

相談料金
  {} 円

相談開始日時
  {}

【お問い合わせ先】
Email: {}",
        consultation_req_id,
        user_account_id,
        fee_per_hour_in_yen,
        consultation_date_time,
        INQUIRY_EMAIL_ADDRESS.as_str()
    )
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;

    use super::*;

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<ConsultationRequestAcceptanceResult>,
    }

    #[derive(Debug)]
    struct Input {
        user_account_id: i64,
        email_address: String,
        param: ConsultationRequestAcceptanceParam,
        current_date_time: DateTime<FixedOffset>,
        room_name: String,
        op: ConsultationRequestAcceptanceOperationMock,
        send_mail: SendMailMock,
    }

    #[derive(Clone, Debug)]
    struct ConsultationRequestAcceptanceOperationMock {
        consultation_req: ConsultationRequest,
        user: Option<UserInfo>,
        meeting_date_time: DateTime<FixedOffset>,
        cnt_user_side_consultation_by_user_account_id: u64,
        cnt_consultant_side_consultation_by_user_account_id: u64,
        cnt_consultant_side_consultation_by_consultant_id: u64,
        cnt_user_side_consultation_by_consultant_id: u64,
        current_date_time: DateTime<FixedOffset>,
        maintenance_info: Vec<Maintenance>,
        consultation: AcceptedConsultation,
        room_name: String,
    }

    #[async_trait]
    impl ConsultationRequestAcceptanceOperation for ConsultationRequestAcceptanceOperationMock {
        async fn find_consultation_req_by_consultation_req_id(
            &self,
            consultation_req_id: i64,
        ) -> Result<Option<ConsultationRequest>, ErrResp> {
            if self.consultation_req.consultation_req_id != consultation_req_id {
                return Ok(None);
            }
            Ok(Some(self.consultation_req.clone()))
        }

        async fn get_user_account_if_available(
            &self,
            user_account_id: i64,
        ) -> Result<Option<UserInfo>, ErrResp> {
            assert_eq!(self.consultation_req.user_account_id, user_account_id);
            Ok(self.user.clone())
        }

        async fn count_user_side_consultation_by_user_account_id(
            &self,
            user_account_id: i64,
            meeting_date_time: DateTime<FixedOffset>,
        ) -> Result<u64, ErrResp> {
            assert_eq!(self.consultation_req.user_account_id, user_account_id);
            assert_eq!(self.meeting_date_time, meeting_date_time);
            Ok(self.cnt_user_side_consultation_by_user_account_id)
        }

        async fn count_consultant_side_consultation_by_user_account_id(
            &self,
            user_account_id: i64,
            meeting_date_time: DateTime<FixedOffset>,
        ) -> Result<u64, ErrResp> {
            assert_eq!(self.consultation_req.user_account_id, user_account_id);
            assert_eq!(self.meeting_date_time, meeting_date_time);
            Ok(self.cnt_consultant_side_consultation_by_user_account_id)
        }

        async fn count_consultant_side_consultation_by_consultant_id(
            &self,
            consultant_id: i64,
            meeting_date_time: DateTime<FixedOffset>,
        ) -> Result<u64, ErrResp> {
            assert_eq!(self.consultation_req.consultant_id, consultant_id);
            assert_eq!(self.meeting_date_time, meeting_date_time);
            Ok(self.cnt_consultant_side_consultation_by_consultant_id)
        }

        async fn count_user_side_consultation_by_consultant_id(
            &self,
            consultant_id: i64,
            meeting_date_time: DateTime<FixedOffset>,
        ) -> Result<u64, ErrResp> {
            assert_eq!(self.consultation_req.consultant_id, consultant_id);
            assert_eq!(self.meeting_date_time, meeting_date_time);
            Ok(self.cnt_user_side_consultation_by_consultant_id)
        }

        async fn filter_maintenance_by_maintenance_end_at(
            &self,
            current_date_time: DateTime<FixedOffset>,
        ) -> Result<Vec<Maintenance>, ErrResp> {
            assert_eq!(self.current_date_time, current_date_time);
            Ok(self.maintenance_info.clone())
        }

        async fn accept_consultation_req(
            &self,
            consultation_req_id: i64,
            meeting_date_time: DateTime<FixedOffset>,
            room_name: String,
            current_date_time: DateTime<FixedOffset>,
            fee_per_hour_in_yen: i32,
        ) -> Result<AcceptedConsultation, ErrResp> {
            assert_eq!(
                self.consultation_req.consultation_req_id,
                consultation_req_id
            );
            assert_eq!(self.meeting_date_time, meeting_date_time);
            assert_eq!(self.room_name, room_name);
            assert_eq!(self.current_date_time, current_date_time);
            assert_eq!(
                self.consultation_req.fee_per_hour_in_yen,
                fee_per_hour_in_yen
            );
            Ok(self.consultation.clone())
        }
    }

    #[derive(Clone, Debug)]
    struct SendMailMock {
        fail: bool,
    }

    #[async_trait]
    impl SendMail for SendMailMock {
        async fn send_mail(
            &self,
            _to: &str,
            from: &str,
            subject: &str,
            _text: &str,
        ) -> Result<(), ErrResp> {
            assert_eq!(from, SYSTEM_EMAIL_ADDRESS.as_str());
            assert_eq!(subject, *CONSULTATION_REQ_ACCEPTANCE_MAIL_SUBJECT);
            if self.fail {
                return Err(unexpected_err_resp());
            }
            Ok(())
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        let user_account_id_of_consultant = 6895;
        let current_date_time = JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2022, 12, 22, 23, 32, 21)
            .unwrap();
        let consultation_req_id = 431;
        let picked_candidate = 1;
        let user_checked = true;
        let user_account_id = 53;
        let fee_per_hour_in_yen = 4500;
        let consultant_email_address = "test0@test.com";
        let user_email_address = "test1@test.com";
        let send_mail = SendMailMock { fail: false };
        let room_name = "ce0cda1b7a934b3ea7a12001b56cf4e4";
        vec![
            TestCase {
                name: "success case (first choise is picked)".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Ok((StatusCode::OK, Json(ConsultationRequestAcceptanceResult {}))),
            },
            TestCase {
                name: "success case (second choise is picked)".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate: 2,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Ok((StatusCode::OK, Json(ConsultationRequestAcceptanceResult {}))),
            },
            TestCase {
                name: "success case (third choise is picked)".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate: 3,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Ok((StatusCode::OK, Json(ConsultationRequestAcceptanceResult {}))),
            },
            TestCase {
                name: "success case (ignore send mail failed)".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: SendMailMock { fail: true },
                },
                expected: Ok((StatusCode::OK, Json(ConsultationRequestAcceptanceResult {}))),
            },
            TestCase {
                name: "success case (no maintenance overlapped case 1)".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![Maintenance {
                            maintenance_id: 1,
                            maintenance_start_at_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 21, 0, 0)
                                .unwrap(),
                            maintenance_end_at_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 22, 0, 0)
                                .unwrap(),
                        }],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Ok((StatusCode::OK, Json(ConsultationRequestAcceptanceResult {}))),
            },
            TestCase {
                name: "success case (no maintenance overlapped case 2)".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 22, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 22, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![Maintenance {
                            maintenance_id: 1,
                            maintenance_start_at_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 21, 0, 0)
                                .unwrap(),
                            maintenance_end_at_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 22, 0, 0)
                                .unwrap(),
                        }],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 22, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Ok((StatusCode::OK, Json(ConsultationRequestAcceptanceResult {}))),
            },
            TestCase {
                name: "success case (no maintenance overlapped case 3)".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 22, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 22, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![Maintenance {
                            maintenance_id: 1,
                            maintenance_start_at_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 0, 0, 0)
                                .unwrap(),
                            maintenance_end_at_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 4, 0, 0)
                                .unwrap(),
                        }],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 22, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Ok((StatusCode::OK, Json(ConsultationRequestAcceptanceResult {}))),
            },
            TestCase {
                name: "success case (no maintenance overlapped case 4)".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![Maintenance {
                            maintenance_id: 1,
                            maintenance_start_at_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 0, 0, 0)
                                .unwrap(),
                            maintenance_end_at_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 4, 0, 0)
                                .unwrap(),
                        }],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Ok((StatusCode::OK, Json(ConsultationRequestAcceptanceResult {}))),
            },
            TestCase {
                name:
                    "success case (pick first candidate, there is barely spare time before meeting)"
                        .to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time: JAPANESE_TIME_ZONE
                        .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                        .unwrap()
                        - Duration::seconds(
                            *MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                        )
                        - Duration::seconds(1),
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap()
                            - Duration::seconds(
                                *MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                            )
                            - Duration::seconds(1),
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Ok((StatusCode::OK, Json(ConsultationRequestAcceptanceResult {}))),
            },
            TestCase {
                name: "invalid candidate case 1".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate: 0,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidCandidate as u32,
                    }),
                )),
            },
            TestCase {
                name: "invalid candidate case 2".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate: 4,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidCandidate as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail UserDoesNotCheckConfirmationItems".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked: false,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::UserDoesNotCheckConfirmationItems as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NonPositiveConsultationReqId case 1".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id: 0,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id: 0,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NonPositiveConsultationReqId as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NonPositiveConsultationReqId case 2".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id: -1,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id: -1,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NonPositiveConsultationReqId as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoConsultationReqFound case 1".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id: consultation_req_id + 1,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoConsultationReqFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoConsultationReqFound case 2".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant + 1,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoConsultationReqFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoConsultationReqFound case 3".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time: JAPANESE_TIME_ZONE
                        .with_ymd_and_hms(2023, 1, 5, 7, 0, 0)
                        .unwrap(),
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 7, 0, 0)
                                .unwrap()
                                + Duration::seconds(
                                    *MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                                ),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 4, 23, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 3, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 7, 0, 0)
                                .unwrap()
                                + Duration::seconds(
                                    *MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                                ),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 7, 0, 0)
                                .unwrap()
                                + Duration::seconds(
                                    *MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                                ),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoConsultationReqFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoConsultationReqFound case 4".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time: JAPANESE_TIME_ZONE
                        .with_ymd_and_hms(2023, 1, 5, 7, 0, 1)
                        .unwrap(),
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 7, 0, 0)
                                .unwrap()
                                + Duration::seconds(
                                    *MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                                ),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 4, 23, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 3, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 7, 0, 0)
                                .unwrap()
                                + Duration::seconds(
                                    *MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                                ),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 7, 0, 0)
                                .unwrap()
                                + Duration::seconds(
                                    *MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64,
                                ),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoConsultationReqFound as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail TheOtherPersonAccountIsNotAvailable".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 6, 15, 0, 0)
                                .unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 7, 7, 0, 0)
                                .unwrap(),
                        },
                        user: None,
                        meeting_date_time: JAPANESE_TIME_ZONE
                            .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                            .unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE
                                .with_ymd_and_hms(2023, 1, 5, 23, 0, 0)
                                .unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::TheOtherPersonAccountIsNotAvailable as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail UserHasSameMeetingDateTime (user has already other meeting as user at the time)".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 6, 15, 0, 0).unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 1,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::UserHasSameMeetingDateTime as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail UserHasSameMeetingDateTime (user has already other meeting as consultant at the time)".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 6, 15, 0, 0).unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 1,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::UserHasSameMeetingDateTime as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail ConsultantHasSameMeetingDateTime (consultant has already other meeting as consultant at the time)".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 6, 15, 0, 0).unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 1,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::ConsultantHasSameMeetingDateTime as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail ConsultantHasSameMeetingDateTime (consultant has already other meeting as user at the time)".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 6, 15, 0, 0).unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 1,
                        current_date_time,
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::ConsultantHasSameMeetingDateTime as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail MeetingDateTimeOverlapsMaintenance case 1".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 6, 15, 0, 0).unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![Maintenance {
                            maintenance_id: 1,
                            maintenance_start_at_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 22, 30, 0).unwrap(),
                            maintenance_end_at_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 30, 0).unwrap(),
                        }],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::MeetingDateTimeOverlapsMaintenance as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail MeetingDateTimeOverlapsMaintenance case 2".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 0, 0).unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 6, 15, 0, 0).unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 0, 0).unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![Maintenance {
                            maintenance_id: 1,
                            maintenance_start_at_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 18, 0, 0).unwrap(),
                            maintenance_end_at_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 21, 0, 0).unwrap(),
                        }],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 0, 0).unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::MeetingDateTimeOverlapsMaintenance as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail MeetingDateTimeOverlapsMaintenance case 3".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 0, 0).unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 6, 15, 0, 0).unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 0, 0).unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![Maintenance {
                            maintenance_id: 1,
                            maintenance_start_at_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 30, 0).unwrap(),
                            maintenance_end_at_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 20, 30, 0).unwrap(),
                        }],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 0, 0).unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::MeetingDateTimeOverlapsMaintenance as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail MeetingDateTimeOverlapsMaintenance case 4".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 0, 0).unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 6, 15, 0, 0).unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 0, 0).unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![Maintenance {
                            maintenance_id: 1,
                            maintenance_start_at_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 15, 0).unwrap(),
                            maintenance_end_at_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 45, 0).unwrap(),
                        }],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 0, 0).unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::MeetingDateTimeOverlapsMaintenance as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail MeetingDateTimeOverlapsMaintenance case 5".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time,
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 0, 0).unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 6, 15, 0, 0).unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 0, 0).unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time,
                        maintenance_info: vec![Maintenance {
                            maintenance_id: 1,
                            maintenance_start_at_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 0, 0).unwrap(),
                            maintenance_end_at_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 20, 0, 0).unwrap(),
                        }],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 19, 0, 0).unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail: send_mail.clone(),
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::MeetingDateTimeOverlapsMaintenance as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoEnoughSpareTimeBeforeMeeting".to_string(),
                input: Input {
                    user_account_id: user_account_id_of_consultant,
                    email_address: consultant_email_address.to_string(),
                    param: ConsultationRequestAcceptanceParam {
                        consultation_req_id,
                        picked_candidate,
                        user_checked,
                    },
                    current_date_time: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap() - Duration::seconds(*MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64),
                    room_name: room_name.to_string(),
                    op: ConsultationRequestAcceptanceOperationMock {
                        consultation_req: ConsultationRequest {
                            consultation_req_id,
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            first_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                            second_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 6, 15, 0, 0).unwrap(),
                            third_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                            latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 7, 7, 0, 0).unwrap(),
                        },
                        user: Some(UserInfo {
                            account_id: user_account_id,
                            email_address: user_email_address.to_string(),
                            mfa_enabled_at: None,
                            disabled_at: None,
                        }),
                        meeting_date_time: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                        cnt_user_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_user_account_id: 0,
                        cnt_consultant_side_consultation_by_consultant_id: 0,
                        cnt_user_side_consultation_by_consultant_id: 0,
                        current_date_time: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap() - Duration::seconds(*MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS as i64),
                        maintenance_info: vec![],
                        consultation: AcceptedConsultation {
                            user_account_id,
                            consultant_id: user_account_id_of_consultant,
                            fee_per_hour_in_yen,
                            consultation_date_time_in_jst: JAPANESE_TIME_ZONE.with_ymd_and_hms(2023, 1, 5, 23, 0, 0).unwrap(),
                        },
                        room_name: room_name.to_string(),
                    },
                    send_mail,
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoEnoughSpareTimeBeforeMeeting as u32,
                    }),
                )),
            },
        ]
    });

    #[tokio::test]
    async fn handle_consultation_request_acceptance_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let account_id = test_case.input.user_account_id;
            let email_address = test_case.input.email_address.clone();
            let param = test_case.input.param.clone();
            let current_date_time = test_case.input.current_date_time;
            let room_name = test_case.input.room_name.clone();
            let op = test_case.input.op.clone();
            let smtp_client = test_case.input.send_mail.clone();

            let result = handle_consultation_request_acceptance(
                account_id,
                email_address,
                &param,
                &current_date_time,
                room_name,
                op,
                smtp_client,
            )
            .await;

            let message = format!("test case \"{}\" failed", test_case.name.clone());
            if test_case.expected.is_ok() {
                let resp = result.expect("failed to get Ok");
                let expected = test_case.expected.as_ref().expect("failed to get Ok");
                assert_eq!(expected.0, resp.0, "{}", message);
                assert_eq!(expected.1 .0, resp.1 .0, "{}", message);
            } else {
                let resp = result.expect_err("failed to get Err");
                let expected = test_case.expected.as_ref().expect_err("failed to get Err");
                assert_eq!(expected.0, resp.0, "{}", message);
                assert_eq!(expected.1 .0, resp.1 .0, "{}", message);
            }
        }
    }

    #[test]
    fn test_create_text_for_user() {
        let consultation_req_id = 1312;
        let consultant_id = 53;
        let fee_per_hour_in_yen = 5000;
        let consultation_date_time = "2022年 11月 12日 7時00分";

        let result = create_text_for_user(
            consultation_req_id,
            consultant_id,
            fee_per_hour_in_yen,
            consultation_date_time,
        );

        let expected = format!(
            r"相談申し込み（相談申し込み番号: {}）が成立しました。ログイン後、スケジュールに相談室が作られていることをご確認下さい。下記に成立した相談申し込みの詳細を記載いたします。

相談相手
  コンサルタントID: {}

相談料金
  {} 円

相談開始日時
  {}

相談開始日時の{}日前までに下記の口座に入金をお願いいたします（入金の際にかかる振込手数料はユーザーのご負担となります）入金が確認できない場合、相談を行うことは出来ないため、期日に余裕を持って入金をするようお願いいたします。

  銀行名: {} (銀行コード: {})
  支店名: {} (支店コード: {})
  口座種別: {}
  口座番号: {}
  口座名義人: {}

入金の際、依頼人名は姓名、空白、相談日時（6桁の数字）として下さい（例 姓名がタナカ　タロウ、相談日時が9月29日19時のとき、依頼人名は「タナカ　タロウ　０９２９１９」となります（※））お名前に加えて相談日時を確認できない場合、正しく入金確認出来ないことがありますので必ず前述の通りご入力をお願いします。

（※）依頼人名に入力可能な文字数制限に達して全て入力出来ない場合、可能なところまで入力して振り込みを行って下さい。

【お問い合わせ先】
Email: {}",
            consultation_req_id,
            consultant_id,
            fee_per_hour_in_yen,
            consultation_date_time,
            DEADLINE_OF_PAYMENT_IN_DAYS,
            *BANK_NAME,
            *BANK_CODE,
            *BANK_BRANCH_NAME,
            *BANK_BRANCH_CODE,
            BANK_ACCOUNT_TYPE,
            *BANK_ACCOUNT_NUMBER,
            *BANK_ACCOUNT_HOLDER_NAME,
            INQUIRY_EMAIL_ADDRESS.as_str()
        );

        assert_eq!(result, expected);
    }

    #[test]
    fn test_create_text_for_consultant() {
        let consultation_req_id = 1312;
        let user_account_id = 533;
        let fee_per_hour_in_yen = 5000;
        let consultation_date_time = "2022年 11月 12日 7時00分";

        let result = create_text_for_consultant(
            consultation_req_id,
            user_account_id,
            fee_per_hour_in_yen,
            consultation_date_time,
        );

        let expected = format!(
            r"相談申し込み（相談申し込み番号: {}）が成立しました。ログイン後、スケジュールに相談室が作られていることをご確認下さい。下記に成立した相談申し込みの詳細を記載いたします。

相談申し込み者
  ユーザーID: {}

相談料金
  {} 円

相談開始日時
  {}

【お問い合わせ先】
Email: {}",
            consultation_req_id,
            user_account_id,
            fee_per_hour_in_yen,
            consultation_date_time,
            INQUIRY_EMAIL_ADDRESS.as_str()
        );

        assert_eq!(result, expected);
    }
}
