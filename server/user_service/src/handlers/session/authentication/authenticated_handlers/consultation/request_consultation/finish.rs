// Copyright 2022 Ken Miura
// TODO: 最終確認後削除

use axum::async_trait;
use axum::http::StatusCode;
use axum::{extract::State, Json};
use chrono::{DateTime, Datelike, FixedOffset, TimeZone, Timelike};
use common::payment_platform::charge::{Charge, ChargeOperation, ChargeOperationImpl};
use common::payment_platform::Metadata;
use common::smtp::{SendMail, SmtpClient, INQUIRY_EMAIL_ADDRESS, SYSTEM_EMAIL_ADDRESS};
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

use super::{
    KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ, KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
    KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ, KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
};
use crate::err::{unexpected_err_resp, Code};
use crate::handlers::session::authentication::authenticated_handlers::payment_platform::ACCESS_INFO;
use crate::handlers::session::authentication::authenticated_handlers::authenticated_users::verified_user::VerifiedUser;
use crate::handlers::session::authentication::authenticated_handlers::consultation::convert_payment_err::convert_payment_err_to_err_resp;
use crate::handlers::session::authentication::authenticated_handlers::payment_platform::PLATFORM_FEE_RATE_IN_PERCENTAGE;
use crate::handlers::session::authentication::user_operation::{FindUserInfoOperationImpl};
use crate::optional_env_var::MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS;

static CONSULTANT_MAIL_SUBJECT: Lazy<String> =
    Lazy::new(|| format!("[{}] 相談申し込み通知", WEB_SITE_NAME));
static USER_ACCOUNT_MAIL_SUBJECT: Lazy<String> =
    Lazy::new(|| format!("[{}] 相談申し込み完了通知", WEB_SITE_NAME));

static MIN_DURATION_IN_HOURS_BEFORE_CONSULTATION_ACCEPTANCE: Lazy<u32> =
    Lazy::new(|| *MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS / 3600);

pub(crate) async fn post_finish_request_consultation(
    VerifiedUser { user_info }: VerifiedUser,
    State(smtp_client): State<SmtpClient>,
    State(pool): State<DatabaseConnection>,
    Json(param): Json<FinishRequestConsultationParam>,
) -> RespResult<FinishRequestConsultationResult> {
    todo!()
    // let charge_id = param.charge_id;
    // let op = FinishRequestConsultationOperationImpl { pool };
    // handle_finish_request_consultation(
    //     user_info.account_id,
    //     user_info.email_address,
    //     charge_id,
    //     op,
    //     smtp_client,
    // )
    // .await
}

