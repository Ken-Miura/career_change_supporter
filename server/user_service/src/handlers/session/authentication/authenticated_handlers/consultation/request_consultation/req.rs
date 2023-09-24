// Copyright 2023 Ken Miura

use axum::async_trait;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Timelike, Utc};
use common::smtp::{SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS};
use common::{ApiError, JAPANESE_TIME_ZONE, WEB_SITE_NAME};
use common::{ErrResp, RespResult};
use entity::sea_orm::ActiveValue::NotSet;
use entity::sea_orm::{ActiveModelTrait, Set};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::err::Code;
use crate::handlers::session::authentication::authenticated_handlers::authenticated_users::verified_user::VerifiedUser;
use crate::handlers::session::authentication::authenticated_handlers::consultation::ConsultationDateTime;
use crate::handlers::session::authentication::user_operation::{FindUserInfoOperationImpl};
use crate::optional_env_var::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS;
use crate::{err::unexpected_err_resp};

use super::consultation_date_time_validator::{
    validate_consultation_date_time, ConsultationDateTimeValidationError,
};

static CONSULTANT_MAIL_SUBJECT: Lazy<String> =
    Lazy::new(|| format!("[{}] 相談申し込み通知", WEB_SITE_NAME));
static USER_ACCOUNT_MAIL_SUBJECT: Lazy<String> =
    Lazy::new(|| format!("[{}] 相談申し込み完了通知", WEB_SITE_NAME));

static MIN_DURATION_IN_DAYS_BEFORE_CONSULTATION_ACCEPTANCE: Lazy<u32> =
    Lazy::new(|| *MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS / (24 * 60 * 60));

pub(crate) async fn post_request_consultation(
    VerifiedUser { user_info }: VerifiedUser,
    State(smtp_client): State<SmtpClient>,
    State(pool): State<DatabaseConnection>,
    Json(param): Json<RequestConsultationParam>,
) -> RespResult<RequestConsultationResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = RequestConsultationOperationImpl { pool };
    handle_request_consultation(
        user_info.account_id,
        user_info.email_address,
        param,
        &current_date_time,
        op,
        smtp_client,
    )
    .await
}

#[derive(Clone, Deserialize, Debug)]
pub(crate) struct RequestConsultationParam {
    consultant_id: i64,
    fee_per_hour_in_yen: i32,
    first_candidate_in_jst: ConsultationDateTime,
    second_candidate_in_jst: ConsultationDateTime,
    third_candidate_in_jst: ConsultationDateTime,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct RequestConsultationResult {}

#[derive(Debug, PartialEq)]
struct Candidates {
    first_candidate_in_jst: DateTime<FixedOffset>,
    second_candidate_in_jst: DateTime<FixedOffset>,
    third_candidate_in_jst: DateTime<FixedOffset>,
}

#[async_trait]
trait RequestConsultationOperation {
    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp>;

    async fn find_fee_per_hour_in_yen_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Option<i32>, ErrResp>;

    async fn create_request_consultation(
        &self,
        user_account_id: i64,
        consultant_id: i64,
        candidates: &Candidates,
        latest_candidate_date_time_in_jst: DateTime<FixedOffset>,
        fee_per_hour_in_yen: i32,
    ) -> Result<i64, ErrResp>;

    async fn get_consultant_email_address_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<String, ErrResp>;
}

struct RequestConsultationOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl RequestConsultationOperation for RequestConsultationOperationImpl {
    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp> {
        let op = FindUserInfoOperationImpl::new(&self.pool);
        super::super::check_if_consultant_is_available(consultant_id, &op).await
    }

    async fn find_fee_per_hour_in_yen_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Option<i32>, ErrResp> {
        let model = entity::consulting_fee::Entity::find_by_id(consultant_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find consulting_fee (consultant_id: {}): {}",
                    consultant_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.fee_per_hour_in_yen))
    }

