// Copyright 2021 Ken Miura

pub mod err;
pub mod payment_platform;
pub mod redis;
pub mod smtp;
pub mod storage;
pub mod util;

use std::{
    env::var,
    error::Error,
    fmt::{Debug, Display},
};

use axum::{
    async_trait, extract,
    extract::{FromRequest, RequestParts},
    http::StatusCode,
    BoxError, Json,
};
use chrono::FixedOffset;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;

/// 任意のステータスコードを指定可能で、BodyにJSONを含むレスポンス
pub type Resp<T> = (StatusCode, Json<T>);

/// 任意のステータスコードを指定可能で、Bodyに[ApiError]をJSONとして含むレスポンス
pub type ErrResp = Resp<ApiError>;

/// API呼び出しに失敗した際のエラー
///
/// メンバー[`Self::code`]に、エラーの理由を示すコードを含む。
#[derive(Serialize, Debug)]
pub struct ApiError {
    pub code: u32,
}

/// [ErrResp]を包含し、[Error]を実装した構造体
///
/// [ErrResp]の実体がtupleで[Error]を実装できない。そのため、[Error]としての型が必要な箇所で[ErrResp]を扱う際に利用する。
#[derive(Debug)]
pub struct ErrRespStruct {
    pub err_resp: ErrResp,
}

impl Display for ErrRespStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "status code: {}, error code: {}",
            self.err_resp.0, self.err_resp.1.code
        )
    }
}

impl Error for ErrRespStruct {}

/// API呼び出しに対して、クライアントに返却するレスポンスを含む[Result]
///
/// [Ok]は、TをJSONとしてBodyに含める[Resp]を包含する。<br>
/// Tには、API呼び出しが成功したときに、レスポンスのBodyに含めたいJSONを示す型を代入する。<br>
/// <br>
/// [Err]は、[ApiError]をJSONとしてBodyに含める[ErrResp]を包含する。
pub type RespResult<T> = Result<Resp<T>, ErrResp>;

/// ログインのために利用するCredential
///
/// ハンドラ関数のパラメータとしてCredentialを受け付けたい場合、妥当性確認済で渡されてくる[ValidCred]を利用し、原則としてこの型を直接パラメータとして指定しない。
/// この型をパラメータとして指定する場合は、ユーザから渡される値のため、利用前に必ず値を妥当性を確認する。
/// 値を確認る際には、[crate::util::validator::validate_email_address()]と[crate::util::validator::validate_password()]を使う。
#[derive(Clone, Deserialize)]
pub struct Credential {
    pub email_address: String,
    pub password: String,
}

/// 妥当性確認済みの[Credential]
pub struct ValidCred(pub Credential);

#[async_trait]
impl<B> FromRequest<B> for ValidCred
where
    B: http_body::Body + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = ErrResp;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let payload = extract::Json::<Credential>::from_request(req)
            .await
            .map_err(|e| {
                tracing::error!("failed to extract credential from req: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError {
                        code: err::Code::UnexpectedErr as u32,
                    }),
                )
            })?;
        let cred = payload.0;
        let _ = util::validator::validate_email_address(&cred.email_address).map_err(|e| {
            tracing::error!("failed to validate credential: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: err::Code::InvalidEmailAddressFormat as u32,
                }),
            )
        })?;
        let _ = util::validator::validate_password(&cred.password).map_err(|e| {
            tracing::error!("failed to validate credential: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: err::Code::InvalidPasswordFormat as u32,
                }),
            )
        })?;
        Ok(Self(cred))
    }
}

pub const KEY_TO_URL_FOR_FRONT_END: &str = "URL_FOR_FRONT_END";
pub static URL_FOR_FRONT_END: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_URL_FOR_FRONT_END).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" (example value: \"https://localhost:8080\") must be set",
            KEY_TO_URL_FOR_FRONT_END
        );
    })
});

/// UTCにおける日本のタイムゾーン（正確には、UTCで日本時間を表すためのオフセットだが、タイムゾーンと同等の意味で利用）
/// [chrono::DateTime] で日本時間を扱う際に利用する。
pub static JAPANESE_TIME_ZONE: Lazy<FixedOffset> = Lazy::new(|| FixedOffset::east(9 * 3600));

/// 時間単位での一時アカウントの有効期限<br>
/// [VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR] 丁度の期間は有効期限に含む
pub const VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR: i64 = 24;

/// 分単位でのパスワード変更要求の有効期限<br>
/// [VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE] 丁度の期間は有効期限に含む
pub const VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE: i64 = 10;

/// 1アカウント当たりに登録可能な職務経歴情報の最大数
pub const MAX_NUM_OF_CAREER_INFO_PER_USER_ACCOUNT: u64 = 8;
