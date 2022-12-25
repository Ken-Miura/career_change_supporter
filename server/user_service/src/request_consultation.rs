// Copyright 2022 Ken Miura

use axum::async_trait;
use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, FixedOffset, NaiveDate, Utc};
use common::payment_platform::charge::{Charge, CreateCharge};
use common::payment_platform::Metadata;
use common::{
    payment_platform::charge::{ChargeOperation, ChargeOperationImpl},
    ErrResp, RespResult,
};
use common::{ApiError, JAPANESE_TIME_ZONE};
use entity::prelude::Settlement;
use entity::prelude::StoppedSettlement;
use entity::sea_orm::{ColumnTrait, QueryFilter};
use entity::{
    prelude::ConsultingFee,
    sea_orm::{DatabaseConnection, EntityTrait},
};
use entity::{settlement, stopped_settlement};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::err::Code;
use crate::util::charge_metadata_key::{
    KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ, KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
    KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ, KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
};
use crate::util::consultation::{convert_payment_err_to_err_resp, ConsultationDateTime};
use crate::util::optional_env_var::{EXPIRY_DAYS_OF_CHARGE, MAX_ANNUAL_REWARDS_IN_YEN};
use crate::util::rewards::{
    calculate_rewards, create_start_and_end_date_time_of_current_year, PaymentInfo,
};
use crate::util::validator::consultation_date_time_validator::{
    validate_consultation_date_time, ConsultationDateTimeValidationError,
};
use crate::{
    err::unexpected_err_resp,
    util::{self, session::User, ACCESS_INFO},
};

pub(crate) async fn post_request_consultation(
    User { account_id }: User,
    State(pool): State<DatabaseConnection>,
    Json(param): Json<RequestConsultationParam>,
) -> RespResult<RequestConsultationResult> {
    let current_date_time = Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    let op = RequestConsultationOperationImpl { pool };
    handle_request_consultation(account_id, param, &current_date_time, op).await
}

#[derive(Clone, Deserialize, Debug)]
pub(crate) struct RequestConsultationParam {
    pub(crate) consultant_id: i64,
    pub(crate) fee_per_hour_in_yen: i32,
    pub(crate) card_token: String,
    pub(crate) first_candidate_in_jst: ConsultationDateTime,
    pub(crate) second_candidate_in_jst: ConsultationDateTime,
    pub(crate) third_candidate_in_jst: ConsultationDateTime,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct RequestConsultationResult {
    pub(crate) charge_id: String,
}

#[async_trait]
trait RequestConsultationOperation {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp>;

    async fn check_if_user_account_is_available(
        &self,
        user_account_id: i64,
    ) -> Result<bool, ErrResp>;

    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp>;

    async fn find_fee_per_hour_in_yen_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Option<i32>, ErrResp>;

    async fn find_tenant_id_by_consultant_id(
        &self,
        consultant_id: i64,
    ) -> Result<Option<String>, ErrResp>;

    async fn filter_valid_settlement_by_consultant_id(
        &self,
        consultant_id: i64,
        current_date_time: &DateTime<FixedOffset>,
    ) -> Result<Vec<PaymentInfo>, ErrResp>;

    async fn filter_valid_stopped_settlement_by_consultant_id(
        &self,
        consultant_id: i64,
        current_date_time: &DateTime<FixedOffset>,
    ) -> Result<Vec<PaymentInfo>, ErrResp>;

    /// （startとendを含む）startからendまでの期間のPaymentInfoを取得する
    async fn filter_receipts_of_the_duration_by_consultant_id(
        &self,
        consultant_id: i64,
        start: &DateTime<FixedOffset>,
        end: &DateTime<FixedOffset>,
    ) -> Result<Vec<PaymentInfo>, ErrResp>;

    async fn create_charge(&self, create_charge: &CreateCharge) -> Result<Charge, ErrResp>;
}

struct RequestConsultationOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl RequestConsultationOperation for RequestConsultationOperationImpl {
    async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
        util::identity_checker::check_if_identity_exists(&self.pool, account_id).await
    }

    async fn check_if_user_account_is_available(
        &self,
        user_account_id: i64,
    ) -> Result<bool, ErrResp> {
        util::check_if_user_account_is_available(&self.pool, user_account_id).await
    }