    async fn create_request_consultation(
        &self,
        user_account_id: i64,
        consultant_id: i64,
        candidates: &Candidates,
        latest_candidate_date_time_in_jst: DateTime<FixedOffset>,
        fee_per_hour_in_yen: i32,
    ) -> Result<i64, ErrResp> {
        let active_model = entity::consultation_req::ActiveModel {
            consultation_req_id: NotSet,
            user_account_id: Set(user_account_id),
            consultant_id: Set(consultant_id),
            first_candidate_date_time: Set(candidates.first_candidate_in_jst),
            second_candidate_date_time: Set(candidates.second_candidate_in_jst),
            third_candidate_date_time: Set(candidates.third_candidate_in_jst),
            latest_candidate_date_time: Set(latest_candidate_date_time_in_jst),
            fee_per_hour_in_yen: Set(fee_per_hour_in_yen),
        };
        let result = active_model.insert(&self.pool).await.map_err(|e| {
            error!(
                "failed to insert consultation_req (user_account_id: {}, consultant_id: {}, candidates: {:?}, latest_candidate_date_time_in_jst: {}, fee_per_hour_in_yen: {}): {}",
                user_account_id, consultant_id, candidates, latest_candidate_date_time_in_jst, fee_per_hour_in_yen, e
            );
            unexpected_err_resp()
        })?;
        Ok(result.consultation_req_id)
    }

    async fn get_consultant_email_address_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<String, ErrResp> {
        let model_option = entity::user_account::Entity::find_by_id(consultant_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find user_account (consultant_id: {}): {}",
                    consultant_id, e
                );
                unexpected_err_resp()
            })?;
        let model = match model_option {
            Some(m) => m,
            None => {
                error!("No consultant found");
                return Err(unexpected_err_resp());
            }
        };
        Ok(model.email_address)
    }
}

async fn handle_request_consultation(
    user_account_id: i64,
    user_email_address: String,
    request_consultation_param: RequestConsultationParam,
    current_date_time: &DateTime<FixedOffset>,
    op: impl RequestConsultationOperation,
    send_mail: impl SendMail,
) -> RespResult<RequestConsultationResult> {
    let consultant_id = request_consultation_param.consultant_id;
    validate_consultant_id_is_positive(consultant_id)?;
    validate_candidates(
        &request_consultation_param.first_candidate_in_jst,
        &request_consultation_param.second_candidate_in_jst,
        &request_consultation_param.third_candidate_in_jst,
        current_date_time,
    )?;
    // 操作者（ユーザー）のアカウントが無効化されているかどうかは個々のURLを示すハンドラに来る前の共通箇所でチェックする
    // 従って、アカウントが無効化されているかどうかは相談申し込みの相手のみ確認する
    validate_consultant_is_available(consultant_id, &op).await?;

    let fee_per_hour_in_yen = get_fee_per_hour_in_yen(consultant_id, &op).await?;
    if fee_per_hour_in_yen != request_consultation_param.fee_per_hour_in_yen {
        error!(
            "fee_per_hour_in_yen was updated (user's request: {}, consultant's fee: {})",
            request_consultation_param.fee_per_hour_in_yen, fee_per_hour_in_yen
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::FeePerHourInYenWasUpdated as u32,
            }),
        ));
    }

    let candidates = convert_to_candidates(
        request_consultation_param.first_candidate_in_jst,
        request_consultation_param.second_candidate_in_jst,
        request_consultation_param.third_candidate_in_jst,
    )?;
    let latest_candiate_in_jst = extract_latest_candidate_date_time_in_jst(&candidates)?;

    let consultation_req_id = op
        .create_request_consultation(
            user_account_id,
            consultant_id,
            &candidates,
            latest_candiate_in_jst,
            fee_per_hour_in_yen,
        )
        .await?;

    let consultant_email_address = op
        .get_consultant_email_address_by_consultant_id(consultant_id)
        .await?;

    let candidates_in_string = convert_candidates_to_string(&candidates);

    send_mail_to_consultant(
        consultant_email_address.as_str(),
        user_account_id,
        consultation_req_id,
        &candidates_in_string,
        &send_mail,
    )
    .await?;

    send_mail_to_user(
        user_email_address.as_str(),
        consultant_id,
        consultation_req_id,
        &candidates_in_string,
        fee_per_hour_in_yen,
        &send_mail,
    )
    .await?;

    Ok((StatusCode::OK, Json(RequestConsultationResult {})))
}

fn validate_consultant_id_is_positive(consultant_id: i64) -> Result<(), ErrResp> {
    if !consultant_id.is_positive() {
        error!("consultant_id ({}) is not positive", consultant_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonPositiveConsultantId as u32,
            }),
        ));
    }
    Ok(())
}

