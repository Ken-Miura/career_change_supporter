// Copyright 2023 Ken Miura

use std::env;

use async_session::log::error;
use axum::http::StatusCode;
use axum::Json;
use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, FixedOffset};
use common::{ApiError, ErrResp, JAPANESE_TIME_ZONE};
use entity::{
    consultation,
    sea_orm::{DatabaseConnection, EntityTrait},
};
use hmac::{Hmac, Mac};
use once_cell::sync::Lazy;
use serde::Serialize;
use sha2::Sha256;

use crate::{
    err::{unexpected_err_resp, Code},
    util::request_consultation::LENGTH_OF_MEETING_IN_MINUTE,
};

pub(crate) mod consultant_side_info;
pub(crate) mod user_side_info;

pub(crate) const KEY_TO_SKY_WAY_SECRET_KEY: &str = "SKY_WAY_SECRET_KEY";
/// SkyWayのPeer生成に使うcredentialを生成する際に利用するキー
static SKY_WAY_SECRET_KEY: Lazy<String> = Lazy::new(|| {
    env::var(KEY_TO_SKY_WAY_SECRET_KEY).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_SKY_WAY_SECRET_KEY
        )
    })
});

/// [SkyWayCredential]のttlに使う値
///
/// 600 - 90000 の間を設定する必要がある
/// https://github.com/skyway/skyway-peer-authentication-samples/blob/master/README.jp.md#ttl
///
/// 相談時間（[LENGTH_OF_MEETING_IN_MINUTE]分）+ 余裕（20分）を設定し、必ず相談時間中にCredentialの期限が切れないようにする。
const SKY_WAY_CREDENTIAL_TTL_IN_SECONDS: u32 = 60 * (LENGTH_OF_MEETING_IN_MINUTE as u32 + 20);

/// SkyWayでPeer認証に用いるCredential
///
/// https://github.com/skyway/skyway-peer-authentication-samples/blob/master/README.jp.md
#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct SkyWayCredential {
    #[serde(rename = "authToken")]
    auth_token: String,
    ttl: u32,
    timestamp: i64,
}

/// 下記URLの仕様に沿ってauthTokenを生成する
/// https://github.com/skyway/skyway-peer-authentication-samples/blob/master/README.jp.md#authtoken
fn generate_sky_way_credential_auth_token(
    peer_id: &str,
    timestamp: i64,
    ttl: u32,
    sky_way_secret_key: &str,
) -> Result<String, ErrResp> {
    let content = format!("{}:{}:{}", timestamp, ttl, peer_id);
    let mut mac = Hmac::<Sha256>::new_from_slice(sky_way_secret_key.as_bytes()).map_err(|e| {
        error!("failed to create HMAC-SHA256 instance: {}", e);
        unexpected_err_resp()
    })?;
    mac.update(content.as_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    // Base64は、方式に関して標準 (+, /)、URLセーフ(-, _)の２つ、パディングに関して有りなしの２つの組み合わせで４パターンある
    // 公式のGolangの例を見ると、標準かつ、パディングありなのでそれに従う
    // https://github.com/skyway/skyway-peer-authentication-samples/blob/master/golang/sample.go#L99
    let encoded = general_purpose::STANDARD.encode(code_bytes);
    Ok(encoded)
}

#[derive(Clone, Debug)]
struct Consultation {
    user_account_id: i64,
    consultant_id: i64,
    consultation_date_time_in_jst: DateTime<FixedOffset>,
    user_account_peer_id: Option<String>,
    user_account_peer_opened_at_in_jst: Option<DateTime<FixedOffset>>,
    consultant_peer_id: Option<String>,
    consultant_peer_opend_at_in_jst: Option<DateTime<FixedOffset>>,
}

async fn find_consultation_by_consultation_id(
    consultation_id: i64,
    pool: &DatabaseConnection,
) -> Result<Option<Consultation>, ErrResp> {
    let model = consultation::Entity::find_by_id(consultation_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find consultation (consultation_id: {}): {}",
                consultation_id, e
            );
            unexpected_err_resp()
        })?;
    Ok(model.map(|m| Consultation {
        user_account_id: m.user_account_id,
        consultant_id: m.consultant_id,
        consultation_date_time_in_jst: m.meeting_at.with_timezone(&(*JAPANESE_TIME_ZONE)),
        user_account_peer_id: m.user_account_peer_id,
        user_account_peer_opened_at_in_jst: m
            .user_account_peer_opened_at
            .map(|u| u.with_timezone(&(*JAPANESE_TIME_ZONE))),
        consultant_peer_id: m.consultant_peer_id,
        consultant_peer_opend_at_in_jst: m
            .consultant_peer_opend_at
            .map(|c| c.with_timezone(&(*JAPANESE_TIME_ZONE))),
    }))
}

fn validate_consultation_id_is_positive(consultation_id: i64) -> Result<(), ErrResp> {
    if !consultation_id.is_positive() {
        error!("consultation_id ({}) is not positive", consultation_id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::NonPositiveConsultationId as u32,
            }),
        ));
    }
    Ok(())
}
