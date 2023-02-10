// Copyright 2023 Ken Miura

use std::env;

use async_session::log::error;
use axum::http::StatusCode;
use axum::Json;
use base64::{engine::general_purpose, Engine};
use chrono::{DateTime, Duration, FixedOffset};
use common::{
    util::validator::uuid_validator::validate_uuid, ApiError, ErrResp, ErrRespStruct,
    JAPANESE_TIME_ZONE,
};
use entity::{
    consultation,
    sea_orm::{DatabaseConnection, DatabaseTransaction, EntityTrait, QuerySelect},
};
use hmac::{Hmac, Mac};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use once_cell::sync::Lazy;
use serde::Serialize;
use sha2::Sha256;

use crate::{
    err::{unexpected_err_resp, Code},
    util::request_consultation::LENGTH_OF_MEETING_IN_MINUTE,
};

pub(crate) mod consultant_side_info;
pub(crate) mod user_side_info;

const LEEWAY_IN_MINUTES: i64 = 5;

pub(crate) const KEY_TO_SKY_WAY_APPLICATION_ID: &str = "SKY_WAY_APPLICATION_ID";
static SKY_WAY_APPLICATION_ID: Lazy<String> = Lazy::new(|| {
    env::var(KEY_TO_SKY_WAY_APPLICATION_ID).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_SKY_WAY_APPLICATION_ID
        )
    })
});

pub(crate) const KEY_TO_SKY_WAY_SECRET_KEY: &str = "SKY_WAY_SECRET_KEY";
static SKY_WAY_SECRET_KEY: Lazy<String> = Lazy::new(|| {
    env::var(KEY_TO_SKY_WAY_SECRET_KEY).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_SKY_WAY_SECRET_KEY
        )
    })
});

struct SkyWayIdentification {
    application_id: String,
    secret: String,
}

/// [SkyWayAuthTokenPayload]のexpを生成するために使う値 ([SkyWayAuthToken]が有効な期間)
///
///
/// 相談時間（[LENGTH_OF_MEETING_IN_MINUTE]分）+ 相談開始時刻前から入室可能な分の余裕（[LEEWAY_IN_MINUTES]分) + 余裕（5分）を設定し、
/// 必ず相談時間中に期限が切れないようにする。
const VALID_TOKEN_DURATION_IN_SECONDS: i64 =
    60 * (LENGTH_OF_MEETING_IN_MINUTE as i64 + LEEWAY_IN_MINUTES + 5);