fn validate_candidates(
    first_candidate_in_jst: &ConsultationDateTime,
    second_candidate_in_jst: &ConsultationDateTime,
    third_candidate_in_jst: &ConsultationDateTime,
    current_date_time: &DateTime<FixedOffset>,
) -> Result<(), ErrResp> {
    validate_consultation_date_time(first_candidate_in_jst, current_date_time).map_err(|e| {
        error!("invalid first_candidate_in_jst: {}", e);
        convert_consultation_date_time_validation_err(&e)
    })?;
    validate_consultation_date_time(second_candidate_in_jst, current_date_time).map_err(|e| {
        error!("invalid second_candidate_in_jst: {}", e);
        convert_consultation_date_time_validation_err(&e)
    })?;
    validate_consultation_date_time(third_candidate_in_jst, current_date_time).map_err(|e| {
        error!("invalid third_candidate_in_jst: {}", e);
        convert_consultation_date_time_validation_err(&e)
    })?;

    if first_candidate_in_jst == second_candidate_in_jst
        || second_candidate_in_jst == third_candidate_in_jst
        || third_candidate_in_jst == first_candidate_in_jst
    {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::DuplicateDateTimeCandidates as u32,
            }),
        ));
    }

    Ok(())
}

fn convert_consultation_date_time_validation_err(
    e: &ConsultationDateTimeValidationError,
) -> ErrResp {
    match e {
        ConsultationDateTimeValidationError::IllegalDateTime {
            year: _,
            month: _,
            day: _,
            hour: _,
        } => (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::IllegalConsultationDateTime as u32,
            }),
        ),
        ConsultationDateTimeValidationError::IllegalConsultationHour { hour: _ } => (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::IllegalConsultationHour as u32,
            }),
        ),
        ConsultationDateTimeValidationError::InvalidConsultationDateTime {
            consultation_date_time: _,
            current_date_time: _,
        } => (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::InvalidConsultationDateTime as u32,
            }),
        ),
    }
}

async fn validate_consultant_is_available(
    consultant_id: i64,
    op: &impl RequestConsultationOperation,
) -> Result<(), ErrResp> {
    let consultant_available = op.check_if_consultant_is_available(consultant_id).await?;
    if !consultant_available {
        error!(
            "consultant is not available (consultant_id: {})",
            consultant_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultantIsNotAvailable as u32,
            }),
        ));
    }
    Ok(())
}

async fn get_fee_per_hour_in_yen(
    consultant_id: i64,
    op: &impl RequestConsultationOperation,
) -> Result<i32, ErrResp> {
    let fee_per_hour_in_yen = op
        .find_fee_per_hour_in_yen_by_consultant_id(consultant_id)
        .await?;
    let fee_per_hour_in_yen = fee_per_hour_in_yen.ok_or_else(|| {
        error!(
            "fee_per_hour_in_yen does not exist (consultant_id: {})",
            consultant_id
        );
        unexpected_err_resp()
    })?;
    Ok(fee_per_hour_in_yen)
}

fn convert_to_candidates(
    first_candidate_consultation_date_time_in_jst: ConsultationDateTime,
    second_candidate_consultation_date_time_in_jst: ConsultationDateTime,
    third_candidate_consultation_date_time_in_jst: ConsultationDateTime,
) -> Result<Candidates, ErrResp> {
    let first_candidate_in_jst =
        convert_to_date_time(first_candidate_consultation_date_time_in_jst)?;
    let second_candidate_in_jst =
        convert_to_date_time(second_candidate_consultation_date_time_in_jst)?;
    let third_candidate_in_jst =
        convert_to_date_time(third_candidate_consultation_date_time_in_jst)?;
    Ok(Candidates {
        first_candidate_in_jst,
        second_candidate_in_jst,
        third_candidate_in_jst,
    })
}

