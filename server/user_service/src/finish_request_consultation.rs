// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{DateTime, Datelike, Duration, FixedOffset, Timelike, Utc};
use common::payment_platform::charge::{Charge, ChargeOperation, ChargeOperationImpl};
use common::payment_platform::Metadata;
use common::smtp::{
    SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SMTP_HOST, SMTP_PASSWORD, SMTP_PORT,
    SMTP_USERNAME, SYSTEM_EMAIL_ADDRESS,
};
use common::{ApiError, JAPANESE_TIME_ZONE, WEB_SITE_NAME};
use common::{ErrResp, ErrRespStruct, RespResult};
use entity::sea_orm::ActiveValue::NotSet;
use entity::sea_orm::{
    ActiveModelTrait, DatabaseConnection, EntityTrait, Set, TransactionError, TransactionTrait,
};
use entity::user_account;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::err::{unexpected_err_resp, Code};
use crate::util::session::User;
use crate::util::{
    self, ACCESS_INFO, EXPIRY_DAYS, KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ,
    KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ, KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
    KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ, MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION,
};

static CONSULTANT_MAIL_SUBJECT: Lazy<String> =
    Lazy::new(|| format!("[{}] 相談申し込み通知", WEB_SITE_NAME));
static USER_ACCOUNT_MAIL_SUBJECT: Lazy<String> =
    Lazy::new(|| format!("[{}] 相談申し込み完了通知", WEB_SITE_NAME));