#[derive(Deserialize)]
pub(crate) struct FinishRequestConsultationParam {
    charge_id: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct FinishRequestConsultationResult {}

// #[derive(Debug, PartialEq)]
// struct Candidates {
//     first_candidate_in_jst: DateTime<FixedOffset>,
//     second_candidate_in_jst: DateTime<FixedOffset>,
//     third_candidate_in_jst: DateTime<FixedOffset>,
// }

// async fn handle_finish_request_consultation(
//     user_account_id: i64,
//     user_email_address: String,
//     charge_id: String,
//     op: impl FinishRequestConsultationOperation,
//     send_mail: impl SendMail,
// ) -> RespResult<FinishRequestConsultationResult> {
//     let charge = op.get_charge_by_charge_id(charge_id.clone()).await?;
//     let consultant_id = extract_consultant_id(&charge)?;
//     // 操作者（ユーザー）のアカウントが無効化されているかどうかは個々のURLを示すハンドラに来る前の共通箇所でチェックする
//     // 従って、アカウントが無効化されているかどうかは相談申し込みの相手のみ確認する
//     validate_consultant_is_available(consultant_id, &op).await?;
//     confirm_three_d_secure_status_is_ok(&charge)?;

//     let candidates_date_time_in_jst = extract_candidates_date_time_in_jst(&charge)?;
//     let latest_candidate_date_time_in_jst =
//         extract_latest_candidate_date_time_in_jst(&candidates_date_time_in_jst)?;
//     let id_and_charge = op
//         .create_request_consultation(
//             user_account_id,
//             consultant_id,
//             candidates_date_time_in_jst,
//             latest_candidate_date_time_in_jst,
//             charge.id,
//         )
//         .await?;
//     let consultation_req_id = id_and_charge.0;
//     let charge = id_and_charge.1;
//     info!(
//         "finished 3D Secure flow (consultation_req_id: {}, charge.id: {})",
//         consultation_req_id, charge.id
//     );

//     let consultant_email_address = op
//         .get_consultant_email_address_by_consultant_id(consultant_id)
//         .await?;
//     send_mail_to_consultant(
//         consultant_email_address.as_str(),
//         user_account_id,
//         consultation_req_id,
//         &charge,
//         &send_mail,
//     )
//     .await?;

//     send_mail_to_user(
//         user_email_address.as_str(),
//         consultant_id,
//         consultation_req_id,
//         &charge,
//         &send_mail,
//     )
//     .await?;

//     Ok((StatusCode::OK, Json(FinishRequestConsultationResult {})))
// }

// fn extract_consultant_id(charge: &Charge) -> Result<i64, ErrResp> {
//     let metadata = match charge.metadata.clone() {
//         Some(metadata) => metadata,
//         None => {
//             error!("no metadata found on charge (id: {})", charge.id);
//             return Err(unexpected_err_resp());
//         }
//     };
//     let consultant_id = match metadata.get(KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ) {
//         Some(c_id) => c_id,
//         None => {
//             error!(
//                 "no consultant_id found in metadata on charge (id: {})",
//                 charge.id
//             );
//             return Err(unexpected_err_resp());
//         }
//     };
//     let consultant_id = match consultant_id.parse::<i64>() {
//         Ok(c_id) => c_id,
//         Err(e) => {
//             error!("failed to parse consultant_id in metadata on charge (id: {}, consultant_id: {}): {}", charge.id, consultant_id, e);
//             return Err(unexpected_err_resp());
//         }
//     };
//     Ok(consultant_id)
// }

// async fn validate_consultant_is_available(
//     consultant_id: i64,
//     op: &impl FinishRequestConsultationOperation,
// ) -> Result<(), ErrResp> {
//     let consultant_available = op.check_if_consultant_is_available(consultant_id).await?;
//     if !consultant_available {
//         error!(
//             "consultant is not available (consultant_id: {})",
//             consultant_id
//         );
//         return Err((
//             StatusCode::BAD_REQUEST,
//             Json(ApiError {
//                 code: Code::ConsultantIsNotAvailable as u32,
//             }),
//         ));
//     }
//     Ok(())
// }

// fn confirm_three_d_secure_status_is_ok(charge: &Charge) -> Result<(), ErrResp> {
//     let three_d_secure_status = match charge.three_d_secure_status.clone() {
//         Some(s) => s,
//         None => {
//             error!(
//                 "three_d_secure_status is None (charge.id: {})",
//                 charge.id.clone()
//             );
//             return Err(unexpected_err_resp());
//         }
//     };
//     if three_d_secure_status != "verified" {
//         error!(
//             "3D secure is not finished correctly (three_d_secure_status: {}, charge.id: {})",
//             three_d_secure_status,
//             charge.id.clone()
//         );
//         return Err((
//             StatusCode::BAD_REQUEST,
//             Json(ApiError {
//                 code: Code::ThreeDSecureError as u32,
//             }),
//         ));
//     }
//     Ok(())
// }

// fn extract_candidates_date_time_in_jst(charge: &Charge) -> Result<Candidates, ErrResp> {
//     let metadata = match charge.metadata.clone() {
//         Some(m) => m,
//         None => {
//             error!("no metadata found (charge.id: {})", charge.id);
//             return Err(unexpected_err_resp());
//         }
//     };

//     let first_candidate_in_jst = extract_candidate_as_date_time(
//         &metadata,
//         KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
//         charge.id.as_str(),
//     )?;
//     let second_candidate_in_jst = extract_candidate_as_date_time(
//         &metadata,
//         KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
//         charge.id.as_str(),
//     )?;
//     let third_candidate_in_jst = extract_candidate_as_date_time(
//         &metadata,
//         KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
//         charge.id.as_str(),
//     )?;

//     Ok(Candidates {
//         first_candidate_in_jst,
//         second_candidate_in_jst,
//         third_candidate_in_jst,
//     })
// }

// fn extract_candidate_as_date_time(
//     metadata: &Metadata,
//     key: &str,
//     charge_id: &str,
// ) -> Result<DateTime<FixedOffset>, ErrResp> {
//     let candidate_in_jst = match metadata.get(key) {
//         Some(date_time_str) => date_time_str,
//         None => {
//             error!("no {} found on metadata (charge_id: {})", key, charge_id);
//             return Err(unexpected_err_resp());
//         }
//     };
//     let candidate_in_jst = DateTime::<FixedOffset>::parse_from_rfc3339(candidate_in_jst.as_str())
//         .map_err(|e| {
//         error!("failed to parse {} as RFC3339: {}", candidate_in_jst, e);
//         unexpected_err_resp()
//     })?;
//     Ok(candidate_in_jst)
// }

// fn extract_latest_candidate_date_time_in_jst(
//     candidates: &Candidates,
// ) -> Result<DateTime<FixedOffset>, ErrResp> {
//     let candidates_in_jst = vec![
//         candidates.second_candidate_in_jst,
//         candidates.third_candidate_in_jst,
//     ];
//     let latest_candidate_in_jst =
//         select_latest_candidate_in_jst(candidates.first_candidate_in_jst, candidates_in_jst);
//     Ok(latest_candidate_in_jst)
// }

// fn select_latest_candidate_in_jst(
//     first_candidate_in_jst: DateTime<FixedOffset>,
//     candidates_in_jst: Vec<DateTime<FixedOffset>>,
// ) -> DateTime<FixedOffset> {
//     let mut latest_candidate_in_jst = first_candidate_in_jst;
//     for c in candidates_in_jst.iter() {
//         if c > &latest_candidate_in_jst {
//             latest_candidate_in_jst = *c
//         }
//     }
//     latest_candidate_in_jst
// }

// async fn send_mail_to_consultant(
//     consultant_email_address: &str,
//     user_account_id: i64,
//     consultation_req_id: i64,
//     charge: &Charge,
//     send_mail: &impl SendMail,
// ) -> Result<(), ErrResp> {
//     let candidates = extract_candidates(charge)?;
//     let text = create_text_for_consultant_mail(user_account_id, consultation_req_id, &candidates);
//     send_mail
//         .send_mail(
//             consultant_email_address,
//             SYSTEM_EMAIL_ADDRESS.as_str(),
//             CONSULTANT_MAIL_SUBJECT.as_str(),
//             text.as_str(),
//         )
//         .await?;
//     Ok(())
// }

// fn create_text_for_consultant_mail(
//     user_account_id: i64,
//     consultation_req_id: i64,
//     candidates: &(String, String, String),
// ) -> String {
//     // TODO: 文面の調整
//     format!(
//         r"ユーザーID ({}) から相談申し込み（相談申し込み番号: {}）が届きました。相談者からの希望相談開始日時を下記に記載します。{}へログインし、相談受け付けのページから該当の申し込みの詳細を確認し、了承する、または拒否するをご選択下さい。

// 希望相談開始日時
//   第一希望: {}
//   第二希望: {}
//   第三希望: {}

// 各希望相談開始日時について、その日時の{}時間前となると、その日時を選択して了承することができなくなりますのでご注意下さい。

// 本メールはシステムより自動配信されています。
// 本メールに返信されましても、回答いたしかねます。
// お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

// 【お問い合わせ先】
// Email: {}",
//         user_account_id,
//         consultation_req_id,
//         WEB_SITE_NAME,
//         candidates.0,
//         candidates.1,
//         candidates.2,
//         *MIN_DURATION_IN_HOURS_BEFORE_CONSULTATION_ACCEPTANCE,
//         INQUIRY_EMAIL_ADDRESS.as_str()
//     )
// }

// async fn send_mail_to_user(
//     user_account_email_address: &str,
//     consultant_id: i64,
//     consultation_req_id: i64,
//     charge: &Charge,
//     send_mail: &impl SendMail,
// ) -> Result<(), ErrResp> {
//     let candidates = extract_candidates(charge)?;
//     let text = create_text_for_user_mail(
//         consultant_id,
//         consultation_req_id,
//         charge.amount,
//         &candidates,
//     );
//     send_mail
//         .send_mail(
//             user_account_email_address,
//             SYSTEM_EMAIL_ADDRESS.as_str(),
//             USER_ACCOUNT_MAIL_SUBJECT.as_str(),
//             text.as_str(),
//         )
//         .await?;
//     Ok(())
// }

// fn create_text_for_user_mail(
//     consultant_id: i64,
//     consultation_req_id: i64,
//     amount: i32,
//     candidates: &(String, String, String),
// ) -> String {
//     // TODO: 文面の調整
//     format!(
//         r"下記の内容で相談申し込み（相談申し込み番号: {}）を行いました。

// 相談相手
//   コンサルタントID: {}

// 相談料金
//   {} 円

// 希望相談開始日時
//   第一希望: {}
//   第二希望: {}
//   第三希望: {}

// 相談申し込みが拒否されていない限り、希望相談開始日時の{}時間前までは、コンサルタントの相談申し込みに対する了承の可能性があります。相談申し込みが了承されたことを見逃さないために、各希望相談開始日時の{}時間前には{}にログイン後、スケジュールのページをご確認下さい。

// 本メールはシステムより自動配信されています。
// 本メールに返信されましても、回答いたしかねます。
// お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

// 【お問い合わせ先】
// Email: {}",
//         consultation_req_id,
//         consultant_id,
//         amount,
//         candidates.0,
//         candidates.1,
//         candidates.2,
//         *MIN_DURATION_IN_HOURS_BEFORE_CONSULTATION_ACCEPTANCE,
//         *MIN_DURATION_IN_HOURS_BEFORE_CONSULTATION_ACCEPTANCE,
//         WEB_SITE_NAME,
//         INQUIRY_EMAIL_ADDRESS.as_str()
//     )
// }

// fn extract_candidates(charge: &Charge) -> Result<(String, String, String), ErrResp> {
//     let metadata = match charge.metadata.clone() {
//         Some(metadata) => metadata,
//         None => {
//             error!("no metadata found on charge (id: {})", charge.id);
//             return Err(unexpected_err_resp());
//         }
//     };
//     let first_candidate_in_jst = extract_candidate_expression_in_japanese(
//         KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
//         &metadata,
//     )?;
//     let second_candidate_in_jst = extract_candidate_expression_in_japanese(
//         KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
//         &metadata,
//     )?;
//     let third_candidate_in_jst = extract_candidate_expression_in_japanese(
//         KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
//         &metadata,
//     )?;
//     Ok((
//         first_candidate_in_jst,
//         second_candidate_in_jst,
//         third_candidate_in_jst,
//     ))
// }

// fn extract_candidate_expression_in_japanese(
//     key_to_candidate_in_jst: &str,
//     metadata: &Metadata,
// ) -> Result<String, ErrResp> {
//     let candidate_in_jst = match metadata.get(key_to_candidate_in_jst) {
//         Some(candidate_in_jst) => candidate_in_jst,
//         None => {
//             error!(
//                 "no value for \"{}\" found in metadata",
//                 key_to_candidate_in_jst
//             );
//             return Err(unexpected_err_resp());
//         }
//     };
//     let candidate_in_jst =
//         DateTime::parse_from_rfc3339(candidate_in_jst.as_str()).map_err(|e| {
//             error!(
//                 "failed to parse \"{}\"\" as RFC3339: {}",
//                 candidate_in_jst, e
//             );
//             unexpected_err_resp()
//         })?;
//     let year = candidate_in_jst.year();
//     let month = candidate_in_jst.month();
//     let day = candidate_in_jst.day();
//     let hour = candidate_in_jst.hour();
//     Ok(format!("{}年 {}月 {}日 {}時00分", year, month, day, hour))
// }

// #[async_trait]
// trait FinishRequestConsultationOperation {
//     async fn get_charge_by_charge_id(&self, charge_id: String) -> Result<Charge, ErrResp>;

//     async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp>;

//     async fn create_request_consultation(
//         &self,
//         account_id: i64,
//         consultant_id: i64,
//         candidates: Candidates,
//         latest_candidate_date_time_in_jst: DateTime<FixedOffset>,
//         charge_id: String,
//     ) -> Result<(i64, Charge), ErrResp>;

//     async fn get_consultant_email_address_by_consultant_id(
//         &self,
//         consultant_id: i64,
//     ) -> Result<String, ErrResp>;
// }

// struct FinishRequestConsultationOperationImpl {
//     pool: DatabaseConnection,
// }

// #[async_trait]
// impl FinishRequestConsultationOperation for FinishRequestConsultationOperationImpl {
//     async fn get_charge_by_charge_id(&self, charge_id: String) -> Result<Charge, ErrResp> {
//         let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
//         let charge = charge_op
//             .ge_charge_by_charge_id(charge_id.as_str())
//             .await
//             .map_err(|e| {
//                 error!("failed to get charge by charge_id ({}): {}", charge_id, e);
//                 convert_payment_err_to_err_resp(&e)
//             })?;
//         Ok(charge)
//     }

//     async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp> {
//         let op = FindUserInfoOperationImpl::new(&self.pool);
//         super::super::check_if_consultant_is_available(consultant_id, &op).await
//     }

//     async fn create_request_consultation(
//         &self,
//         account_id: i64,
//         consultant_id: i64,
//         candidates: Candidates,
//         latest_candidate_date_time_in_jst: DateTime<FixedOffset>,
//         charge_id: String,
//     ) -> Result<(i64, Charge), ErrResp> {
//         let id_and_charge = self.pool.transaction::<_, (i64, Charge), ErrRespStruct>(|txn| {
//             Box::pin(async move {
//                 // 適切にロールバックが可能なように、通常はDB操作を行った後、最後にその他の失敗する可能性のある処理を行う。
//                 // しかし、ここでは先にDB操作以外の処理（PAYJP APIの呼び出し）を行い、その後にDB操作を行うように記載している。
//                 // これはDBに入れる値であるexpired_atがPAYJP APIの呼び出し以降出ない限り、手に入らないため。
//                 // 万が一、PAYJP APIの呼び出しが成功した後、DB操作が失敗してロールバックした場合、PAYJP上に余計なChargeが残ったままになる。
//                 // しかし、このChargeは、captured=falseである（=与信枠の確保のみで支払いは確定していない）ため、大きな問題とならないと考えられる。
//                 let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
//                 let charge = charge_op.finish_three_d_secure_flow(charge_id.as_str())
//                     .await
//                     .map_err(|e| {
//                         error!("failed to finish 3D secure flow (charge_id: {}): {}", charge_id, e);
//                         ErrRespStruct {
//                             err_resp: convert_payment_err_to_err_resp(&e),
//                         }
//                     })?;

//                 let expired_at_timestamp = charge.expired_at.ok_or_else(|| {
//                     error!("failed to get expired_at (charge_id: {})", charge_id);
//                     ErrRespStruct {
//                         err_resp: unexpected_err_resp()
//                     }
//                 })?;
//                 // 日本のタイムゾーンにおいて、（サマータイム等による）タイムゾーンの遷移は発生しないので一意にならない場合はすべてエラー
//                 let expired_at = match JAPANESE_TIME_ZONE.timestamp_opt(expired_at_timestamp, 0) {
//                     chrono::LocalResult::None => {
//                         error!("failed to get expired_at (expired_at_timestamp: {})", expired_at_timestamp);
//                         return Err(ErrRespStruct {
//                             err_resp: unexpected_err_resp()
//                         });
//                     },
//                     chrono::LocalResult::Single(s) => s,
//                     chrono::LocalResult::Ambiguous(a1, a2) => {
//                         error!("failed to get expired_at (expired_at_timestamp: {}, ambiguous1: {}, ambiguous2: {})", expired_at_timestamp, a1, a2);
//                         return Err(ErrRespStruct {
//                             err_resp: unexpected_err_resp()
//                         });
//                     },
//                 };
//                 let active_model = entity::consultation_req::ActiveModel {
//                     consultation_req_id: NotSet,
//                     user_account_id: Set(account_id),
//                     consultant_id: Set(consultant_id),
//                     first_candidate_date_time: Set(candidates.first_candidate_in_jst),
//                     second_candidate_date_time: Set(candidates.second_candidate_in_jst),
//                     third_candidate_date_time: Set(candidates.third_candidate_in_jst),
//                     latest_candidate_date_time: Set(latest_candidate_date_time_in_jst),
//                     charge_id: Set(charge.id.clone()),
//                     fee_per_hour_in_yen: Set(charge.amount), // 3Dセキュアフロー内の一連の流れであり、途中に返金処理が発生していることはない。従ってcharge.amount_refundedを考慮する必要はない
//                     platform_fee_rate_in_percentage: Set(PLATFORM_FEE_RATE_IN_PERCENTAGE.to_string()), // platform_fee_rateは、captured=trueの状態にならない限り手に入らないため、（テナントに設定してある）想定値を使う
//                     credit_facilities_expired_at: Set(expired_at)
//                 };
//                 let result = active_model.insert(txn).await.map_err(|e| {
//                     error!(
//                         "failed to insert consultation_req (account_id: {}, consultant_id: {}, charge.id: {}, latest_candidate_date_time_in_jst: {}): {}",
//                         account_id, consultant_id, charge.id.clone(), latest_candidate_date_time_in_jst, e
//                     );
//                     ErrRespStruct {
//                         err_resp: unexpected_err_resp(),
//                     }
//                 })?;

//                 Ok((result.consultation_req_id, charge))
//             })
//         }).await.map_err(|e| match e {
//             TransactionError::Connection(db_err) => {
//                 error!("connection error: {}", db_err);
//                 unexpected_err_resp()
//             }
//             TransactionError::Transaction(err_resp_struct) => {
//                 error!("failed to create_request_consultation: {}", err_resp_struct);
//                 err_resp_struct.err_resp
//             }
//         })?;
//         Ok((id_and_charge.0, id_and_charge.1))
//     }

//     async fn get_consultant_email_address_by_consultant_id(
//         &self,
//         consultant_id: i64,
//     ) -> Result<String, ErrResp> {
//         let model_option = user_account::Entity::find_by_id(consultant_id)
//             .one(&self.pool)
//             .await
//             .map_err(|e| {
//                 error!(
//                     "failed to find user_account (consultant_id: {}): {}",
//                     consultant_id, e
//                 );
//                 unexpected_err_resp()
//             })?;
//         let model = match model_option {
//             Some(m) => m,
//             None => {
//                 error!("No consultant found");
//                 return Err(unexpected_err_resp());
//             }
//         };
//         Ok(model.email_address)
//     }
// }

// #[cfg(test)]
// mod tests {

//     use chrono::{Duration, Utc};
//     use common::payment_platform::customer::Card;

//     use crate::optional_env_var::EXPIRY_DAYS_OF_CHARGE;

//     use super::*;

//     #[derive(Debug)]
//     struct TestCase {
//         name: String,
//         input: Input,
//         expected: RespResult<FinishRequestConsultationResult>,
//     }

//     #[derive(Debug)]
//     struct Input {
//         user_account_id: i64,
//         user_email_address: String,
//         charge_id: String,
//         op: FinishRequestConsultationOperationMock,
//         smtp_client: SendMailMock,
//     }

//     #[derive(Clone, Debug)]
//     struct FinishRequestConsultationOperationMock {
//         account_id: i64,
//         charge_id: String,
//         charge: Charge,
//         consultant_id: i64,
//         latest_candidate_date_time_in_jst: DateTime<FixedOffset>,
//         consultant_email_address: String,
//     }

//     #[async_trait]
//     impl FinishRequestConsultationOperation for FinishRequestConsultationOperationMock {
//         async fn get_charge_by_charge_id(&self, charge_id: String) -> Result<Charge, ErrResp> {
//             assert_eq!(self.charge_id, charge_id);
//             Ok(self.charge.clone())
//         }

//         async fn check_if_consultant_is_available(
//             &self,
//             consultant_id: i64,
//         ) -> Result<bool, ErrResp> {
//             if self.consultant_id != consultant_id {
//                 return Ok(false);
//             };
//             Ok(true)
//         }

//         async fn create_request_consultation(
//             &self,
//             account_id: i64,
//             consultant_id: i64,
//             candidates: Candidates,
//             latest_candidate_date_time_in_jst: DateTime<FixedOffset>,
//             charge_id: String,
//         ) -> Result<(i64, Charge), ErrResp> {
//             assert_eq!(self.account_id, account_id);
//             assert_eq!(self.consultant_id, consultant_id);
//             assert_eq!(self.charge.id, charge_id);
//             let c = extract_candidates_date_time_in_jst(&self.charge).expect("failed to get Ok");
//             assert_eq!(c, candidates);
//             assert_eq!(
//                 self.latest_candidate_date_time_in_jst,
//                 latest_candidate_date_time_in_jst
//             );
//             // captured_at、expired_atは使わかないので用意する必要はないが、
//             // 本番環境ではセットされてくるので、それに合わせたデータを用意してく
//             let mut charge = self.charge.clone();
//             let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
//             charge.captured_at = Some(current_date_time.timestamp());
//             let expired_at = current_date_time + Duration::days((*EXPIRY_DAYS_OF_CHARGE) as i64);
//             charge.expired_at = Some(expired_at.timestamp());
//             Ok((1, charge))
//         }

//         async fn get_consultant_email_address_by_consultant_id(
//             &self,
//             consultant_id: i64,
//         ) -> Result<String, ErrResp> {
//             assert_eq!(self.consultant_id, consultant_id);
//             Ok(self.consultant_email_address.clone())
//         }
//     }

//     #[derive(Clone, Debug)]
//     struct SendMailMock {}

//     #[async_trait]
//     impl SendMail for SendMailMock {
//         async fn send_mail(
//             &self,
//             _to: &str,
//             _from: &str,
//             _subject: &str,
//             _text: &str,
//         ) -> Result<(), ErrResp> {
//             Ok(())
//         }
//     }

//     static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
//         let user_email_address = "test0@test.com";
//         vec![
//             TestCase {
//                 name: "success case 1 (3D secure status verified)".to_string(),
//                 input: Input {
//                     user_account_id: 1,
//                     user_email_address: user_email_address.to_string(),
//                     charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
//                     op: FinishRequestConsultationOperationMock {
//                         account_id: 1,
//                         charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
//                         charge: create_dummy_charge(
//                             "ch_fa990a4c10672a93053a774730b0a",
//                             5000,
//                             "verified",
//                             create_metadata(
//                                 2,
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
//                                     .unwrap(),
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 4, 23, 0, 0)
//                                     .unwrap(),
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
//                                     .unwrap(),
//                             ),
//                         ),
//                         consultant_id: 2,
//                         latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
//                             .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
//                             .unwrap(),
//                         consultant_email_address: "test1@test.com".to_string(),
//                     },
//                     smtp_client: SendMailMock {},
//                 },
//                 expected: Ok((StatusCode::OK, Json(FinishRequestConsultationResult {}))),
//             },
//             TestCase {
//                 name: "fail ThreeDSecureError (3D secure status attempted)".to_string(),
//                 input: Input {
//                     user_account_id: 1,
//                     user_email_address: user_email_address.to_string(),
//                     charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
//                     op: FinishRequestConsultationOperationMock {
//                         account_id: 1,
//                         charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
//                         charge: create_dummy_charge(
//                             "ch_fa990a4c10672a93053a774730b0a",
//                             5000,
//                             "attempted",
//                             create_metadata(
//                                 2,
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
//                                     .unwrap(),
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 4, 23, 0, 0)
//                                     .unwrap(),
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
//                                     .unwrap(),
//                             ),
//                         ),
//                         consultant_id: 2,
//                         latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
//                             .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
//                             .unwrap(),
//                         consultant_email_address: "test1@test.com".to_string(),
//                     },
//                     smtp_client: SendMailMock {},
//                 },
//                 expected: Err((
//                     StatusCode::BAD_REQUEST,
//                     Json(ApiError {
//                         code: Code::ThreeDSecureError as u32,
//                     }),
//                 )),
//             },
//             TestCase {
//                 name: "fail ConsultantIsNotAvailable".to_string(),
//                 input: Input {
//                     user_account_id: 1,
//                     user_email_address: user_email_address.to_string(),
//                     charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
//                     op: FinishRequestConsultationOperationMock {
//                         account_id: 1,
//                         charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
//                         charge: create_dummy_charge(
//                             "ch_fa990a4c10672a93053a774730b0a",
//                             5000,
//                             "verified",
//                             create_metadata(
//                                 3,
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
//                                     .unwrap(),
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 4, 23, 0, 0)
//                                     .unwrap(),
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
//                                     .unwrap(),
//                             ),
//                         ),
//                         consultant_id: 2,
//                         latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
//                             .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
//                             .unwrap(),
//                         consultant_email_address: "test1@test.com".to_string(),
//                     },
//                     smtp_client: SendMailMock {},
//                 },
//                 expected: Err((
//                     StatusCode::BAD_REQUEST,
//                     Json(ApiError {
//                         code: Code::ConsultantIsNotAvailable as u32,
//                     }),
//                 )),
//             },
//             TestCase {
//                 name: "fail ThreeDSecureError unverified".to_string(),
//                 input: Input {
//                     user_account_id: 1,
//                     user_email_address: user_email_address.to_string(),
//                     charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
//                     op: FinishRequestConsultationOperationMock {
//                         account_id: 1,
//                         charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
//                         charge: create_dummy_charge(
//                             "ch_fa990a4c10672a93053a774730b0a",
//                             5000,
//                             "unverified",
//                             create_metadata(
//                                 2,
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
//                                     .unwrap(),
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 4, 23, 0, 0)
//                                     .unwrap(),
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
//                                     .unwrap(),
//                             ),
//                         ),
//                         consultant_id: 2,
//                         latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
//                             .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
//                             .unwrap(),
//                         consultant_email_address: "test1@test.com".to_string(),
//                     },
//                     smtp_client: SendMailMock {},
//                 },
//                 expected: Err((
//                     StatusCode::BAD_REQUEST,
//                     Json(ApiError {
//                         code: Code::ThreeDSecureError as u32,
//                     }),
//                 )),
//             },
//             TestCase {
//                 name: "fail ThreeDSecureError failed".to_string(),
//                 input: Input {
//                     user_account_id: 1,
//                     user_email_address: user_email_address.to_string(),
//                     charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
//                     op: FinishRequestConsultationOperationMock {
//                         account_id: 1,
//                         charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
//                         charge: create_dummy_charge(
//                             "ch_fa990a4c10672a93053a774730b0a",
//                             5000,
//                             "failed",
//                             create_metadata(
//                                 2,
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
//                                     .unwrap(),
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 4, 23, 0, 0)
//                                     .unwrap(),
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
//                                     .unwrap(),
//                             ),
//                         ),
//                         consultant_id: 2,
//                         latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
//                             .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
//                             .unwrap(),
//                         consultant_email_address: "test1@test.com".to_string(),
//                     },
//                     smtp_client: SendMailMock {},
//                 },
//                 expected: Err((
//                     StatusCode::BAD_REQUEST,
//                     Json(ApiError {
//                         code: Code::ThreeDSecureError as u32,
//                     }),
//                 )),
//             },
//             TestCase {
//                 name: "fail ThreeDSecureError error".to_string(),
//                 input: Input {
//                     user_account_id: 1,
//                     user_email_address: user_email_address.to_string(),
//                     charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
//                     op: FinishRequestConsultationOperationMock {
//                         account_id: 1,
//                         charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
//                         charge: create_dummy_charge(
//                             "ch_fa990a4c10672a93053a774730b0a",
//                             5000,
//                             "error",
//                             create_metadata(
//                                 2,
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 4, 7, 0, 0)
//                                     .unwrap(),
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 4, 23, 0, 0)
//                                     .unwrap(),
//                                 JAPANESE_TIME_ZONE
//                                     .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
//                                     .unwrap(),
//                             ),
//                         ),
//                         consultant_id: 2,
//                         latest_candidate_date_time_in_jst: JAPANESE_TIME_ZONE
//                             .with_ymd_and_hms(2022, 11, 22, 7, 0, 0)
//                             .unwrap(),
//                         consultant_email_address: "test1@test.com".to_string(),
//                     },
//                     smtp_client: SendMailMock {},
//                 },
//                 expected: Err((
//                     StatusCode::BAD_REQUEST,
//                     Json(ApiError {
//                         code: Code::ThreeDSecureError as u32,
//                     }),
//                 )),
//             },
//         ]
//     });