fn convert_to_date_time(
    consultation_date_time_in_jst: ConsultationDateTime,
) -> Result<DateTime<FixedOffset>, ErrResp> {
    // 日本のタイムゾーンにおいて、（サマータイム等による）タイムゾーンの遷移は発生しないので一意にならない場合はすべてエラー
    let date_time = match JAPANESE_TIME_ZONE.with_ymd_and_hms(
        consultation_date_time_in_jst.year,
        consultation_date_time_in_jst.month,
        consultation_date_time_in_jst.day,
        consultation_date_time_in_jst.hour,
        0,
        0,
    ) {
        chrono::LocalResult::None => {
            error!(
                "failed to get date_time (consultation_date_time_in_jst: {:?})",
                consultation_date_time_in_jst
            );
            return Err(unexpected_err_resp());
        }
        chrono::LocalResult::Single(s) => s,
        chrono::LocalResult::Ambiguous(a1, a2) => {
            error!("failed to get date_time (consultation_date_time_in_jst: {:?}, ambiguous1: {}, ambiguous2: {})", consultation_date_time_in_jst, a1, a2);
            return Err(unexpected_err_resp());
        }
    };
    Ok(date_time)
}

fn extract_latest_candidate_date_time_in_jst(
    candidates: &Candidates,
) -> Result<DateTime<FixedOffset>, ErrResp> {
    let candidates_in_jst = vec![
        candidates.second_candidate_in_jst,
        candidates.third_candidate_in_jst,
    ];
    let latest_candidate_in_jst =
        select_latest_candidate_in_jst(candidates.first_candidate_in_jst, candidates_in_jst);
    Ok(latest_candidate_in_jst)
}

fn select_latest_candidate_in_jst(
    first_candidate_in_jst: DateTime<FixedOffset>,
    candidates_in_jst: Vec<DateTime<FixedOffset>>,
) -> DateTime<FixedOffset> {
    let mut latest_candidate_in_jst = first_candidate_in_jst;
    for c in candidates_in_jst.iter() {
        if c > &latest_candidate_in_jst {
            latest_candidate_in_jst = *c
        }
    }
    latest_candidate_in_jst
}

fn convert_candidates_to_string(candidates: &Candidates) -> (String, String, String) {
    (
        format!(
            "{}年 {}月 {}日 {}時00分",
            candidates.first_candidate_in_jst.year(),
            candidates.first_candidate_in_jst.month(),
            candidates.first_candidate_in_jst.day(),
            candidates.first_candidate_in_jst.hour()
        ),
        format!(
            "{}年 {}月 {}日 {}時00分",
            candidates.second_candidate_in_jst.year(),
            candidates.second_candidate_in_jst.month(),
            candidates.second_candidate_in_jst.day(),
            candidates.second_candidate_in_jst.hour()
        ),
        format!(
            "{}年 {}月 {}日 {}時00分",
            candidates.third_candidate_in_jst.year(),
            candidates.third_candidate_in_jst.month(),
            candidates.third_candidate_in_jst.day(),
            candidates.third_candidate_in_jst.hour()
        ),
    )
}

async fn send_mail_to_consultant(
    consultant_email_address: &str,
    user_account_id: i64,
    consultation_req_id: i64,
    candidates_in_string: &(String, String, String),
    send_mail: &impl SendMail,
) -> Result<(), ErrResp> {
    let text =
        create_text_for_consultant_mail(user_account_id, consultation_req_id, candidates_in_string);
    send_mail
        .send_mail(
            consultant_email_address,
            SYSTEM_EMAIL_ADDRESS.as_str(),
            CONSULTANT_MAIL_SUBJECT.as_str(),
            text.as_str(),
        )
        .await?;
    Ok(())
}

fn create_text_for_consultant_mail(
    user_account_id: i64,
    consultation_req_id: i64,
    candidates: &(String, String, String),
) -> String {
    // TODO: 文面の調整
    format!(
        r"ユーザーID ({}) から相談申し込み（相談申し込み番号: {}）が届きました。相談者からの希望相談開始日時を下記に記載します。{}へログインし、相談受け付けのページから該当の申し込みの詳細を確認し、了承する、または拒否するをご選択下さい。

希望相談開始日時
  第一希望: {}
  第二希望: {}
  第三希望: {}

各希望相談開始日時について、その日時の{}日前となると、その日時を選択して了承することができなくなりますのでご注意下さい。

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        user_account_id,
        consultation_req_id,
        WEB_SITE_NAME,
        candidates.0,
        candidates.1,
        candidates.2,
        *MIN_DURATION_IN_DAYS_BEFORE_CONSULTATION_ACCEPTANCE,
        INQUIRY_EMAIL_ADDRESS.as_str()
    )
}