pub(crate) async fn post_finish_request_consultation(
    User { account_id }: User,
    Json(param): Json<FinishRequestConsultationParam>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<FinishRequestConsultationResult> {
    let charge_id = param.charge_id;
    let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
    let op = FinishRequestConsultationOperationImpl { pool };
    let smtp_client = SmtpClient::new(
        SMTP_HOST.to_string(),
        *SMTP_PORT,
        SMTP_USERNAME.to_string(),
        SMTP_PASSWORD.to_string(),
    );
    handle_finish_request_consultation(account_id, charge_id, &current_date_time, op, smtp_client)
        .await
}

#[derive(Deserialize)]
pub(crate) struct FinishRequestConsultationParam {
    pub charge_id: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct FinishRequestConsultationResult {}

async fn handle_finish_request_consultation(
    account_id: i64,
    charge_id: String,
    current_date_time: &DateTime<FixedOffset>,
    op: impl FinishRequestConsultationOperation,
    send_mail: impl SendMail,
) -> RespResult<FinishRequestConsultationResult> {
    let _ = validate_identity_exists(account_id, &op).await?;
    let charge = op.get_charge_by_charge_id(charge_id.clone()).await?;
    let consultant_id = extract_consultant_id(&charge)?;
    let _ = validate_consultant_is_available(consultant_id, &op).await?;
    let _ = confirm_three_d_secure_status_is_ok(&charge)?;

    let expiry_date_time = *current_date_time + Duration::days(EXPIRY_DAYS as i64);
    let charge = op
        .create_request_consultation(account_id, consultant_id, charge_id, expiry_date_time)
        .await?;
    info!("finished 3D Secure flow (charge.id: {})", charge.id);

    let consultant_email_address = op
        .get_consultant_email_address_by_consultant_id(consultant_id)
        .await?;
    let _ = send_mail_to_consultant(
        consultant_email_address.as_str(),
        account_id,
        &charge,
        &send_mail,
    )
    .await?;

    let user_email_address = op
        .get_user_account_email_address_by_user_account_id(account_id)
        .await?;
    let _ = send_mail_to_user(
        user_email_address.as_str(),
        consultant_id,
        &charge,
        &send_mail,
    )
    .await?;

    Ok((StatusCode::OK, Json(FinishRequestConsultationResult {})))
}

async fn validate_identity_exists(
    account_id: i64,
    op: &impl FinishRequestConsultationOperation,
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

fn extract_consultant_id(charge: &Charge) -> Result<i64, ErrResp> {
    let metadata = match charge.metadata.clone() {
        Some(metadata) => metadata,
        None => {
            error!("no metadata found on charge (id: {})", charge.id);
            return Err(unexpected_err_resp());
        }
    };
    let consultant_id = match metadata.get(KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ) {
        Some(c_id) => c_id,
        None => {
            error!(
                "no consultant_id found in metadata on charge (id: {})",
                charge.id
            );
            return Err(unexpected_err_resp());
        }
    };
    let consultant_id = match consultant_id.parse::<i64>() {
        Ok(c_id) => c_id,
        Err(e) => {
            error!("failed to parse consultant_id in metadata on charge (id: {}, consultant_id: {}): {}", charge.id, consultant_id, e);
            return Err(unexpected_err_resp());
        }
    };
    Ok(consultant_id)
}

async fn validate_consultant_is_available(
    consultant_id: i64,
    op: &impl FinishRequestConsultationOperation,
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

fn confirm_three_d_secure_status_is_ok(charge: &Charge) -> Result<(), ErrResp> {
    let three_d_secure_status = match charge.three_d_secure_status.clone() {
        Some(s) => s,
        None => {
            error!(
                "three_d_secure_status is None (charge.id: {})",
                charge.id.clone()
            );
            return Err(unexpected_err_resp());
        }
    };
    if !(three_d_secure_status == "attempted" || three_d_secure_status == "verified") {
        error!(
            "3D secure is not finished correctly (three_d_secure_status: {}, charge.id: {})",
            three_d_secure_status,
            charge.id.clone()
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ThreeDSecureError as u32,
            }),
        ));
    }
    Ok(())
}

async fn send_mail_to_consultant(
    consultant_email_address: &str,
    user_account_id: i64,
    charge: &Charge,
    send_mail: &impl SendMail,
) -> Result<(), ErrResp> {
    let candidates = extract_candidates(charge)?;
    let text = create_text_for_consultant_mail(user_account_id, &candidates);
    let _ = send_mail
        .send_mail(
            consultant_email_address,
            SYSTEM_EMAIL_ADDRESS,
            CONSULTANT_MAIL_SUBJECT.as_str(),
            text.as_str(),
        )
        .await?;
    Ok(())
}

fn create_text_for_consultant_mail(
    user_account_id: i64,
    candidates: &(String, String, String),
) -> String {
    // TODO: 文面の調整
    format!(
        r"ユーザーID ({}) から相談申し込みの依頼（希望相談開始日時は下記に記載）が届きました。{}へログインし、相談受け付けのページから該当の申込みの詳細を確認し、了承する、または拒否するをご選択下さい。なお、{}日後までにどちらもご選択されていない場合、自動的に拒否するを選択されたものとして処理されます。

希望相談開始日時
  第一希望: {}
  第二希望: {}
  第三希望: {}

各希望相談開始日時について、その日時の{}時間前となると、その日時を選択して了承することができなくなりますのでご注意下さい。

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        user_account_id,
        WEB_SITE_NAME,
        EXPIRY_DAYS,
        candidates.0,
        candidates.1,
        candidates.2,
        MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION,
        INQUIRY_EMAIL_ADDRESS
    )
}

async fn send_mail_to_user(
    user_account_email_address: &str,
    consultant_id: i64,
    charge: &Charge,
    send_mail: &impl SendMail,
) -> Result<(), ErrResp> {
    let candidates = extract_candidates(charge)?;
    let text = create_text_for_user_mail(consultant_id, charge.amount, &candidates);
    let _ = send_mail
        .send_mail(
            user_account_email_address,
            SYSTEM_EMAIL_ADDRESS,
            USER_ACCOUNT_MAIL_SUBJECT.as_str(),
            text.as_str(),
        )
        .await?;
    Ok(())
}

fn create_text_for_user_mail(
    consultant_id: i64,
    amount: i32,
    candidates: &(String, String, String),
) -> String {
    // TODO: 文面の調整
    format!(
        r"下記の内容で相談申し込みを行いました。{}日後までに相談申し込みが了承されない場合、自動的に相談申し込みが拒否されたものとして扱われます。

相談相手
  コンサルタントID: {}

相談料金
  {} 円

希望相談開始日時
  第一希望: {}
  第二希望: {}
  第三希望: {}

相談申し込みが拒否されていない限り、希望相談開始日時の{}時間前までは、コンサルタントの相談申し込みに対する了承の可能性があります。相談申し込みが了承されたことを見逃さないために、各希望相談開始日時の{}時間前には{}にログイン後、スケジュールのページをご確認下さい。

本メールはシステムより自動配信されています。
本メールに返信されましても、回答いたしかねます。
お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

【お問い合わせ先】
Email: {}",
        EXPIRY_DAYS,
        consultant_id,
        amount,
        candidates.0,
        candidates.1,
        candidates.2,
        MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION,
        MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION,
        WEB_SITE_NAME,
        INQUIRY_EMAIL_ADDRESS
    )
}

fn extract_candidates(charge: &Charge) -> Result<(String, String, String), ErrResp> {
    let metadata = match charge.metadata.clone() {
        Some(metadata) => metadata,
        None => {
            error!("no metadata found on charge (id: {})", charge.id);
            return Err(unexpected_err_resp());
        }
    };
    let first_candidate_in_jst = extract_candidate_expression_in_japanese(
        KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
        &metadata,
    )?;
    let second_candidate_in_jst = extract_candidate_expression_in_japanese(
        KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
        &metadata,
    )?;
    let third_candidate_in_jst = extract_candidate_expression_in_japanese(
        KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
        &metadata,
    )?;
    Ok((
        first_candidate_in_jst,
        second_candidate_in_jst,
        third_candidate_in_jst,
    ))
}

fn extract_candidate_expression_in_japanese(
    key_to_candidate_in_jst: &str,
    metadata: &Metadata,
) -> Result<String, ErrResp> {
    let candidate_in_jst = match metadata.get(key_to_candidate_in_jst) {
        Some(candidate_in_jst) => candidate_in_jst,
        None => {
            error!(
                "no value for \"{}\" found in metadata",
                key_to_candidate_in_jst
            );
            return Err(unexpected_err_resp());
        }
    };
    let candidate_in_jst =
        DateTime::parse_from_rfc3339(candidate_in_jst.as_str()).map_err(|e| {
            error!(
                "failed to parse \"{}\"\" as RFC3339: {}",
                candidate_in_jst, e
            );
            unexpected_err_resp()
        })?;
    let year = candidate_in_jst.year();
    let month = candidate_in_jst.month();
    let day = candidate_in_jst.day();
    let hour = candidate_in_jst.hour();
    Ok(format!("{}年 {}月 {}日 {}時00分", year, month, day, hour))
}

#[async_trait]
trait FinishRequestConsultationOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;
    async fn get_charge_by_charge_id(&self, charge_id: String) -> Result<Charge, ErrResp>;
    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp>;
    async fn create_request_consultation(
        &self,
        account_id: i64,
        consultant_id: i64,
        charge_id: String,
        expiry_date_time: DateTime<FixedOffset>,
    ) -> Result<Charge, ErrResp>;
    async fn get_user_account_email_address_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<String, ErrResp>;
    async fn get_consultant_email_address_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<String, ErrResp>;
}

