// Copyright 2023 Ken Miura

use std::env;

use async_session::log::error;
use base64::{engine::general_purpose, Engine};
use common::ErrResp;
use hmac::{Hmac, Mac};
use once_cell::sync::Lazy;
use serde::Serialize;
use sha2::Sha256;

use crate::err::unexpected_err_resp;

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