// Copyright 2022 Ken Miura

use axum::{http::StatusCode, Json};
use chrono::{DateTime, FixedOffset};
use common::{ApiError, ErrResp, JAPANESE_TIME_ZONE};
use entity::sea_orm::{DatabaseConnection, EntityTrait};
use tracing::error;

use crate::err::{unexpected_err_resp, Code};

/// 相談申し込み
#[derive(Clone, Debug)]
pub(crate) struct ConsultationRequest {
    pub(crate) consultation_req_id: i64,
    pub(crate) user_account_id: i64,
    pub(crate) consultant_id: i64,
    pub(crate) fee_per_hour_in_yen: i32,
    pub(crate) first_candidate_date_time_in_jst: DateTime<FixedOffset>,
    pub(crate) second_candidate_date_time_in_jst: DateTime<FixedOffset>,
    pub(crate) third_candidate_date_time_in_jst: DateTime<FixedOffset>,
    pub(crate) charge_id: String,
    pub(crate) latest_candidate_date_time_in_jst: DateTime<FixedOffset>,
}

/// 相談申し込みを取得する
///
/// 取得した相談申し込みは、consultant_idがリクエスト送信元のユーザーIDと一致するか（操作可能なユーザーか）必ずチェックする
pub(crate) async fn find_consultation_req_by_consultation_req_id(
    pool: &DatabaseConnection,
    consultation_req_id: i64,
) -> Result<Option<ConsultationRequest>, ErrResp> {
    let model = entity::prelude::ConsultationReq::find_by_id(consultation_req_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find consultation_req (consultation_req_id: {}): {}",
                consultation_req_id, e
            );
            unexpected_err_resp()
        })?;
    Ok(model.map(|m| ConsultationRequest {
        consultation_req_id: m.consultation_req_id,
        user_account_id: m.user_account_id,
        consultant_id: m.consultant_id,
        fee_per_hour_in_yen: m.fee_per_hour_in_yen,
        first_candidate_date_time_in_jst: m
            .first_candidate_date_time
            .with_timezone(&(*JAPANESE_TIME_ZONE)),
        second_candidate_date_time_in_jst: m
            .second_candidate_date_time
            .with_timezone(&(*JAPANESE_TIME_ZONE)),
        third_candidate_date_time_in_jst: m
            .third_candidate_date_time
            .with_timezone(&(*JAPANESE_TIME_ZONE)),
        charge_id: m.charge_id,
        latest_candidate_date_time_in_jst: m
            .latest_candidate_date_time
            .with_timezone(&(*JAPANESE_TIME_ZONE)),
    }))
}

/// 取得した相談申し込みの存在確認をする
pub(crate) fn consultation_req_exists(
    consultation_request: Option<ConsultationRequest>,
    consultation_req_id: i64,
) -> Result<ConsultationRequest, ErrResp> {
    let req = consultation_request.ok_or_else(|| {
        error!(
            "no consultation_req (consultation_req_id: {}) found",
            consultation_req_id
        );
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NoConsultationReqFound as u32,
            }),
        )
    })?;
    Ok(req)
}
