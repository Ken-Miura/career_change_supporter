// Copyright 2023 Ken Miura

use std::env;

use async_session::log::error;
use axum::http::StatusCode;
use axum::Json;
use base64::{engine::general_purpose, Engine};
use common::{ApiError, ErrResp};
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