//     // create_dummy_chargeでAPI呼び出しの結果返却されるChargeを作成する
//     // 返却されたChargeはパラメータで指定した値だけ利用し、他を参照することはないのでパラメータの値以外はダミーの関係ない値で埋めてある
//     fn create_dummy_charge(
//         id: &str,
//         amount: i32,
//         three_d_secure_status: &str,
//         metadata: Metadata,
//     ) -> Charge {
//         Charge {
//             id: id.to_string(),
//             object: "charge".to_string(),
//             livemode: false,
//             created: 1639931415,
//             amount,
//             currency: "jpy".to_string(),
//             paid: true,
//             expired_at: None,
//             captured: false,
//             captured_at: Some(1639931415),
//             card: Some(Card {
//                 object: "card".to_string(),
//                 id: "car_33ab04bcdc00f0cc6d6df16bbe79".to_string(),
//                 created: 1639931415,
//                 name: None,
//                 last4: "4242".to_string(),
//                 exp_month: 12,
//                 exp_year: 2022,
//                 brand: "Visa".to_string(),
//                 cvc_check: "passed".to_string(),
//                 fingerprint: "e1d8225886e3a7211127df751c86787f".to_string(),
//                 address_state: None,
//                 address_city: None,
//                 address_line1: None,
//                 address_line2: None,
//                 country: None,
//                 address_zip: None,
//                 address_zip_check: "unchecked".to_string(),
//                 metadata: None,
//             }),
//             customer: None,
//             description: None,
//             failure_code: None,
//             failure_message: None,
//             fee_rate: Some("3.00".to_string()),
//             refunded: false,
//             amount_refunded: 0,
//             refund_reason: None,
//             subscription: None,
//             metadata: Some(metadata),
//             platform_fee: None,
//             tenant: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
//             platform_fee_rate: Some("30.0".to_string()),
//             total_platform_fee: Some(1350),
//             three_d_secure_status: Some(three_d_secure_status.to_string()),
//         }
//     }