async fn send_mail_to_user(
    user_account_email_address: &str,
    consultant_id: i64,
    consultation_req_id: i64,
    candidates_in_string: &(String, String, String),
    fee_per_hour_in_yen: i32,
    send_mail: &impl SendMail,
) -> Result<(), ErrResp> {
    let text = create_text_for_user_mail(
        consultant_id,
        consultation_req_id,
        fee_per_hour_in_yen,
        candidates_in_string,
    );
    send_mail
        .send_mail(
            user_account_email_address,
            SYSTEM_EMAIL_ADDRESS.as_str(),
            USER_ACCOUNT_MAIL_SUBJECT.as_str(),
            text.as_str(),
        )
        .await?;
    Ok(())
}

fn create_text_for_user_mail(
    consultant_id: i64,
    consultation_req_id: i64,
    fee_per_hour_in_yen: i32,
    candidates: &(String, String, String),
) -> String {
    // TODO: 文面の調整
    format!(
        r"下記の内容で相談申し込み（相談申し込み番号: {}）を行いました。

相談相手
  コンサルタントID: {}

相談料金
  {} 円

希望相談開始日時
  第一希望: {}
  第二希望: {}
  第三希望: {}

相談申し込みが拒否されていない限り、希望相談開始日時の{}日前までは、コンサルタントの相談申し込みに対する了承の可能性があります。相談申し込みが了承されたことを見逃さないために、各希望相談開始日時の{}日前には{}にログイン後、スケジュールのページをご確認下さい。

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        consultation_req_id,
        consultant_id,
        fee_per_hour_in_yen,
        candidates.0,
        candidates.1,
        candidates.2,
        *MIN_DURATION_IN_DAYS_BEFORE_CONSULTATION_ACCEPTANCE,
        *MIN_DURATION_IN_DAYS_BEFORE_CONSULTATION_ACCEPTANCE,
        WEB_SITE_NAME,
        INQUIRY_EMAIL_ADDRESS.as_str()
    )
}

#[cfg(test)]
mod tests {

    // use common::payment_platform::customer::Card;
    // use once_cell::sync::Lazy;

    // use super::*;

    // #[derive(Debug)]
    // struct TestCase {
    //     name: String,
    //     input: Input,
    //     expected: RespResult<RequestConsultationResult>,
    // }

    // #[derive(Debug)]
    // struct Input {
    //     account_id: i64,
    //     param: RequestConsultationParam,
    //     current_date_time: DateTime<FixedOffset>,
    //     req_op: RequestConsultationOperationMock,
    // }

    // #[derive(Clone, Debug)]
    // struct RequestConsultationOperationMock {
    //     consultant_id: i64,
    //     fee_per_hour_in_yen: Option<i32>,
    //     tenant_id: Option<String>,
    //     card_token: String,
    //     charge: Charge,
    //     current_date_time: DateTime<FixedOffset>,
    //     first_candidate_in_jst: DateTime<FixedOffset>,
    //     second_candidate_in_jst: DateTime<FixedOffset>,
    //     third_candidate_in_jst: DateTime<FixedOffset>,
    //     settlement_payments: Vec<PaymentInfo>,
    //     stopped_settlement_payments: Vec<PaymentInfo>,
    //     receipt_payments: Vec<PaymentInfo>,
    // }

    // #[async_trait]
    // impl RequestConsultationOperation for RequestConsultationOperationMock {
    //     async fn check_if_consultant_is_available(
    //         &self,
    //         consultant_id: i64,
    //     ) -> Result<bool, ErrResp> {
    //         if self.consultant_id != consultant_id {
    //             return Ok(false);
    //         };
    //         Ok(true)
    //     }

    //     async fn find_fee_per_hour_in_yen_by_consultant_id(
    //         &self,
    //         consultant_id: i64,
    //     ) -> Result<Option<i32>, ErrResp> {
    //         assert_eq!(self.consultant_id, consultant_id);
    //         Ok(self.fee_per_hour_in_yen)
    //     }

    //     async fn find_tenant_id_by_consultant_id(
    //         &self,
    //         consultant_id: i64,
    //     ) -> Result<Option<String>, ErrResp> {
    //         assert_eq!(self.consultant_id, consultant_id);
    //         Ok(self.tenant_id.clone())
    //     }

    //     async fn filter_valid_settlement_by_consultant_id(
    //         &self,
    //         consultant_id: i64,
    //         current_date_time: &DateTime<FixedOffset>,
    //     ) -> Result<Vec<PaymentInfo>, ErrResp> {
    //         assert_eq!(self.consultant_id, consultant_id);
    //         assert_eq!(self.current_date_time, *current_date_time);
    //         Ok(self.settlement_payments.clone())
    //     }

