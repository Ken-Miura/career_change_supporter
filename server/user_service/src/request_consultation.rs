// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{DateTime, Datelike, Duration, FixedOffset, NaiveDate, Utc};
use common::payment_platform::charge::{Charge, CreateCharge};
use common::payment_platform::Metadata;
use common::{
    payment_platform::charge::{ChargeOperation, ChargeOperationImpl},
    ErrResp, RespResult,
};
use common::{ApiError, JAPANESE_TIME_ZONE};
use entity::prelude::ConsultationReq;
use entity::prelude::Settlement;
use entity::sea_orm::{ColumnTrait, QueryFilter};
use entity::{consultation_req, settlement};
use entity::{
    prelude::ConsultingFee,
    sea_orm::{DatabaseConnection, EntityTrait},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::err::Code;
use crate::util::rewards::MAX_NUM_OF_CHARGES_PER_REQUEST;
use crate::util::validator::consultation_date_time_validator::{
    validate_consultation_date_time, ConsultationDateTimeValidationError,
};
use crate::util::{
    convert_payment_err_to_err_resp, create_start_and_end_timestamps_of_current_year,
    EXPIRY_DAYS_OF_CHARGE, KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ,
    KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ, KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
    KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ, MAX_ANNUAL_REWARDS_IN_YEN,
    MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE,
};
use crate::{
    err::unexpected_err_resp,
    util::{self, session::User, ACCESS_INFO},
};

pub(crate) async fn post_request_consultation(
    User { account_id }: User,
    Json(param): Json<RequestConsultationParam>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<RequestConsultationResult> {
    let current_date_time = Utc::now().with_timezone(&JAPANESE_TIME_ZONE.to_owned());
    let request_consultation_op = RequestConsultationOperationImpl { pool };
    handle_request_consultation(
        account_id,
        param,
        &current_date_time,
        request_consultation_op,
    )
    .await
}

#[derive(Clone, Deserialize, Debug)]
pub(crate) struct RequestConsultationParam {
    pub consultant_id: i64,
    pub fee_per_hour_in_yen: i32,
    pub card_token: String,
    pub first_candidate_in_jst: ConsultationDateTime,
    pub second_candidate_in_jst: ConsultationDateTime,
    pub third_candidate_in_jst: ConsultationDateTime,
}

#[derive(Clone, Deserialize, Debug, PartialEq)]
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

    async fn create_charge(&self, create_charge: &CreateCharge) -> Result<Charge, ErrResp>;

    /// 相談依頼があり、まだ相談を受け付けるか決めていないものの相談料の合計
    async fn get_amount_of_consultation_req(
        &self,
        consultant_id: i64,
        current_date_time: &DateTime<FixedOffset>,
    ) -> Result<i32, ErrResp>;

    /// 相談を受け付けたが、まだ未決済（相談者がまだ評価を実施していない、または自動決済が走っていない状態）となっているものの相談料の合計
    async fn get_expected_rewards(
        &self,
        consultant_id: i64,
        current_date_time: &DateTime<FixedOffset>,
    ) -> Result<i32, ErrResp>;

    /// 決済が済んでいるものの相談料の合計
    async fn get_rewards_of_the_year(
        &self,
        since_timestamp: i64,
        until_timestamp: i64,
        tenant_id: &str,
    ) -> Result<i32, ErrResp>;
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

    async fn create_charge(&self, create_charge: &CreateCharge) -> Result<Charge, ErrResp> {
        let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
        let charge = charge_op.create_charge(create_charge).await.map_err(|e| {
            error!("failed to create charge: {}", e);
            convert_payment_err_to_err_resp(&e)
        })?;
        Ok(charge)
    }

    async fn get_amount_of_consultation_req(
        &self,
        consultant_id: i64,
        current_date_time: &DateTime<FixedOffset>,
    ) -> Result<i32, ErrResp> {
        let criteria = *current_date_time
            + Duration::hours(*MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE as i64);
        let reqs = ConsultationReq::find()
            .filter(consultation_req::Column::ConsultantId.eq(consultant_id))
            .filter(consultation_req::Column::LatestCandidateDateTime.gt(criteria))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter consultation_req (consultant_id: {}, current_date_time: {}, MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE: {}): {}",
                    consultant_id, current_date_time, *MIN_DURATION_IN_HOUR_BEFORE_CONSULTATION_ACCEPTANCE, e
                );
                unexpected_err_resp()
            })?;
        let charge_ids = reqs.into_iter().map(|r| r.charge_id).collect();
        let amount = get_sum_of_amount(charge_ids).await?;
        Ok(amount)
    }

    async fn get_expected_rewards(
        &self,
        consultant_id: i64,
        current_date_time: &DateTime<FixedOffset>,
    ) -> Result<i32, ErrResp> {
        let settlements = Settlement::find()
            .filter(settlement::Column::ConsultantId.eq(consultant_id))
            .filter(settlement::Column::ExpiredAt.gt(*current_date_time))
            .filter(settlement::Column::Settled.eq(false))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter settlement (consultant_id: {}, current_date_time: {}): {}",
                    consultant_id, current_date_time, e
                );
                unexpected_err_resp()
            })?;
        let charge_ids = settlements.into_iter().map(|s| s.charge_id).collect();
        let amount = get_sum_of_amount(charge_ids).await?;
        Ok(amount)
    }

    async fn get_rewards_of_the_year(
        &self,
        since_timestamp: i64,
        until_timestamp: i64,
        tenant_id: &str,
    ) -> Result<i32, ErrResp> {
        let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
        crate::util::rewards::get_rewards_of_the_duration(
            charge_op,
            MAX_NUM_OF_CHARGES_PER_REQUEST,
            since_timestamp,
            until_timestamp,
            tenant_id,
        )
        .await
    }
}