//     fn create_metadata(
//         consultant_id: i64,
//         first_candidate_in_jst: DateTime<FixedOffset>,
//         second_candidate_in_jst: DateTime<FixedOffset>,
//         third_candidate_in_jst: DateTime<FixedOffset>,
//     ) -> Metadata {
//         let mut metadata = Metadata::with_capacity(4);

//         let _ = metadata.insert(
//             KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ.to_string(),
//             consultant_id.to_string(),
//         );

//         let _ = metadata.insert(
//             KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ.to_string(),
//             first_candidate_in_jst.to_rfc3339(),
//         );

//         let _ = metadata.insert(
//             KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ.to_string(),
//             second_candidate_in_jst.to_rfc3339(),
//         );

//         let _ = metadata.insert(
//             KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ.to_string(),
//             third_candidate_in_jst.to_rfc3339(),
//         );

//         metadata
//     }

//     #[tokio::test]
//     async fn handle_finish_request_consultation_tests() {
//         for test_case in TEST_CASE_SET.iter() {
//             let user_account_id = test_case.input.user_account_id;
//             let user_email_address = test_case.input.user_email_address.clone();
//             let charge_id = test_case.input.charge_id.clone();
//             let op = test_case.input.op.clone();
//             let smtp_client = test_case.input.smtp_client.clone();