    //     async fn filter_valid_stopped_settlement_by_consultant_id(
    //         &self,
    //         consultant_id: i64,
    //         current_date_time: &DateTime<FixedOffset>,
    //     ) -> Result<Vec<PaymentInfo>, ErrResp> {
    //         assert_eq!(self.consultant_id, consultant_id);
    //         assert_eq!(self.current_date_time, *current_date_time);
    //         Ok(self.stopped_settlement_payments.clone())
    //     }

    //     async fn filter_receipts_of_the_duration_by_consultant_id(
    //         &self,
    //         consultant_id: i64,
    //         start: &DateTime<FixedOffset>,
    //         end: &DateTime<FixedOffset>,
    //     ) -> Result<Vec<PaymentInfo>, ErrResp> {
    //         assert_eq!(self.consultant_id, consultant_id);
    //         let (s, e) = create_start_and_end_date_time_of_current_year(&self.current_date_time)?;
    //         assert_eq!(s, *start);
    //         assert_eq!(e, *end);
    //         Ok(self.receipt_payments.clone())
    //     }

    //     async fn create_charge(&self, create_charge: &CreateCharge) -> Result<Charge, ErrResp> {
    //         assert_eq!(
    //             self.fee_per_hour_in_yen.expect("failed to get Ok"),
    //             create_charge.price().expect("failed to get Ok").0
    //         );
    //         assert_eq!(
    //             self.card_token,
    //             create_charge.card().expect("failed to get Ok")
    //         );
    //         assert_eq!(self.tenant_id, create_charge.tenant());
    //         let metadata = create_charge.metadata().expect("failed to get Ok");
    //         let consultant_id = metadata
    //             .get(KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ)
    //             .expect("failed to get Ok")
    //             .parse::<i64>()
    //             .expect("failed to get Ok");
    //         assert_eq!(self.consultant_id, consultant_id);
    //         let first_candidate = metadata
    //             .get(KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ)
    //             .expect("failed to get Ok");
    //         let first_candidate = DateTime::<FixedOffset>::parse_from_rfc3339(first_candidate)
    //             .expect("failed to get Ok");
    //         assert_eq!(self.first_candidate_in_jst, first_candidate);
    //         let second_candidate = metadata
    //             .get(KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ)
    //             .expect("failed to get Ok");
    //         let second_candidate = DateTime::<FixedOffset>::parse_from_rfc3339(second_candidate)
    //             .expect("failed to get Ok");
    //         assert_eq!(self.second_candidate_in_jst, second_candidate);
    //         let third_candidate = metadata
    //             .get(KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ)
    //             .expect("failed to get Ok");
    //         let third_candidate = DateTime::<FixedOffset>::parse_from_rfc3339(third_candidate)
    //             .expect("failed to get Ok");
    //         assert_eq!(self.third_candidate_in_jst, third_candidate);
    //         Ok(self.charge.clone())
    //     }
    // }