async fn get_sum_of_amount(charge_ids: Vec<String>) -> Result<i32, ErrResp> {
    let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
    let mut amount = 0;
    for charge_id in charge_ids {
        let charge = charge_op
            .ge_charge_by_charge_id(charge_id.as_str())
            .await
            .map_err(|err| match err {
                common::payment_platform::Error::RequestProcessingError(err) => {
                    error!("failed to process request on getting charges: {}", err);
                    unexpected_err_resp()
                }
                common::payment_platform::Error::ApiError(err) => {
                    error!("failed to request charge operation: {}", err);
                    let status_code = err.error.status as u16;
                    if status_code == StatusCode::TOO_MANY_REQUESTS.as_u16() {
                        return (
                            StatusCode::TOO_MANY_REQUESTS,
                            Json(ApiError {
                                code: Code::ReachPaymentPlatformRateLimit as u32,
                            }),
                        );
                    }
                    unexpected_err_resp()
                }
            })?;
        amount += charge.amount;
    }
    Ok(amount)
}

async fn handle_request_consultation(
    account_id: i64,
    request_consultation_param: RequestConsultationParam,
    current_date_time: &DateTime<FixedOffset>,
    request_consultation_op: impl RequestConsultationOperation,
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

    let _ = ensure_expected_annual_rewards_does_not_exceed_max_annual_rewards(
        consultant_id,
        tenant_id.as_str(),
        current_date_time,
        fee_per_hour_in_yen,
        &request_consultation_op,
    )
    .await?;

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
        .expiry_days(*EXPIRY_DAYS_OF_CHARGE)
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
    let charge = request_consultation_op
        .create_charge(&create_charge)
        .await?;

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

async fn ensure_expected_annual_rewards_does_not_exceed_max_annual_rewards(
    consultant_id: i64,
    tenant_id: &str,
    current_date_time: &DateTime<FixedOffset>,
    fee_per_hour_in_yen: i32,
    request_consultation_op: &impl RequestConsultationOperation,
) -> Result<(), ErrResp> {
    let amount_of_consultation_req = request_consultation_op
        .get_amount_of_consultation_req(consultant_id, current_date_time)
        .await?;

    let expected_rewards = request_consultation_op
        .get_expected_rewards(consultant_id, current_date_time)
        .await?;

    let (current_year_since_timestamp, current_year_until_timestamp) =
        create_start_and_end_timestamps_of_current_year(current_date_time.year());
    let rewards_of_the_year = request_consultation_op
        .get_rewards_of_the_year(
            current_year_since_timestamp,
            current_year_until_timestamp,
            tenant_id,
        )
        .await?;

    let expected_annual_rewards =
        fee_per_hour_in_yen + amount_of_consultation_req + expected_rewards + rewards_of_the_year;
    if expected_annual_rewards > *MAX_ANNUAL_REWARDS_IN_YEN {
        error!("exceed max annual rewards (expected_annual_rewards ({} = fee_per_hour_in_yen ({}) + amount_of_consultation_req ({}) + expected_rewards ({}) + rewards_of_the_year({})) > MAX_ANNUAL_REWARDS_IN_YEN ({}))", 
            expected_annual_rewards, fee_per_hour_in_yen, amount_of_consultation_req, expected_rewards, rewards_of_the_year, *MAX_ANNUAL_REWARDS_IN_YEN);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ExceedMaxAnnualRewards as u32,
            }),
        ));
    }
    Ok(())
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
        date_time.to_rfc3339(),
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
        date_time.to_rfc3339(),
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
        date_time.to_rfc3339(),
    );

    metadata
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;
    use axum::{async_trait, Json};
    use chrono::{DateTime, Datelike, FixedOffset, TimeZone};
    use common::payment_platform::customer::Card;
    use common::{
        payment_platform::charge::{Charge, CreateCharge},
        ErrResp, RespResult, JAPANESE_TIME_ZONE,
    };
    use once_cell::sync::Lazy;

    use crate::util::{
        KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ, KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
        KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ, KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
    };

    use super::{
        handle_request_consultation, ConsultationDateTime, RequestConsultationOperation,
        RequestConsultationParam, RequestConsultationResult,
    };

    #[derive(Debug)]
    struct TestCase {
        name: String,
        input: Input,
        expected: RespResult<RequestConsultationResult>,
    }

    #[derive(Debug)]
    struct Input {
        account_id: i64,
        param: RequestConsultationParam,
        current_date_time: DateTime<FixedOffset>,
        req_op: RequestConsultationOperationMock,
    }

    #[derive(Clone, Debug)]
    struct RequestConsultationOperationMock {
        account_id: i64,
        consultant_id: i64,
        fee_per_hour_in_yen: Option<i32>,
        tenant_id: Option<String>,
        card_token: String,
        charge: Charge,
        current_date_time: DateTime<FixedOffset>,
        amount: i32,
        expected_rewards: i32,
        first_candidate_in_jst: DateTime<FixedOffset>,
        second_candidate_in_jst: DateTime<FixedOffset>,
        third_candidate_in_jst: DateTime<FixedOffset>,
        rewards_of_the_year: i32,
    }

    #[async_trait]
    impl RequestConsultationOperation for RequestConsultationOperationMock {
        async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
            if self.account_id != account_id {
                return Ok(false);
            };
            Ok(true)
        }

        async fn check_if_consultant_is_available(
            &self,
            consultant_id: i64,
        ) -> Result<bool, ErrResp> {
            if self.consultant_id != consultant_id {
                return Ok(false);
            };
            Ok(true)
        }

        async fn find_fee_per_hour_in_yen_by_consultant_id(
            &self,
            consultant_id: i64,
        ) -> Result<Option<i32>, ErrResp> {
            assert_eq!(self.consultant_id, consultant_id);
            Ok(self.fee_per_hour_in_yen)
        }

        async fn find_tenant_id_by_consultant_id(
            &self,
            consultant_id: i64,
        ) -> Result<Option<String>, ErrResp> {
            assert_eq!(self.consultant_id, consultant_id);
            Ok(self.tenant_id.clone())
        }

        async fn create_charge(&self, create_charge: &CreateCharge) -> Result<Charge, ErrResp> {
            assert_eq!(
                self.fee_per_hour_in_yen.expect("failed to get Ok"),
                create_charge.price().expect("failed to get Ok").0
            );
            assert_eq!(
                self.card_token,
                create_charge.card().expect("failed to get Ok")
            );
            assert_eq!(self.tenant_id, create_charge.tenant());
            let metadata = create_charge.metadata().expect("failed to get Ok");
            let consultant_id = metadata
                .get(KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ)
                .expect("failed to get Ok")
                .parse::<i64>()
                .expect("failed to get Ok");
            assert_eq!(self.consultant_id, consultant_id);
            let first_candidate = metadata
                .get(KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ)
                .expect("failed to get Ok");
            let first_candidate = DateTime::<FixedOffset>::parse_from_rfc3339(first_candidate)
                .expect("failed to get Ok");
            assert_eq!(self.first_candidate_in_jst, first_candidate);
            let second_candidate = metadata
                .get(KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ)
                .expect("failed to get Ok");
            let second_candidate = DateTime::<FixedOffset>::parse_from_rfc3339(second_candidate)
                .expect("failed to get Ok");
            assert_eq!(self.second_candidate_in_jst, second_candidate);
            let third_candidate = metadata
                .get(KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ)
                .expect("failed to get Ok");
            let third_candidate = DateTime::<FixedOffset>::parse_from_rfc3339(third_candidate)
                .expect("failed to get Ok");
            assert_eq!(self.third_candidate_in_jst, third_candidate);
            Ok(self.charge.clone())
        }

        async fn get_amount_of_consultation_req(
            &self,
            consultant_id: i64,
            current_date_time: &DateTime<FixedOffset>,
        ) -> Result<i32, ErrResp> {
            assert_eq!(self.consultant_id, consultant_id);
            assert_eq!(self.current_date_time, *current_date_time);
            Ok(self.amount)
        }

        async fn get_expected_rewards(
            &self,
            consultant_id: i64,
            current_date_time: &DateTime<FixedOffset>,
        ) -> Result<i32, ErrResp> {
            assert_eq!(self.consultant_id, consultant_id);
            assert_eq!(self.current_date_time, *current_date_time);
            Ok(self.expected_rewards)
        }

        async fn get_rewards_of_the_year(
            &self,
            since_timestamp: i64,
            until_timestamp: i64,
            tenant_id: &str,
        ) -> Result<i32, ErrResp> {
            assert_eq!(self.tenant_id.clone().expect("failed to get Ok"), tenant_id);
            let year = self.current_date_time.year();
            let since = JAPANESE_TIME_ZONE
                .ymd(year, 1, 1)
                .and_hms(0, 0, 0)
                .timestamp();
            assert_eq!(since, since_timestamp);
            let until = JAPANESE_TIME_ZONE
                .ymd(year, 12, 31)
                .and_hms(23, 59, 59)
                .timestamp();
            assert_eq!(until, until_timestamp);
            Ok(self.rewards_of_the_year)
        }
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        vec![TestCase {
            name: "success case 1".to_string(),
            input: Input {
                account_id: 1,
                param: RequestConsultationParam {
                    consultant_id: 2,
                    fee_per_hour_in_yen: 5000,
                    card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                    first_candidate_in_jst: ConsultationDateTime {
                        year: 2022,
                        month: 11,
                        day: 4,
                        hour: 7,
                    },
                    second_candidate_in_jst: ConsultationDateTime {
                        year: 2022,
                        month: 11,
                        day: 4,
                        hour: 23,
                    },
                    third_candidate_in_jst: ConsultationDateTime {
                        year: 2022,
                        month: 11,
                        day: 22,
                        hour: 7,
                    },
                },
                current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(7, 0, 0),
                req_op: RequestConsultationOperationMock {
                    account_id: 1,
                    consultant_id: 2,
                    fee_per_hour_in_yen: Some(5000),
                    tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
                    card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                    charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
                    current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(7, 0, 0),
                    amount: 5000,
                    expected_rewards: 15000,
                    first_candidate_in_jst: JAPANESE_TIME_ZONE.ymd(2022, 11, 4).and_hms(7, 0, 0),
                    second_candidate_in_jst: JAPANESE_TIME_ZONE.ymd(2022, 11, 4).and_hms(23, 0, 0),
                    third_candidate_in_jst: JAPANESE_TIME_ZONE.ymd(2022, 11, 22).and_hms(7, 0, 0),
                    rewards_of_the_year: 20000,
                },
            },
            expected: Ok((
                StatusCode::OK,
                Json(RequestConsultationResult {
                    charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
                }),
            )),
        }]
    });

    // create_dummy_chargeでAPI呼び出しの結果返却されるChargeを作成する
    // 返却されたChargeはidだけ利用し、他を参照することはないのでid以外はダミーで関係ない値で埋めてある
    fn create_dummy_charge(charge_id: &str) -> Charge {
        Charge {
            id: charge_id.to_string(),
            object: "charge".to_string(),
            livemode: false,
            created: 1639931415,
            amount: 5000,
            currency: "jpy".to_string(),
            paid: true,
            expired_at: None,
            captured: false,
            captured_at: Some(1639931415),
            card: Some(Card {
                object: "card".to_string(),
                id: "car_33ab04bcdc00f0cc6d6df16bbe79".to_string(),
                created: 1639931415,
                name: None,
                last4: "4242".to_string(),
                exp_month: 12,
                exp_year: 2022,
                brand: "Visa".to_string(),
                cvc_check: "passed".to_string(),
                fingerprint: "e1d8225886e3a7211127df751c86787f".to_string(),
                address_state: None,
                address_city: None,
                address_line1: None,
                address_line2: None,
                country: None,
                address_zip: None,
                address_zip_check: "unchecked".to_string(),
                metadata: None,
            }),
            customer: None,
            description: None,
            failure_code: None,
            failure_message: None,
            fee_rate: Some("3.00".to_string()),
            refunded: false,
            amount_refunded: 0,
            refund_reason: None,
            subscription: None,
            metadata: None,
            platform_fee: None,
            tenant: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
            platform_fee_rate: Some("30.0".to_string()),
            total_platform_fee: Some(1350),
            three_d_secure_status: Some("unverified".to_string()),
        }
    }

    #[tokio::test]
    async fn handle_request_consultation_tests() {
        for test_case in TEST_CASE_SET.iter() {
            let account_id = test_case.input.account_id;
            let param = test_case.input.param.clone();
            let current_date_time = test_case.input.current_date_time;
            let req_op = test_case.input.req_op.clone();

            let resp =
                handle_request_consultation(account_id, param, &current_date_time, req_op).await;

            let message = format!("test case \"{}\" failed", test_case.name.clone());
            if test_case.expected.is_ok() {
                let result = resp.expect("failed to get Ok");
                let expected_result = test_case.expected.as_ref().expect("failed to get Ok");
                assert_eq!(expected_result.0, result.0, "{}", message);
                assert_eq!(expected_result.1 .0, result.1 .0, "{}", message);
            } else {
                let result = resp.expect_err("failed to get Err");
                let expected_result = test_case.expected.as_ref().expect_err("failed to get Err");
                assert_eq!(expected_result.0, result.0, "{}", message);
                assert_eq!(expected_result.1 .0, result.1 .0, "{}", message);
            }
        }
    }
}
