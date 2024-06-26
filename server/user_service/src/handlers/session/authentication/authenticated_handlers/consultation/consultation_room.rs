// Copyright 2023 Ken Miura

use std::env;

use async_session::log::error;
use axum::http::StatusCode;
use axum::Json;
use chrono::{DateTime, Duration, FixedOffset};
use common::{
    util::validator::uuid_validator::validate_uuid, ApiError, ErrResp, ErrRespStruct,
    JAPANESE_TIME_ZONE,
};
use entity::{
    consultation,
    sea_orm::{DatabaseConnection, DatabaseTransaction, EntityTrait, QuerySelect},
};
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use once_cell::sync::Lazy;
use serde::Serialize;

use crate::{
    err::{unexpected_err_resp, Code},
    handlers::session::authentication::LENGTH_OF_MEETING_IN_MINUTE,
    optional_env_var::CHECK_IF_CONSULTATION_ROOM_IS_OPENED,
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

#[derive(Clone, Debug)]
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
    jti: String, // UUID V4（6668affc-5afa-4996-b65a-6afe2f72756b のようなハイフン有り形式）
    iat: i64,    // 秒単位のタイムスタンプ（DateTime<FixedOffset>.timestamp()で取得できる値）
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

const MAX_DURATION_ON_SKY_WAY_API_IN_SECONDS: i64 = 60 * 60 * 24 * 3;

fn create_sky_way_auth_token_payload(
    token_id: String,
    current_date_time: DateTime<FixedOffset>,
    expiration_date_time: DateTime<FixedOffset>,
    application_id: String,
    room_name: String,
    member_name: String,
) -> Result<SkyWayAuthTokenPayload, ErrResp> {
    if current_date_time >= expiration_date_time {
        error!(
            "current_date_time ({}) is equal to or exceeds expiration_date_time ({})",
            current_date_time, expiration_date_time
        );
        return Err(unexpected_err_resp());
    }
    let duration = Duration::seconds(MAX_DURATION_ON_SKY_WAY_API_IN_SECONDS);
    let max_exp = current_date_time + duration;
    if expiration_date_time > max_exp {
        error!(
            "expiration_date_time ({}) exceeds max_exp ({})",
            expiration_date_time, max_exp
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
        alg: Algorithm::HS256,
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

fn ensure_audio_test_is_done(audio_test_done: bool) -> Result<(), ErrResp> {
    if !audio_test_done {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::AudioTestIsNotDone as u32,
            }),
        ));
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct Consultation {
    user_account_id: i64,
    consultant_id: i64,
    consultation_date_time_in_jst: DateTime<FixedOffset>,
    room_name: String,
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
        room_name: m.room_name,
    }))
}

async fn exist_awaiting_payment(
    consultation_id: i64,
    pool: &DatabaseConnection,
) -> Result<bool, ErrResp> {
    let model = entity::awaiting_payment::Entity::find_by_id(consultation_id)
        .one(pool)
        .await
        .map_err(|e| {
            error!(
                "failed to find awaiting_payment (consultation_id: {}): {}",
                consultation_id, e
            );
            unexpected_err_resp()
        })?;
    Ok(model.is_some())
}

