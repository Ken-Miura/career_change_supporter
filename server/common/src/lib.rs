// Copyright 2021 Ken Miura

// TODO: #[macro_use]なしでdieselのマクロが使えるように変更が入った際に取り除く
// https://github.com/diesel-rs/diesel/issues/1764
#[macro_use]
extern crate diesel;

mod err_code;
pub mod model;
pub mod payment;
pub mod redis;
pub mod schema;
pub mod smtp;
pub mod util;

use std::{env::var, fmt::Debug};

use axum::{
    async_trait, extract,
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
    BoxError, Json,
};

use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};
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

/// API呼び出しに対して、クライアントに返却するレスポンスを含む[Result]
///
/// [Ok]は、TをJSONとしてBodyに含める[Resp]を包含する。<br>
/// Tには、API呼び出しが成功したときに、レスポンスのBodyに含めたいJSONを示す型を代入する。<br>
/// <br>
/// [Err]は、[ApiError]をJSONとしてBodyに含める[ErrResp]を包含する。
pub type RespResult<T> = Result<Resp<T>, ErrResp>;

pub type ConnectionPool = Pool<ConnectionManager<PgConnection>>;

/// データベースへのコネクション
///
/// ハンドラ関数内でデータベースへのアクセスを行いたい場合、原則としてこの型をパラメータとして受け付ける。
/// ハンドラ内で複数のコネクションが必要な場合のみ、[axum::extract::Extension]<[ConnectionPool]>をパラメータとして受け付ける。
pub struct DatabaseConnection(pub PooledConnection<ConnectionManager<PgConnection>>);

#[async_trait]
impl<B> FromRequest<B> for DatabaseConnection
where
    B: Send,
{
    type Rejection = ErrResp;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<ConnectionPool>::from_request(req)
            .await
            .map_err(|e| {
                tracing::error!("failed to extract connection pool from req: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError {
                        code: err_code::UNEXPECTED_ERR,
                    }),
                )
            })?;
        let conn = pool.get().map_err(|e| {
            tracing::error!("failed to get connection from pool: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: err_code::UNEXPECTED_ERR,
                }),
            )
        })?;
        Ok(Self(conn))
    }
}

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
                        code: err_code::UNEXPECTED_ERR,
                    }),
                )
            })?;
        let cred = payload.0;
        let _ = util::validator::validate_email_address(&cred.email_address).map_err(|e| {
            tracing::error!("failed to validate credential: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: err_code::INVALID_EMAIL_ADDRESS_FORMAT,
                }),
            )
        })?;
        let _ = util::validator::validate_password(&cred.password).map_err(|e| {
            tracing::error!("failed to validate credential: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: err_code::INVALID_PASSWORD_FORMAT,
                }),
            )
        })?;
        Ok(Self(cred))
    }
}

/// dieselのトランザクション内で発生したエラー<br>
/// <br>
/// アプリケーションに関連するエラーの場合、[TransactionErr::ApplicationErr]を、
/// データベースアクセスに関連するエラー（diesel api呼び出しの結果のエラー）の場合、[TransactionErr::DatabaseErr]を利用する。<br>
/// <br>
/// NOTE: dieselのトランザクションは、エラーとしてFrom\<diesel::result::Error\>を実装した型を要求しているため、ErrRespを直接返却することができない。
/// そのため、[TransactionErr]が必要となる。
pub enum TransactionErr {
    ApplicationErr(ErrResp),
    DatabaseErr(diesel::result::Error),
}

impl From<diesel::result::Error> for TransactionErr {
    fn from(e: diesel::result::Error) -> Self {
        TransactionErr::DatabaseErr(e)
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

/// 時間単位での一時アカウントの有効期限<br>
/// [VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR] 丁度の期間は有効期限に含む
pub const VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR: i64 = 24;

/// 分単位での新規パスワードの有効期限<br>
/// [VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE] 丁度の期間は有効期限に含む
pub const VALID_PERIOD_OF_NEW_PASSWORD_IN_MINUTE: i64 = 10;