    async fn check_if_consultant_is_available(&self, consultant_id: i64) -> Result<bool, ErrResp> {
        util::check_if_user_account_is_available(&self.pool, consultant_id).await
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

    async fn filter_valid_settlement_by_consultant_id(
        &self,
        consultant_id: i64,
        current_date_time: &DateTime<FixedOffset>,
    ) -> Result<Vec<PaymentInfo>, ErrResp> {
        let models = Settlement::find()
            .filter(settlement::Column::ConsultantId.eq(consultant_id))
            .filter(settlement::Column::CreditFacilitiesExpiredAt.gt(*current_date_time))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter settlement (consultant_id: {}, current_date_time: {}): {}",
                    consultant_id, current_date_time, e
                );
                unexpected_err_resp()
            })?;
        // 正確な報酬額を得るためには取得したレコードに記載されているcharge_idを使い、
        // 一つ一つChageオブジェクトをPAYJPから取得して計算をする必要がある。
        // しかし、PAYJPの流量制限に引っかかりやすくなる危険性を考慮し、レコードのキャシュしてある値を使い報酬を計算する
        Ok(models
            .into_iter()
            .map(|m| PaymentInfo {
                fee_per_hour_in_yen: m.fee_per_hour_in_yen,
                platform_fee_rate_in_percentage: m.platform_fee_rate_in_percentage,
            })
            .collect::<Vec<PaymentInfo>>())
    }

    async fn filter_valid_stopped_settlement_by_consultant_id(
        &self,
        consultant_id: i64,
        current_date_time: &DateTime<FixedOffset>,
    ) -> Result<Vec<PaymentInfo>, ErrResp> {
        let models = StoppedSettlement::find()
            .filter(stopped_settlement::Column::ConsultantId.eq(consultant_id))
            .filter(stopped_settlement::Column::CreditFacilitiesExpiredAt.gt(*current_date_time))
            .all(&self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to filter stopped_settlement (consultant_id: {}, current_date_time: {}): {}",
                    consultant_id, current_date_time, e
                );
                unexpected_err_resp()
            })?;
        // 正確な報酬額を得るためには取得したレコードに記載されているcharge_idを使い、
        // 一つ一つChageオブジェクトをPAYJPから取得して計算をする必要がある。
        // しかし、PAYJPの流量制限に引っかかりやすくなる危険性を考慮し、レコードのキャシュしてある値を使い報酬を計算する
        Ok(models
            .into_iter()
            .map(|m| PaymentInfo {
                fee_per_hour_in_yen: m.fee_per_hour_in_yen,
                platform_fee_rate_in_percentage: m.platform_fee_rate_in_percentage,
            })
            .collect::<Vec<PaymentInfo>>())
    }

    async fn filter_receipts_of_the_duration_by_consultant_id(
        &self,
        consultant_id: i64,
        start: &DateTime<FixedOffset>,
        end: &DateTime<FixedOffset>,
    ) -> Result<Vec<PaymentInfo>, ErrResp> {
        util::rewards::filter_receipts_of_the_duration_by_consultant_id(
            &self.pool,
            consultant_id,
            start,
            end,
        )
        .await
    }

    async fn create_charge(&self, create_charge: &CreateCharge) -> Result<Charge, ErrResp> {
        let charge_op = ChargeOperationImpl::new(&ACCESS_INFO);
        let charge = charge_op.create_charge(create_charge).await.map_err(|e| {
            error!("failed to create charge: {}", e);
            convert_payment_err_to_err_resp(&e)
        })?;
        Ok(charge)
    }
}

async fn handle_request_consultation(
    account_id: i64,
    request_consultation_param: RequestConsultationParam,
    current_date_time: &DateTime<FixedOffset>,
    op: impl RequestConsultationOperation,
) -> RespResult<RequestConsultationResult> {
    let consultant_id = request_consultation_param.consultant_id;
    validate_consultant_id_is_positive(consultant_id)?;
    validate_candidates(
        &request_consultation_param.first_candidate_in_jst,
        &request_consultation_param.second_candidate_in_jst,
        &request_consultation_param.third_candidate_in_jst,
        current_date_time,
    )?;
    validate_user_account_is_available(account_id, &op).await?;
    validate_identity_exists(account_id, &op).await?;
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

    ensure_expected_annual_rewards_does_not_exceed_max_annual_rewards(
        consultant_id,
        current_date_time,
        fee_per_hour_in_yen,
        &op,
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
    let tenant_id = get_tenant_id(consultant_id, &op).await?;
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
    let charge = op.create_charge(&create_charge).await?;

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

async fn validate_user_account_is_available(
    user_account_id: i64,
    op: &impl RequestConsultationOperation,
) -> Result<(), ErrResp> {
    let user_account_available = op
        .check_if_user_account_is_available(user_account_id)
        .await?;
    if !user_account_available {
        error!(
            "user account is not available (user_account_id: {})",
            user_account_id
        );
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::AccountDisabled as u32,
            }),
        ));
    }
    Ok(())
}