fn ensure_consultation_room_can_be_opened(
    current_date_time: &DateTime<FixedOffset>,
    consultation_date_time_in_jst: &DateTime<FixedOffset>,
) -> Result<(), ErrResp> {
    if !(*CHECK_IF_CONSULTATION_ROOM_IS_OPENED) {
        return Ok(());
    }
    let leeway = Duration::minutes(LEEWAY_IN_MINUTES);
    let start_criteria = *consultation_date_time_in_jst - leeway;
    if *current_date_time < start_criteria {
        error!("consultation room has not opened yet (current_date_time: {}, consultation_date_time_in_jst: {}, leeway: {})", 
            current_date_time, consultation_date_time_in_jst, leeway);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultationRoomHasNotOpenedYet as u32,
            }),
        ));
    }
    let end_criteria =
        *consultation_date_time_in_jst + Duration::minutes(LENGTH_OF_MEETING_IN_MINUTE as i64);
    if *current_date_time > end_criteria {
        error!("consultation room has already closed (current_date_time: {}, consultation_date_time_in_jst: {}, LENGTH_OF_MEETING_IN_MINUTE: {})", 
            current_date_time, consultation_date_time_in_jst, LENGTH_OF_MEETING_IN_MINUTE);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: Code::ConsultationRoomHasAlreadyClosed as u32,
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

#[cfg(test)]
mod tests {

    use chrono::TimeZone;

    use super::*;

    pub(super) const TOKEN_ID: &str = "6668affc-5afa-4996-b65a-6afe2f72756b";
    pub(super) const DUMMY_APPLICATION_ID: &str = "fb374e11-742b-454e-a313-17d3207d41f6";
    pub(super) const DUMMY_SECRET: &str = "C7AV42GBs7jCJ4pmk5Jt9iyXNxN/h99um=";
    pub(super) const ROOM_NAME: &str = "187313a8d6cf41bc963d71d4bfd5f363";
    pub(super) const MEMBER_NAME: &str = "234";
    pub(super) const TOKEN: &str = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJqdGkiOiI2NjY4YWZmYy01YWZhLTQ5OTYtYjY1YS02YWZlMmY3Mjc1NmIiLCJpYXQiOjE2NzY4MDk5NDEsImV4cCI6MTY3NjgxNDE0MSwic2NvcGUiOnsiYXBwIjp7ImlkIjoiZmIzNzRlMTEtNzQyYi00NTRlLWEzMTMtMTdkMzIwN2Q0MWY2IiwiYWN0aW9ucyI6WyJyZWFkIl0sImNoYW5uZWxzIjpbeyJuYW1lIjoiMTg3MzEzYThkNmNmNDFiYzk2M2Q3MWQ0YmZkNWYzNjMiLCJhY3Rpb25zIjpbInJlYWQiLCJjcmVhdGUiLCJkZWxldGUiXSwibWVtYmVycyI6W3sibmFtZSI6IjIzNCIsImFjdGlvbnMiOlsiY3JlYXRlIiwiZGVsZXRlIiwic2lnbmFsIl0sInB1YmxpY2F0aW9uIjp7ImFjdGlvbnMiOlsiY3JlYXRlIiwiZGVsZXRlIl19LCJzdWJzY3JpcHRpb24iOnsiYWN0aW9ucyI6WyJjcmVhdGUiLCJkZWxldGUiXX19XX1dfX19.qb1VLQwK2WMzmjilEG0eBO0_cqF9Xq1saxbaP0toqDg";
    pub(super) static CURRENT_DATE_TIME: Lazy<DateTime<FixedOffset>> = Lazy::new(|| {
        JAPANESE_TIME_ZONE
            .with_ymd_and_hms(2023, 2, 19, 21, 32, 21)
            .unwrap()
    });

    #[test]
    fn test_create_sky_way_auth_token_payload_success1() {
        let expiration_date_time =
            *CURRENT_DATE_TIME + Duration::seconds(VALID_TOKEN_DURATION_IN_SECONDS);

        let result = create_sky_way_auth_token_payload(
            TOKEN_ID.to_string(),
            *CURRENT_DATE_TIME,
            expiration_date_time,
            DUMMY_APPLICATION_ID.to_string(),
            ROOM_NAME.to_string(),
            MEMBER_NAME.to_string(),
        )
        .expect("failed to get Ok");

        let expected_result = SkyWayAuthTokenPayload {
            jti: TOKEN_ID.to_string(),
            iat: (*CURRENT_DATE_TIME).timestamp(),
            exp: expiration_date_time.timestamp(),
            scope: SkyWayScope {
                app: SkyWayAppScope {
                    id: DUMMY_APPLICATION_ID.to_string(),
                    actions: vec!["read".to_string()],
                    channels: vec![SkyWayChannelScope {
                        name: ROOM_NAME.to_string(),
                        actions: vec![
                            "read".to_string(),
                            "create".to_string(),
                            "delete".to_string(),
                        ],
                        members: vec![SkyWayMemberScope {
                            name: MEMBER_NAME.to_string(),
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
        };

        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_create_sky_way_auth_token_payload_success2() {
        let expiration_date_time =
            *CURRENT_DATE_TIME + Duration::seconds(MAX_DURATION_ON_SKY_WAY_API_IN_SECONDS);

        let result = create_sky_way_auth_token_payload(
            TOKEN_ID.to_string(),
            *CURRENT_DATE_TIME,
            expiration_date_time,
            DUMMY_APPLICATION_ID.to_string(),
            ROOM_NAME.to_string(),
            MEMBER_NAME.to_string(),
        )
        .expect("failed to get Ok");

        let expected_result = SkyWayAuthTokenPayload {
            jti: TOKEN_ID.to_string(),
            iat: (*CURRENT_DATE_TIME).timestamp(),
            exp: expiration_date_time.timestamp(),
            scope: SkyWayScope {
                app: SkyWayAppScope {
                    id: DUMMY_APPLICATION_ID.to_string(),
                    actions: vec!["read".to_string()],
                    channels: vec![SkyWayChannelScope {
                        name: ROOM_NAME.to_string(),
                        actions: vec![
                            "read".to_string(),
                            "create".to_string(),
                            "delete".to_string(),
                        ],
                        members: vec![SkyWayMemberScope {
                            name: MEMBER_NAME.to_string(),
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
        };

        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_create_sky_way_auth_token_payload_fail_current_date_time_equal_expiration_date_time() {
        let expiration_date_time = *CURRENT_DATE_TIME;

        let result = create_sky_way_auth_token_payload(
            TOKEN_ID.to_string(),
            *CURRENT_DATE_TIME,
            expiration_date_time,
            DUMMY_APPLICATION_ID.to_string(),
            ROOM_NAME.to_string(),
            MEMBER_NAME.to_string(),
        )
        .expect_err("failed to get Err");

        let expected_result = unexpected_err_resp();

        assert_eq!(result.0, expected_result.0);
        assert_eq!(result.1 .0, expected_result.1 .0);
    }

    #[test]
    fn test_create_sky_way_auth_token_payload_fail_current_date_time_exceeds_expiration_date_time()
    {
        let expiration_date_time = *CURRENT_DATE_TIME - Duration::seconds(1);

        let result = create_sky_way_auth_token_payload(
            TOKEN_ID.to_string(),
            *CURRENT_DATE_TIME,
            expiration_date_time,
            DUMMY_APPLICATION_ID.to_string(),
            ROOM_NAME.to_string(),
            MEMBER_NAME.to_string(),
        )
        .expect_err("failed to get Err");

        let expected_result = unexpected_err_resp();

        assert_eq!(result.0, expected_result.0);
        assert_eq!(result.1 .0, expected_result.1 .0);
    }

    #[test]
    fn test_create_sky_way_auth_token_payload_fai_expiration_date_time_exceeds_sky_way_service_limit(
    ) {
        let expiration_date_time = *CURRENT_DATE_TIME
            + Duration::seconds(MAX_DURATION_ON_SKY_WAY_API_IN_SECONDS)
            + Duration::seconds(1);

        let result = create_sky_way_auth_token_payload(
            TOKEN_ID.to_string(),
            *CURRENT_DATE_TIME,
            expiration_date_time,
            DUMMY_APPLICATION_ID.to_string(),
            ROOM_NAME.to_string(),
            MEMBER_NAME.to_string(),
        )
        .expect_err("failed to get Err");

        let expected_result = unexpected_err_resp();

        assert_eq!(result.0, expected_result.0);
        assert_eq!(result.1 .0, expected_result.1 .0);
    }

    #[test]
    fn test_create_sky_way_auth_token_payload_fai_invalid_room_name() {
        let expiration_date_time =
            *CURRENT_DATE_TIME + Duration::seconds(VALID_TOKEN_DURATION_IN_SECONDS);
        let room_name = "test room".to_string(); // non UUID v4 simple format

        let result = create_sky_way_auth_token_payload(
            TOKEN_ID.to_string(),
            *CURRENT_DATE_TIME,
            expiration_date_time,
            DUMMY_APPLICATION_ID.to_string(),
            room_name,
            MEMBER_NAME.to_string(),
        )
        .expect_err("failed to get Err");

        let expected_result = unexpected_err_resp();

        assert_eq!(result.0, expected_result.0);
        assert_eq!(result.1 .0, expected_result.1 .0);
    }

    #[test]
    fn test_create_sky_way_auth_token_success() {
        let expiration_date_time =
            *CURRENT_DATE_TIME + Duration::seconds(VALID_TOKEN_DURATION_IN_SECONDS);

        let payload = create_sky_way_auth_token_payload(
            TOKEN_ID.to_string(),
            *CURRENT_DATE_TIME,
            expiration_date_time,
            DUMMY_APPLICATION_ID.to_string(),
            ROOM_NAME.to_string(),
            MEMBER_NAME.to_string(),
        )
        .expect("failed to get Ok");

        let result =
            create_sky_way_auth_token(&payload, DUMMY_SECRET.as_bytes()).expect("failed to get Ok");

        let expected_result = TOKEN;
        assert_eq!(result, expected_result);
    }
}
