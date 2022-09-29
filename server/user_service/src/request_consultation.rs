// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use common::payment_platform::charge::CreateCharge;
use common::payment_platform::Metadata;
use common::{
    payment_platform::charge::{ChargeOperation, ChargeOperationImpl},
    ErrResp, RespResult,
};
use common::{ApiError, JAPANESE_TIME_ZONE};
use entity::{
    prelude::ConsultingFee,
    sea_orm::{DatabaseConnection, EntityTrait},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::err::Code;
use crate::util::validator::consultation_date_time_validator::{
    validate_consultation_date_time, ConsultationDateTimeValidationError,
};
use crate::util::{
    KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ, KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
    KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ, KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
};
use crate::{
    err::unexpected_err_resp,
    util::{self, session::User, ACCESS_INFO},
};

const EXPIRY_DAYS: u32 = 7;

pub(crate) async fn post_request_consultation(
    User { account_id }: User,
    Json(param): Json<RequestConsultationParam>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<RequestConsultationResult> {
    let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
    let request_consultation_op = RequestConsultationOperationImpl { pool };
    let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
    handle_request_consultation(
        account_id,
        param,
        &current_date_time,
        request_consultation_op,
        charge_op,
    )
    .await
}

#[derive(Deserialize)]
pub(crate) struct RequestConsultationParam {
    pub consultant_id: i64,
    pub fee_per_hour_in_yen: i32,
    pub card_token: String,
    pub first_candidate_in_jst: ConsultationDateTime,
    pub second_candidate_in_jst: ConsultationDateTime,
    pub third_candidate_in_jst: ConsultationDateTime,
}

#[derive(Deserialize, PartialEq)]
pub(crate) struct ConsultationDateTime {
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) day: u32,
    pub(crate) hour: u32,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct RequestConsultationResult {
    pub charge_id: String,
}

#[async_trait]
trait RequestConsultationOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;

    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp>;

    async fn find_fee_per_hour_in_yen_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Option<i32>, ErrResp>;

    async fn find_tenant_id_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Option<String>, ErrResp>;
}

struct RequestConsultationOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl RequestConsultationOperation for RequestConsultationOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp> {
        util::check_if_consultant_is_available(&self.pool, consultant_id).await
    }

    async fn find_fee_per_hour_in_yen_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Option<i32>, ErrResp> {
        let model = ConsultingFee::find_by_id(consultant_id)
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

    async fn find_tenant_id_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Option<String>, ErrResp> {
        let model = entity::prelude::Tenant::find_by_id(consultant_id)
            .one(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find tenant (consultant_id: {}): {}",
                    consultant_id, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|m| m.tenant_id))
    }
}