async fn validate_identity_exists(
    account_id: i64,
    op: &impl RequestConsultationOperation,
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

async fn get_tenant_id(
    consultant_id: i64,
    op: &impl RequestConsultationOperation,
) -> Result<String, ErrResp> {
    let tenant_id = op.find_tenant_id_by_consultant_id(consultant_id).await?;
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
    current_date_time: &DateTime<FixedOffset>,
    fee_per_hour_in_yen: i32,
    op: &impl RequestConsultationOperation,
) -> Result<(), ErrResp> {
    let expected_rewards = get_expected_rewards(consultant_id, current_date_time, op).await?;
    let rewards = get_rewards(consultant_id, current_date_time, op).await?;

    let expected_annual_rewards = fee_per_hour_in_yen + expected_rewards + rewards;
    if expected_annual_rewards > *MAX_ANNUAL_REWARDS_IN_YEN {
        error!("consultant (consultant_id: {}) exceeds max annual rewards (expected_annual_rewards ({} = fee_per_hour_in_yen ({}) + expected_rewards ({}) + rewards({})) > MAX_ANNUAL_REWARDS_IN_YEN ({}))", 
          consultant_id, expected_annual_rewards, fee_per_hour_in_yen, expected_rewards, rewards, *MAX_ANNUAL_REWARDS_IN_YEN);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ExceedMaxAnnualRewards as u32,
            }),
        ));
    }
    Ok(())
}

async fn get_expected_rewards(
    consultant_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    op: &impl RequestConsultationOperation,
) -> Result<i32, ErrResp> {
    let mut p1 = op
        .filter_valid_settlement_by_consultant_id(consultant_id, current_date_time)
        .await?;
    let mut p2 = op
        .filter_valid_stopped_settlement_by_consultant_id(consultant_id, current_date_time)
        .await?;
    p1.append(&mut p2);
    let expected_rewards = calculate_rewards(&p1)?;
    Ok(expected_rewards)
}

