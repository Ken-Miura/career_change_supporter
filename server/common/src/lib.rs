// Copyright 2021 Ken Miura

// TODO: #[macro_use]なしでdieselのマクロが使えるように変更が入った際に取り除く
// https://github.com/diesel-rs/diesel/issues/1764
#[macro_use]
extern crate diesel;

mod err_code;
pub mod model;
pub mod schema;
pub mod util;

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
use serde::Deserialize;
use serde::Serialize;

/// 任意のステータスコードを指定可能で、BodyにJSONを含むレスポンス
pub type JsonResp<T> = (StatusCode, Json<T>);

/// [Ok]と[Err]の両方で、[JsonResp]を返却する[Result]
///
/// Sには、[Ok]のときにレスポンスのBodyに含めるJSONを示す型を代入する。
/// [Err]のときは、[ApiError]をJSONとしてBodyに含める。
pub type JsonRespResult<S> = Result<JsonResp<S>, JsonResp<ApiError>>;

/// API呼び出しに失敗した際、その理由を示すエラーコード
///
/// [JsonRespResult]で[Err]を返却する際、JSONとしてBodyに含める。
#[derive(Serialize)]
pub struct ApiError {
    pub code: u32,
}

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
    type Rejection = JsonResp<ApiError>;

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
    type Rejection = JsonResp<ApiError>;

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