// クライアントがSkay WayにアクセスするためのJWTのペイロード部分を表す構造体
// このサービスに必要な分のメンバーのみを定義する
//
// https://skyway.ntt.com/ja/docs/user-guide/authentication/
#[derive(Clone, Debug, Serialize, PartialEq)]
struct SkyWayAuthTokenPayload {
    iat: i64,    // 秒単位のタイムスタンプ（DateTime<FixedOffset>.timestamp()で取得できる値）
    jti: String, // UUID V4（6668affc-5afa-4996-b65a-6afe2f72756b のようなハイフン有り形式）
    exp: i64,    // 秒単位のタイムスタンプ（DateTime<FixedOffset>.timestamp()で取得できる値）
    scope: SkyWayScope,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
struct SkyWayScope {
    app: SkyWayAppScope,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
struct SkyWayAppScope {
    id: String,           // アプリケーションID
    actions: Vec<String>, // 使える値はreadのみ
    channels: Vec<SkyWayChannelScope>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
struct SkyWayChannelScope {
    name: String, // idまたはnameのどちらかの指定が必須。このサービスではnameを指定する
    actions: Vec<String>, // 使える値はwrite, read, create, delete, updateMetadataの5つ。このサービスでは必要なread, create, deleteのみを指定する。
    members: Vec<SkyWayMemberScope>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
struct SkyWayMemberScope {
    name: String, // idまたはnameのどちらかの指定が必須。このサービスではnameを指定する
    actions: Vec<String>, // 使える値はwrite, create, delete, updateMetadata, signalの5つ。このサービスでは必要なcreate, delete, signalのみを指定する。
    publication: SkyWayPublicationScope,
    subscription: SkyWaySubscriptionScope,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
struct SkyWayPublicationScope {
    actions: Vec<String>, // 使える値はwrite, create, delete, updateMetadata, enable, disableの6つ。このサービスでは必要なcreate, deleteのみを指定する。
}

#[derive(Clone, Debug, Serialize, PartialEq)]
struct SkyWaySubscriptionScope {
    // 使える値はwrite, create, deleteの3つ。このサービスではcreate, deleteを指定する。
    // （2023年2月時点では、このリソースに関してcreate, deleteを指定することはwriteと同じだが、他と合わせるためにcreate, deleteを指定）
    actions: Vec<String>,
}

const MAX_DURATION_IN_SECONDS: i64 = 60 * 60 * 24 * 3;

fn create_sky_way_auth_token_payload(
    token_id: String,
    current_date_time: DateTime<FixedOffset>,
    expiration_date_time: DateTime<FixedOffset>,
    application_id: String,
    room_name: String,
    member_name: String,
) -> Result<SkyWayAuthTokenPayload, ErrResp> {
    let duration = Duration::seconds(MAX_DURATION_IN_SECONDS);
    let criteria = current_date_time + duration;
    if criteria > expiration_date_time {
        error!(
            "current_date_time ({}) over expiration_date_time ({})",
            current_date_time, expiration_date_time
        );
        return Err(unexpected_err_resp());
    }
    validate_uuid(room_name.as_str()).map_err(|e| {
        error!(
            "failed to validate room name (UUID v4 simple format) ({}): {}",
            room_name, e
        );
        unexpected_err_resp()
    })?;
    Ok(SkyWayAuthTokenPayload {
        iat: current_date_time.timestamp(),
        jti: token_id,
        exp: expiration_date_time.timestamp(),
        scope: SkyWayScope {
            app: SkyWayAppScope {
                id: application_id,
                actions: vec!["read".to_string()],
                channels: vec![SkyWayChannelScope {
                    name: room_name,
                    actions: vec![
                        "read".to_string(),
                        "create".to_string(),
                        "delete".to_string(),
                    ],
                    members: vec![SkyWayMemberScope {
                        name: member_name,
                        actions: vec![
                            "create".to_string(),
                            "delete".to_string(),
                            "signal".to_string(),
                        ],
                        publication: SkyWayPublicationScope {
                            actions: vec!["create".to_string(), "delete".to_string()],
                        },
                        subscription: SkyWaySubscriptionScope {
                            actions: vec!["create".to_string(), "delete".to_string()],
                        },
                    }],
                }],
            },
        },
    })
}

fn create_sky_way_auth_token(
    payload: &SkyWayAuthTokenPayload,
    secret: &[u8],
) -> Result<String, ErrResp> {
    let header = Header {
        alg: Algorithm::HS512,
        typ: Some("JWT".to_string()),
        cty: None,
        jku: None,
        jwk: None,
        kid: None,
        x5u: None,
        x5c: None,
        x5t: None,
        x5t_s256: None,
    };
    let token = encode(&header, payload, &EncodingKey::from_secret(secret)).map_err(|e| {
        error!(
            "failed to encode to jwt (header: {:?}, payload: {:?}): {}",
            header, payload, e
        );
        unexpected_err_resp()
    })?;
    Ok(token)
}

async fn find_room_name_by_consultation_id(
    pool: &DatabaseConnection,
    consultation_id: i64,
) -> Result<Option<String>, ErrResp> {
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
    Ok(model.map(|m| m.room_name))
}

// ここより下は旧SkyWay用の必要ないコード

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

fn create_sky_way_credential(peer_id: &str, timestamp: i64) -> Result<SkyWayCredential, ErrResp> {
    todo!()
    // let auth_token = generate_sky_way_credential_auth_token(
    //     peer_id,
    //     timestamp,
    //     SKY_WAY_CREDENTIAL_TTL_IN_SECONDS,
    //     "",
    // )?;
    // Ok(SkyWayCredential {
    //     auth_token,
    //     ttl: SKY_WAY_CREDENTIAL_TTL_IN_SECONDS,
    //     timestamp,
    // })
}

#[derive(Clone, Debug)]
struct Consultation {
    user_account_id: i64,
    consultant_id: i64,
    consultation_date_time_in_jst: DateTime<FixedOffset>,
    user_account_peer_id: Option<String>,
    consultant_peer_id: Option<String>,
}

async fn find_consultation_by_consultation_id(
    consultation_id: i64,
    pool: &DatabaseConnection,
) -> Result<Option<Consultation>, ErrResp> {
    todo!()
    // let model = consultation::Entity::find_by_id(consultation_id)
    //     .one(pool)
    //     .await
    //     .map_err(|e| {
    //         error!(
    //             "failed to find consultation (consultation_id: {}): {}",
    //             consultation_id, e
    //         );
    //         unexpected_err_resp()
    //     })?;
    // Ok(model.map(|m| Consultation {
    //     user_account_id: m.user_account_id,
    //     consultant_id: m.consultant_id,
    //     consultation_date_time_in_jst: m.meeting_at.with_timezone(&(*JAPANESE_TIME_ZONE)),
    //     user_account_peer_id: m.user_account_peer_id,
    //     consultant_peer_id: m.consultant_peer_id,
    // }))
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

async fn get_consultation_with_exclusive_lock(
    consultation_id: i64,
    txn: &DatabaseTransaction,
) -> Result<consultation::Model, ErrRespStruct> {
    let result = consultation::Entity::find_by_id(consultation_id)
        .lock_exclusive()
        .one(txn)
        .await
        .map_err(|e| {
            error!(
                "failed to find consultation (consultation_id: {}): {}",
                consultation_id, e
            );
            ErrRespStruct {
                err_resp: unexpected_err_resp(),
            }
        })?;
    result.ok_or_else(|| {
        error!(
            "failed to get consultation (consultation_id: {})",
            consultation_id
        );
        ErrRespStruct {
            err_resp: unexpected_err_resp(),
        }
    })
}