async fn get_rewards(
    consultant_id: i64,
    current_date_time: &DateTime<FixedOffset>,
    op: &impl RequestConsultationOperation,
) -> Result<i32, ErrResp> {
    let (start, end) = create_start_and_end_date_time_of_current_year(current_date_time);
    let p = op
        .filter_receipts_of_the_duration_by_consultant_id(consultant_id, &start, &end)
        .await?;
    let rewards = calculate_rewards(&p)?;
    Ok(rewards)
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
    use chrono::{DateTime, FixedOffset, TimeZone};
    use common::payment_platform::customer::Card;
    use common::ApiError;
    use common::{
        payment_platform::charge::{Charge, CreateCharge},
        ErrResp, RespResult, JAPANESE_TIME_ZONE,
    };
    use once_cell::sync::Lazy;

    use crate::err::Code;
    use crate::util::charge_metadata_key::{
        KEY_TO_CONSULTAND_ID_ON_CHARGE_OBJ, KEY_TO_FIRST_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
        KEY_TO_SECOND_CANDIDATE_IN_JST_ON_CHARGE_OBJ, KEY_TO_THIRD_CANDIDATE_IN_JST_ON_CHARGE_OBJ,
    };
    use crate::util::{
        consultation::ConsultationDateTime,
        rewards::{create_start_and_end_date_time_of_current_year, PaymentInfo},
    };

    use super::{
        handle_request_consultation, RequestConsultationOperation, RequestConsultationParam,
        RequestConsultationResult,
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
        first_candidate_in_jst: DateTime<FixedOffset>,
        second_candidate_in_jst: DateTime<FixedOffset>,
        third_candidate_in_jst: DateTime<FixedOffset>,
        user_account_is_available: bool,
        settlement_payments: Vec<PaymentInfo>,
        stopped_settlement_payments: Vec<PaymentInfo>,
        receipt_payments: Vec<PaymentInfo>,
    }

    #[async_trait]
    impl RequestConsultationOperation for RequestConsultationOperationMock {
        async fn check_if_identity_exists(&self, account_id: i64) -> Result<bool, ErrResp> {
            if self.account_id != account_id {
                return Ok(false);
            };
            Ok(true)
        }

        async fn check_if_user_account_is_available(
            &self,
            _user_account_id: i64,
        ) -> Result<bool, ErrResp> {
            Ok(self.user_account_is_available)
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

        async fn filter_valid_settlement_by_consultant_id(
            &self,
            consultant_id: i64,
            current_date_time: &DateTime<FixedOffset>,
        ) -> Result<Vec<PaymentInfo>, ErrResp> {
            assert_eq!(self.consultant_id, consultant_id);
            assert_eq!(self.current_date_time, *current_date_time);
            Ok(self.settlement_payments.clone())
        }

        async fn filter_valid_stopped_settlement_by_consultant_id(
            &self,
            consultant_id: i64,
            current_date_time: &DateTime<FixedOffset>,
        ) -> Result<Vec<PaymentInfo>, ErrResp> {
            assert_eq!(self.consultant_id, consultant_id);
            assert_eq!(self.current_date_time, *current_date_time);
            Ok(self.stopped_settlement_payments.clone())
        }

        async fn filter_receipts_of_the_duration_by_consultant_id(
            &self,
            consultant_id: i64,
            start: &DateTime<FixedOffset>,
            end: &DateTime<FixedOffset>,
        ) -> Result<Vec<PaymentInfo>, ErrResp> {
            assert_eq!(self.consultant_id, consultant_id);
            let (s, e) = create_start_and_end_date_time_of_current_year(&self.current_date_time);
            assert_eq!(s, *start);
            assert_eq!(e, *end);
            Ok(self.receipt_payments.clone())
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
    }

    static TEST_CASE_SET: Lazy<Vec<TestCase>> = Lazy::new(|| {
        vec![
            TestCase {
                name: "success case 1".to_string(),
                input: Input {
                    account_id: 1,
                    param: RequestConsultationParam {
                        consultant_id: 2,
                        fee_per_hour_in_yen: 4000,
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
                        fee_per_hour_in_yen: Some(4000),
                        tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
                        current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(7, 0, 0),
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(7, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(23, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 22)
                            .and_hms(7, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Ok((
                    StatusCode::OK,
                    Json(RequestConsultationResult {
                        charge_id: "ch_fa990a4c10672a93053a774730b0a".to_string(),
                    }),
                )),
            },
            TestCase {
                name: "consultant id is negative".to_string(),
                input: Input {
                    account_id: 1,
                    param: RequestConsultationParam {
                        consultant_id: -1,
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
                        consultant_id: -1,
                        fee_per_hour_in_yen: Some(5000),
                        tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
                        current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(7, 0, 0),
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(7, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(23, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 22)
                            .and_hms(7, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NonPositiveConsultantId as u32,
                    }),
                )),
            },
            TestCase {
                name: "first_candidate_in_jst IllegalConsultationDateTime".to_string(),
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
                            hour: 24,
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
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(7, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(23, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 22)
                            .and_hms(7, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalConsultationDateTime as u32,
                    }),
                )),
            },
            TestCase {
                name: "first_candidate_in_jst IllegalConsultationHour".to_string(),
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
                            hour: 6,
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
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(6, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(23, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 22)
                            .and_hms(7, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalConsultationHour as u32,
                    }),
                )),
            },
            TestCase {
                name: "first_candidate_in_jst InvalidConsultationDateTime".to_string(),
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
                    current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(7, 0, 1),
                    req_op: RequestConsultationOperationMock {
                        account_id: 1,
                        consultant_id: 2,
                        fee_per_hour_in_yen: Some(5000),
                        tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
                        current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(7, 0, 1),
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(7, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(23, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 22)
                            .and_hms(7, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidConsultationDateTime as u32,
                    }),
                )),
            },
            TestCase {
                name: "second_candidate_in_jst IllegalConsultationDateTime".to_string(),
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
                            hour: 24,
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
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(7, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(23, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 22)
                            .and_hms(7, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalConsultationDateTime as u32,
                    }),
                )),
            },
            TestCase {
                name: "second_candidate_in_jst IllegalConsultationHour".to_string(),
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
                            hour: 0,
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
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(7, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(0, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 22)
                            .and_hms(7, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalConsultationHour as u32,
                    }),
                )),
            },
            TestCase {
                name: "second_candidate_in_jst InvalidConsultationDateTime".to_string(),
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
                            day: 22,
                            hour: 7,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 12,
                            hour: 7,
                        },
                    },
                    current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(6, 59, 59),
                    req_op: RequestConsultationOperationMock {
                        account_id: 1,
                        consultant_id: 2,
                        fee_per_hour_in_yen: Some(5000),
                        tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
                        current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(6, 59, 59),
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(7, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 22)
                            .and_hms(7, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 12)
                            .and_hms(7, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidConsultationDateTime as u32,
                    }),
                )),
            },
            TestCase {
                name: "third_candidate_in_jst IllegalConsultationDateTime".to_string(),
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
                            day: 21,
                            hour: 24,
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
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(7, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(23, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 21)
                            .and_hms(7, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalConsultationDateTime as u32,
                    }),
                )),
            },
            TestCase {
                name: "third_candidate_in_jst IllegalConsultationHour".to_string(),
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
                            hour: 6,
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
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(7, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(23, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 22)
                            .and_hms(6, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::IllegalConsultationHour as u32,
                    }),
                )),
            },
            TestCase {
                name: "third_candidate_in_jst InvalidConsultationDateTime".to_string(),
                input: Input {
                    account_id: 1,
                    param: RequestConsultationParam {
                        consultant_id: 2,
                        fee_per_hour_in_yen: 5000,
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        first_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 5,
                            hour: 7,
                        },
                        second_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 22,
                            hour: 7,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 4,
                            hour: 7,
                        },
                    },
                    current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(7, 0, 1),
                    req_op: RequestConsultationOperationMock {
                        account_id: 1,
                        consultant_id: 2,
                        fee_per_hour_in_yen: Some(5000),
                        tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
                        current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(7, 0, 1),
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 5)
                            .and_hms(7, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 22)
                            .and_hms(7, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(7, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::InvalidConsultationDateTime as u32,
                    }),
                )),
            },
            TestCase {
                name: "first_candidate_in_jst == second_candidate_in_jst".to_string(),
                input: Input {
                    account_id: 1,
                    param: RequestConsultationParam {
                        consultant_id: 2,
                        fee_per_hour_in_yen: 5000,
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        first_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 5,
                            hour: 7,
                        },
                        second_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 5,
                            hour: 7,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 4,
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
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 5)
                            .and_hms(7, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 5)
                            .and_hms(7, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(7, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::DuplicateDateTimeCandidates as u32,
                    }),
                )),
            },
            TestCase {
                name: "second_candidate_in_jst == third_candidate_in_jst".to_string(),
                input: Input {
                    account_id: 1,
                    param: RequestConsultationParam {
                        consultant_id: 2,
                        fee_per_hour_in_yen: 5000,
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        first_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 5,
                            hour: 7,
                        },
                        second_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 4,
                            hour: 18,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 4,
                            hour: 18,
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
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 5)
                            .and_hms(7, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(18, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(18, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::DuplicateDateTimeCandidates as u32,
                    }),
                )),
            },
            TestCase {
                name: "third_candidate_in_jst == first_candidate_in_jst".to_string(),
                input: Input {
                    account_id: 1,
                    param: RequestConsultationParam {
                        consultant_id: 2,
                        fee_per_hour_in_yen: 5000,
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        first_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 20,
                            hour: 21,
                        },
                        second_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 4,
                            hour: 18,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 20,
                            hour: 21,
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
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 20)
                            .and_hms(21, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(18, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 20)
                            .and_hms(21, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::DuplicateDateTimeCandidates as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail AccountDisabled".to_string(),
                input: Input {
                    account_id: 1,
                    param: RequestConsultationParam {
                        consultant_id: 2,
                        fee_per_hour_in_yen: 5000,
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        first_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 14,
                            hour: 21,
                        },
                        second_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 4,
                            hour: 18,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 20,
                            hour: 21,
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
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 14)
                            .and_hms(21, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(18, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 20)
                            .and_hms(21, 0, 0),
                        user_account_is_available: false,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::AccountDisabled as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail NoIdentityRegistered".to_string(),
                input: Input {
                    account_id: 1,
                    param: RequestConsultationParam {
                        consultant_id: 2,
                        fee_per_hour_in_yen: 5000,
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        first_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 14,
                            hour: 21,
                        },
                        second_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 4,
                            hour: 18,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 20,
                            hour: 21,
                        },
                    },
                    current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(7, 0, 0),
                    req_op: RequestConsultationOperationMock {
                        account_id: 3,
                        consultant_id: 2,
                        fee_per_hour_in_yen: Some(5000),
                        tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
                        current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(7, 0, 0),
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 14)
                            .and_hms(21, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(18, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 20)
                            .and_hms(21, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::NoIdentityRegistered as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail ConsultantIsNotAvailable".to_string(),
                input: Input {
                    account_id: 1,
                    param: RequestConsultationParam {
                        consultant_id: 2,
                        fee_per_hour_in_yen: 5000,
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        first_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 14,
                            hour: 21,
                        },
                        second_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 4,
                            hour: 18,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 20,
                            hour: 21,
                        },
                    },
                    current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(7, 0, 0),
                    req_op: RequestConsultationOperationMock {
                        account_id: 1,
                        consultant_id: 3,
                        fee_per_hour_in_yen: Some(5000),
                        tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
                        current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(7, 0, 0),
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 14)
                            .and_hms(21, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(18, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 20)
                            .and_hms(21, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::ConsultantIsNotAvailable as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail FeePerHourInYenWasUpdated".to_string(),
                input: Input {
                    account_id: 1,
                    param: RequestConsultationParam {
                        consultant_id: 2,
                        fee_per_hour_in_yen: 5000,
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        first_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 14,
                            hour: 21,
                        },
                        second_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 4,
                            hour: 18,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 20,
                            hour: 21,
                        },
                    },
                    current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(7, 0, 0),
                    req_op: RequestConsultationOperationMock {
                        account_id: 1,
                        consultant_id: 2,
                        fee_per_hour_in_yen: Some(6000),
                        tenant_id: Some("32ac9a3c14bf4404b0ef6941a95934ec".to_string()),
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        charge: create_dummy_charge("ch_fa990a4c10672a93053a774730b0a"),
                        current_date_time: JAPANESE_TIME_ZONE.ymd(2022, 11, 1).and_hms(7, 0, 0),
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 14)
                            .and_hms(21, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(18, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 20)
                            .and_hms(21, 0, 0),
                        user_account_is_available: true,
                        settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        stopped_settlement_payments: vec![PaymentInfo {
                            fee_per_hour_in_yen: 5000,
                            platform_fee_rate_in_percentage: "30.0".to_string(),
                        }],
                        receipt_payments: vec![
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                            PaymentInfo {
                                fee_per_hour_in_yen: 5000,
                                platform_fee_rate_in_percentage: "30.0".to_string(),
                            },
                        ],
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::FeePerHourInYenWasUpdated as u32,
                    }),
                )),
            },
            TestCase {
                name: "fail ExceedMaxAnnualRewards".to_string(),
                input: Input {
                    account_id: 1,
                    param: RequestConsultationParam {
                        consultant_id: 2,
                        fee_per_hour_in_yen: 5000,
                        card_token: "tok_76e202b409f3da51a0706605ac81".to_string(),
                        first_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 14,
                            hour: 21,
                        },
                        second_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 4,
                            hour: 18,
                        },
                        third_candidate_in_jst: ConsultationDateTime {
                            year: 2022,
                            month: 11,
                            day: 20,
                            hour: 21,
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
                        first_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 14)
                            .and_hms(21, 0, 0),
                        second_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 4)
                            .and_hms(18, 0, 0),
                        third_candidate_in_jst: JAPANESE_TIME_ZONE
                            .ymd(2022, 11, 20)
                            .and_hms(21, 0, 0),
                        user_account_is_available: true,
                        settlement_payments:
                            create_payments_that_has_reward_over_one_third_of_max_rewards(),
                        stopped_settlement_payments:
                            create_payments_that_has_reward_over_one_third_of_max_rewards(),
                        receipt_payments:
                            create_payments_that_has_reward_over_one_third_of_max_rewards(),
                    },
                },
                expected: Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: Code::ExceedMaxAnnualRewards as u32,
                    }),
                )),
            },
        ]
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

    fn create_payments_that_has_reward_over_one_third_of_max_rewards() -> Vec<PaymentInfo> {
        // *MAX_ANNUAL_REWARDS_IN_YENのおよそ三分の一
        let one_third_of_max = 156667;
        let mut result = vec![];
        let mut rewards = 0;
        while rewards < one_third_of_max {
            let p = PaymentInfo {
                fee_per_hour_in_yen: 3000,
                platform_fee_rate_in_percentage: "30.0".to_string(),
            };
            result.push(p);
            rewards += 2100;
        }
        result
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