async fn handle_request_consultation(
    account_id: i64,
    request_consultation_param: RequestConsultationParam,
    current_date_time: &DateTime<FixedOffset>,
    request_consultation_op: impl RequestConsultationOperation,
    charge_op: impl ChargeOperation,
) -> RespResult<RequestConsultationResult> {
    let consultant_id = request_consultation_param.consultant_id;
    let _ = validate_consultant_id_is_positive(consultant_id)?;
    let _ = validate_candidates(
        &request_consultation_param.first_candidate_in_jst,
        &request_consultation_param.second_candidate_in_jst,
        &request_consultation_param.third_candidate_in_jst,
        current_date_time,
    )?;
    let _ = validate_identity_exists(account_id, &request_consultation_op).await?;
    let _ = validate_consultant_is_available(consultant_id, &request_consultation_op).await?;

    let fee_per_hour_in_yen =
        get_fee_per_hour_in_yen(consultant_id, &request_consultation_op).await?;
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

    let tenant_id = get_tenant_id(consultant_id, &request_consultation_op).await?;

    let price = (fee_per_hour_in_yen, "jpy".to_string());
    let card = request_consultation_param.card_token.as_str();
    let metadata = generate_metadata(
        consultant_id,
        &request_consultation_param.first_candidate_in_jst,
        &request_consultation_param.second_candidate_in_jst,
        &request_consultation_param.third_candidate_in_jst,
    );
    let create_charge = CreateCharge::build()
        .price(&price)
        .card(card)
        .capture(false)
        .expiry_days(EXPIRY_DAYS)
        .metadata(&metadata)
        .tenant(tenant_id.as_str())
        .three_d_secure(true)
        .finish()
        .map_err(|e| {
            error!("failed to build CreateCharge: {}", e);
            // finishで発生する可能性のあるエラーは与えられる引数では発生することはないのでunexpected_err_respとして処理する
            // 補足: priceのamountの範囲も、fee_per_hour_yenを設定している箇所で、MIN_FEE_PER_HOUR_IN_YEN..=MAX_FEE_PER_HOUR_IN_YENの範囲内のため、
            //       amountに関するエラーも発生しない
            unexpected_err_resp()
        })?;
    let charge = charge_op.create_charge(&create_charge).await.map_err(|e| {
        // TODO: https://pay.jp/docs/api/#error に基づいてハンドリングする
        error!("failed to create charge: {}", e);
        unexpected_err_resp()
    })?;

    info!(
        "started 3D secure flow (account_id, {}, consultant_id{}, charge.id: {})",
        account_id, consultant_id, charge.id
    );
    Ok((
        StatusCode::OK,
        Json(RequestConsultationResult {
            charge_id: charge.id,
        }),
    ))
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
    let _ = validate_consultation_date_time(first_candidate_in_jst, current_date_time).map_err(
        |e| {
            error!("invalid first_candidate_in_jst: {}", e);
            convert_consultation_date_time_validation_err(&e)
        },
    )?;
    let _ = validate_consultation_date_time(second_candidate_in_jst, current_date_time).map_err(
        |e| {
            error!("invalid second_candidate_in_jst: {}", e);
            convert_consultation_date_time_validation_err(&e)
        },
    )?;
    let _ = validate_consultation_date_time(third_candidate_in_jst, current_date_time).map_err(
        |e| {
            error!("invalid third_candidate_in_jst: {}", e);
            convert_consultation_date_time_validation_err(&e)
        },
    )?;

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

async fn validate_identity_exists(
    account_id: i64,
    request_consultation_op: &impl RequestConsultationOperation,
) -> Result<(), ErrResp> {
    let identity_exists = request_consultation_op
        .check_if_identity_exists(account_id)
        .await?;
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

async fn validate_consultant_is_available(
    consultant_id: i64,
    request_consultation_op: &impl RequestConsultationOperation,
) -> Result<(), ErrResp> {
    let consultant_available = request_consultation_op
        .check_if_consultant_is_available(consultant_id)
        .await?;
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
    request_consultation_op: &impl RequestConsultationOperation,
) -> Result<i32, ErrResp> {
    let fee_per_hour_in_yen = request_consultation_op
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

async fn get_tenant_id(
    consultant_id: i64,
    request_consultation_op: &impl RequestConsultationOperation,
) -> Result<String, ErrResp> {
    let tenant_id = request_consultation_op
        .find_tenant_id_by_consultant_id(consultant_id)
        .await?;
    let tenant_id = tenant_id.ok_or_else(|| {
        error!(
            "tenant_id does not exist (consultant_id: {})",
            consultant_id
        );
        unexpected_err_resp()
    })?;
    Ok(tenant_id)
}

fn generate_metadata(
    consultant_id: i64,
    first_candidate_in_jst: &ConsultationDateTime,
    second_candidate_in_jst: &ConsultationDateTime,
    third_candidate_in_jst: &ConsultationDateTime,
) -> Metadata {
    let mut metadata = Metadata::with_capacity(4);

    let _ = metadata.insert(
        KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ.to_string(),
        consultant_id.to_string(),
    );

    let date_time = DateTime::<FixedOffset>::from_local(
        NaiveDate::from_ymd(
            first_candidate_in_jst.year,
            first_candidate_in_jst.month,
            first_candidate_in_jst.day,
        )
        .and_hms(first_candidate_in_jst.hour, 0, 0),
        *JAPANESE_TIME_ZONE,
    );
    let _ = metadata.insert(
        KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ.to_string(),
        date_time.to_rfc2822(),
    );

    let date_time = DateTime::<FixedOffset>::from_local(
        NaiveDate::from_ymd(
            second_candidate_in_jst.year,
            second_candidate_in_jst.month,
            second_candidate_in_jst.day,
        )
        .and_hms(second_candidate_in_jst.hour, 0, 0),
        *JAPANESE_TIME_ZONE,
    );
    let _ = metadata.insert(
        KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ.to_string(),
        date_time.to_rfc2822(),
    );

    let date_time = DateTime::<FixedOffset>::from_local(
        NaiveDate::from_ymd(
            third_candidate_in_jst.year,
            third_candidate_in_jst.month,
            third_candidate_in_jst.day,
        )
        .and_hms(third_candidate_in_jst.hour, 0, 0),
        *JAPANESE_TIME_ZONE,
    );
    let _ = metadata.insert(
        KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ.to_string(),
        date_time.to_rfc2822(),
    );

    metadata
}

#[cfg(test)]
mod tests {
    // TODO
}