//             let result = handle_finish_request_consultation(
//                 user_account_id,
//                 user_email_address,
//                 charge_id,
//                 op,
//                 smtp_client,
//             )
//             .await;

//             let message = format!("test case \"{}\" failed", test_case.name.clone());
//             if test_case.expected.is_ok() {
//                 let resp = result.expect("failed to get Ok");
//                 let expected = test_case.expected.as_ref().expect("failed to get Ok");
//                 assert_eq!(expected.0, resp.0, "{}", message);
//                 assert_eq!(expected.1 .0, resp.1 .0, "{}", message);
//             } else {
//                 let resp = result.expect_err("failed to get Err");
//                 let expected = test_case.expected.as_ref().expect_err("failed to get Err");
//                 assert_eq!(expected.0, resp.0, "{}", message);
//                 assert_eq!(expected.1 .0, resp.1 .0, "{}", message);
//             }
//         }
//     }

//     #[test]
//     fn test_create_text_for_consultant_mail() {
//         let user_account_id = 1;
//         let consultation_req_id = 1;
//         let first_candidate = "2022年 11月 12日 7時00分";
//         let second_candidate = "2022年 11月 12日 23時00分";
//         let third_candidate = "2022年 11月 22日 7時00分";

//         let result = create_text_for_consultant_mail(
//             user_account_id,
//             consultation_req_id,
//             &(
//                 first_candidate.to_string(),
//                 second_candidate.to_string(),
//                 third_candidate.to_string(),
//             ),
//         );