    // static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
    //     vec![
    //         TestCase {
    //             name: "success case 1".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 4000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 7,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 23,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 22,
    //                         hour: 7,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: Some(4000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 23, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Ok((
    //                 StatusCode::OK,
    //                 Json(RequestConsultationResult {
    //                     charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "consultant id is negative".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: -1,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 7,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 23,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 22,
    //                         hour: 7,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: -1,
    //                     fee_per_hour_in_yen: Some(5000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 23, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::NonPositiveConsultantId as u32,
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "first_candidate_in_jst IllegalConsultationDateTime".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 24,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 23,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 22,
    //                         hour: 7,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: Some(5000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 23, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::IllegalConsultationDateTime as u32,
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "first_candidate_in_jst IllegalConsultationHour".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 6,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 23,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 22,
    //                         hour: 7,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: Some(5000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 6, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 23, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::IllegalConsultationHour as u32,
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "first_candidate_in_jst InvalidConsultationDateTime".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 7,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 23,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 22,
    //                         hour: 7,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 1)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: Some(5000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 1)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 23, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::InvalidConsultationDateTime as u32,
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "second_candidate_in_jst IllegalConsultationDateTime".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 7,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 24,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 22,
    //                         hour: 7,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: Some(5000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 23, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::IllegalConsultationDateTime as u32,
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "second_candidate_in_jst IllegalConsultationHour".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 7,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 0,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 22,
    //                         hour: 7,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: Some(5000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 0, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::IllegalConsultationHour as u32,
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "second_candidate_in_jst InvalidConsultationDateTime".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 7,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 22,
    //                         hour: 7,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 12,
    //                         hour: 7,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 6, 59, 59)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: Some(5000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 6, 59, 59)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 12, 7, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::InvalidConsultationDateTime as u32,
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "third_candidate_in_jst IllegalConsultationDateTime".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 7,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 23,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 21,
    //                         hour: 24,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: Some(5000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 23, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 21, 7, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::IllegalConsultationDateTime as u32,
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "third_candidate_in_jst IllegalConsultationHour".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 7,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 23,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 22,
    //                         hour: 6,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: Some(5000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 23, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 22, 6, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::IllegalConsultationHour as u32,
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "third_candidate_in_jst InvalidConsultationDateTime".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 5,
    //                         hour: 7,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 22,
    //                         hour: 7,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 7,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 1)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: Some(5000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 1)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 5, 7, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::InvalidConsultationDateTime as u32,
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "first_candidate_in_jst == second_candidate_in_jst".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 5,
    //                         hour: 7,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 5,
    //                         hour: 7,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 7,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: Some(5000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 5, 7, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 5, 7, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::DuplicateDateTimeCandidates as u32,
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "second_candidate_in_jst == third_candidate_in_jst".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 5,
    //                         hour: 7,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 18,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 18,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: Some(5000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 5, 7, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 18, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 18, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::DuplicateDateTimeCandidates as u32,
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "third_candidate_in_jst == first_candidate_in_jst".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 20,
    //                         hour: 21,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 18,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 20,
    //                         hour: 21,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: Some(5000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 20, 21, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 18, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 20, 21, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::DuplicateDateTimeCandidates as u32,
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "fail ConsultantIsNotAvailable".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 14,
    //                         hour: 21,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 18,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 20,
    //                         hour: 21,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 3,
    //                     fee_per_hour_in_yen: Some(5000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 14, 21, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 18, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 20, 21, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::ConsultantIsNotAvailable as u32,
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "fail FeePerHourInYenWasUpdated".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 14,
    //                         hour: 21,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 18,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 20,
    //                         hour: 21,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: Some(6000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 14, 21, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 18, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 20, 21, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     stopped_settlement_payments: vec![PaymentInfo {
    //                         fee_per_hour_in_yen: 5000,
    //                         platform_fee_rate_in_percentage: "30.0".to_string(),
    //                     }],
    //                     receipt_payments: vec![
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                         PaymentInfo {
    //                             fee_per_hour_in_yen: 5000,
    //                             platform_fee_rate_in_percentage: "30.0".to_string(),
    //                         },
    //                     ],
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::FeePerHourInYenWasUpdated as u32,
    //                 }),
    //             )),
    //         },
    //         TestCase {
    //             name: "fail ExceedMaxAnnualRewards".to_string(),
    //             input: Input {
    //                 account_id: 1,
    //                 param: RequestConsultationParam {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: 5000,
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     first_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 14,
    //                         hour: 21,
    //                     },
    //                     second_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 4,
    //                         hour: 18,
    //                     },
    //                     third_candidate_in_jst: ConsultationDateTime {
    //                         year: 2022,
    //                         month: 11,
    //                         day: 20,
    //                         hour: 21,
    //                     },
    //                 },
    //                 current_date_time: JAPANESE_TIME_ZONE
    //                     .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                     .unwrap(),
    //                 req_op: RequestConsultationOperationMock {
    //                     consultant_id: 2,
    //                     fee_per_hour_in_yen: Some(5000),
    //                     tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //                     card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
    //                     charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
    //                     current_date_time: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 1, 7, 0, 0)
    //                         .unwrap(),
    //                     first_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 14, 21, 0, 0)
    //                         .unwrap(),
    //                     second_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 4, 18, 0, 0)
    //                         .unwrap(),
    //                     third_candidate_in_jst: JAPANESE_TIME_ZONE
    //                         .with_ymd_and_hms(2022, 11, 20, 21, 0, 0)
    //                         .unwrap(),
    //                     settlement_payments:
    //                         create_payments_that_has_reward_over_one_third_of_max_rewards(),
    //                     stopped_settlement_payments:
    //                         create_payments_that_has_reward_over_one_third_of_max_rewards(),
    //                     receipt_payments:
    //                         create_payments_that_has_reward_over_one_third_of_max_rewards(),
    //                 },
    //             },
    //             expected: Err((
    //                 StatusCode::BAD_REQUEST,
    //                 Json(ApiError {
    //                     code: Code::ExceedMaxAnnualRewards as u32,
    //                 }),
    //             )),
    //         },
    //     ]
    // });

    // // create_dummy_chargeでAPI呼び出しの結果返却されるChargeを作成する
    // // 返却されたChargeはidだけ利用し、他を参照することはないのでid以外はダミーで関係ない値で埋めてある
    // fn create_dummy_charge(charge_id: &str) -> Charge {
    //     Charge {
    //         id: charge_id.to_string(),
    //         object: "charge".to_string(),
    //         livemode: false,
    //         created: 1639931415,
    //         amount: 5000,
    //         currency: "jpy".to_string(),
    //         paid: true,
    //         expired_at: None,
    //         captured: false,
    //         captured_at: Some(1639931415),
    //         card: Some(Card {
    //             object: "card".to_string(),
    //             id: "car_33ab04bcdc00f0cc6d6df16bbe79".to_string(),
    //             created: 1639931415,
    //             name: None,
    //             last4: "4242".to_string(),
    //             exp_month: 12,
    //             exp_year: 2022,
    //             brand: "Visa".to_string(),
    //             cvc_check: "passed".to_string(),
    //             fingerprint: "e1d8225886e3a7211127df751c86787f".to_string(),
    //             address_state: None,
    //             address_city: None,
    //             address_line1: None,
    //             address_line2: None,
    //             country: None,
    //             address_zip: None,
    //             address_zip_check: "unchecked".to_string(),
    //             metadata: None,
    //         }),
    //         customer: None,
    //         description: None,
    //         failure_code: None,
    //         failure_message: None,
    //         fee_rate: Some("3.00".to_string()),
    //         refunded: false,
    //         amount_refunded: 0,
    //         refund_reason: None,
    //         subscription: None,
    //         metadata: None,
    //         platform_fee: None,
    //         tenant: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
    //         platform_fee_rate: Some("30.0".to_string()),
    //         total_platform_fee: Some(1350),
    //         three_d_secure_status: Some("unverified".to_string()),
    //     }
    // }

    // fn create_payments_that_has_reward_over_one_third_of_max_rewards() -> Vec<PaymentInfo> {
    //     // *MAX_ANNUAL_REWARDS_IN_YENのおよそ三分の一
    //     let one_third_of_max = 156667;
    //     let mut result = vec![];
    //     let mut rewards = 0;
    //     while rewards < one_third_of_max {
    //         let p = PaymentInfo {
    //             fee_per_hour_in_yen: 3000,
    //             platform_fee_rate_in_percentage: "30.0".to_string(),
    //         };
    //         result.push(p);
    //         rewards += 2100;
    //     }
    //     result
    // }

    // #[tokio::test]
    // async fn handle_request_consultation_tests() {
    //     for test_case in TEST_CASE_SET.iter() {
    //         let account_id = test_case.input.account_id;
    //         let param = test_case.input.param.clone();
    //         let current_date_time = test_case.input.current_date_time;
    //         let req_op = test_case.input.req_op.clone();

    //         let resp =
    //             handle_request_consultation(account_id, param, &current_date_time, req_op).await;

    //         let message = format!("test case \"{}\" failed", test_case.name.clone());
    //         if test_case.expected.is_ok() {
    //             let result = resp.expect("failed to get Ok");
    //             let expected_result = test_case.expected.as_ref().expect("failed to get Ok");
    //             assert_eq!(expected_result.0, result.0, "{}", message);
    //             assert_eq!(expected_result.1 .0, result.1 .0, "{}", message);
    //         } else {
    //             let result = resp.expect_err("failed to get Err");
    //             let expected_result = test_case.expected.as_ref().expect_err("failed to get Err");
    //             assert_eq!(expected_result.0, result.0, "{}", message);
    //             assert_eq!(expected_result.1 .0, result.1 .0, "{}", message);
    //         }
    //     }
    // }
}