struct FinishRequestConsultationOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl FinishRequestConsultationOperation for FinishRequestConsultationOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn get_charge_by_charge_id(&self, charge_id: String) -> Result<Charge, ErrResp> {
        let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
        let charge = charge_op
            .ge_charge_by_charge_id(charge_id.as_str())
            .await
            .map_err(|e| {
                // TODO: https://pay.jp/docs/api/#error に基づいてハンドリングする
                error!("failed to get charge by charge_id ({}): {}", charge_id, e);
                unexpected_err_resp()
            })?;
        Ok(charge)
    }

    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp> {
        util::check_if_consultant_is_available(&self.pool, consultant_id).await
    }

    async fn create_request_consultation(
        &self,
        account_id: i64,
        consultant_id: i64,
        charge_id: String,
        // TODO: latest_candidate_date_timeに変更
        expiry_date_time: DateTime<FixedOffset>,
    ) -> Result<Charge, ErrResp> {
        let charge = self.pool.transaction::<_, Charge, ErrRespStruct>(|txn| {
            Box::pin(async move {
                let active_model = entity::consultation_req::ActiveModel {
                    consultation_req_id: NotSet,
                    user_account_id: Set(account_id),
                    consultant_id: Set(consultant_id),
                    charge_id: Set(charge_id.clone()),
                    latest_candidate_date_time: Set(expiry_date_time),
                };
                active_model.insert(txn).await.map_err(|e| {
                    error!(
                        "failed to insert consultation_req (account_id: {}, consultant_id: {}, charge_id: {}, expiry_date_time: {}): {}",
                        account_id, consultant_id, charge_id.clone(), expiry_date_time, e
                    );
                    ErrRespStruct {
                        err_resp: unexpected_err_resp(),
                    }
                })?;

                let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
                let charge = charge_op.finish_three_d_secure_flow(charge_id.as_str())
                    .await.map_err(|e| {
                        // TODO: https://pay.jp/docs/api/#error に基づいてハンドリングする
                        error!("failed to finish 3D secure flow (charge_id: {}): {}", charge_id, e);
                        ErrRespStruct {
                            err_resp: unexpected_err_resp(),
                        }
                    })?;

                Ok(charge)
            })
        }).await.map_err(|e| match e {
            TransactionError::Connection(db_err) => {
                error!("connection error: {}", db_err);
                unexpected_err_resp()
            }
            TransactionError::Transaction(err_resp_struct) => {
                error!("failed to create_request_consultation: {}", err_resp_struct);
                err_resp_struct.err_resp
            }
        })?;
        Ok(charge)
    }

    async fn get_user_account_email_address_by_user_account_id(
        &self,
        user_account_id: i64,
    ) -> Result<String, ErrResp> {
        let model_option = user_account::Entity::find_by_id(user_account_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find user_account (user_account_id: {}): {}",
                    user_account_id, e
                );
                unexpected_err_resp()
            })?;
        let model = match model_option {
            Some(m) => m,
            None => {
                error!("No user found");
                return Err(unexpected_err_resp());
            }
        };
        Ok(model.email_address)
    }

    async fn get_consultant_email_address_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<String, ErrResp> {
        let model_option = user_account::Entity::find_by_id(consultant_id)
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

#[cfg(test)]
mod tests {
    // TODO
}