//         let expected = format!(
//             r"ユーザーID ({}) から相談申し込み（相談申し込み番号: {}）が届きました。相談者からの希望相談開始日時を下記に記載します。{}へログインし、相談受け付けのページから該当の申し込みの詳細を確認し、了承する、または拒否するをご選択下さい。

// 希望相談開始日時
//   第一希望: {}
//   第二希望: {}
//   第三希望: {}

// 各希望相談開始日時について、その日時の{}時間前となると、その日時を選択して了承することができなくなりますのでご注意下さい。

// 本メールはシステムより自動配信されています。
// 本メールに返信されましても、回答いたしかねます。
// お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

// 【お問い合わせ先】
// Email: {}",
//             user_account_id,
//             consultation_req_id,
//             WEB_SITE_NAME,
//             first_candidate,
//             second_candidate,
//             third_candidate,
//             *MIN_DURATION_IN_HOURS_BEFORE_CONSULTATION_ACCEPTANCE,
//             INQUIRY_EMAIL_ADDRESS.as_str()
//         );

//         assert_eq!(result, expected);
//     }

//     #[test]
//     fn test_create_text_for_user_mail() {
//         let consultant_id = 2;
//         let consultation_req_id = 13;
//         let amount = 5000;
//         let first_candidate = "2022年 11月 12日 7時00分";
//         let second_candidate = "2022年 11月 12日 23時00分";
//         let third_candidate = "2022年 11月 22日 7時00分";

//         let result = create_text_for_user_mail(
//             consultant_id,
//             consultation_req_id,
//             amount,
//             &(
//                 first_candidate.to_string(),
//                 second_candidate.to_string(),
//                 third_candidate.to_string(),
//             ),
//         );

//         let expected = format!(
//             r"下記の内容で相談申し込み（相談申し込み番号: {}）を行いました。

// 相談相手
//   コンサルタントID: {}

// 相談料金
//   {} 円

// 希望相談開始日時
//   第一希望: {}
//   第二希望: {}
//   第三希望: {}

// 相談申し込みが拒否されていない限り、希望相談開始日時の{}時間前までは、コンサルタントの相談申し込みに対する了承の可能性があります。相談申し込みが了承されたことを見逃さないために、各希望相談開始日時の{}時間前には{}にログイン後、スケジュールのページをご確認下さい。

// 本メールはシステムより自動配信されています。
// 本メールに返信されましても、回答いたしかねます。
// お問い合わせは、下記のお問い合わせ先までご連絡くださいますようお願いいたします。

// 【お問い合わせ先】
// Email: {}",
//             consultation_req_id,
//             consultant_id,
//             amount,
//             first_candidate,
//             second_candidate,
//             third_candidate,
//             *MIN_DURATION_IN_HOURS_BEFORE_CONSULTATION_ACCEPTANCE,
//             *MIN_DURATION_IN_HOURS_BEFORE_CONSULTATION_ACCEPTANCE,
//             WEB_SITE_NAME,
//             INQUIRY_EMAIL_ADDRESS.as_str()
//         );

//         assert_eq!(result, expected);
//     }
// }
