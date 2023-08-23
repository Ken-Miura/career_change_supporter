// Copyright 2021 Ken Miura

pub mod db;
pub mod err;
pub mod log;
pub mod mfa;
pub mod opensearch;
pub mod password;
pub mod payment_platform;
pub mod rating;
pub mod redis;
pub mod smtp;
pub mod storage;
pub mod util;

use std::{
    env::var,
    error::Error,
    fmt::{Debug, Display},
};

use ::opensearch::OpenSearch;
use async_fred_session::RedisSessionStore;
use axum::{
    async_trait,
    body::{Body, HttpBody},
    extract::FromRequest,
    extract::{self, FromRef},
    http::{Request, StatusCode},
    BoxError, Json,
};
use axum_extra::extract::cookie::Key;
use chrono::FixedOffset;
use entity::sea_orm::DatabaseConnection;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;
use smtp::SmtpClient;
use storage::StorageClient;
use tracing::error;

/// ユーザーに公開するサイト名
pub const WEB_SITE_NAME: &str = "就職先・転職先を見極めるためのサイト";

/// 任意のステータスコードを指定可能で、BodyにJSONを含むレスポンス
pub type Resp<T> = (StatusCode, Json<T>);

/// 任意のステータスコードを指定可能で、Bodyに[ApiError]をJSONとして含むレスポンス
pub type ErrResp = Resp<ApiError>;

/// API呼び出しに失敗した際のエラー
///
/// メンバー[`Self::code`]に、エラーの理由を示すコードを含む。
#[derive(Serialize, Debug, Clone, PartialEq, Eq)]
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
impl<S, B> FromRequest<S, B> for ValidCred
where
    S: Send + Sync,
    B: HttpBody + Send + 'static,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = ErrResp;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let payload = extract::Json::<Credential>::from_request(req, state)
            .await
            .map_err(|e| {
                error!("failed to extract credential from req: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiError {
                        code: err::Code::UnexpectedErr as u32,
                    }),
                )
            })?;
        let cred = payload.0;
        let _ =
            util::validator::email_address_validator::validate_email_address(&cred.email_address)
                .map_err(|e| {
                error!("failed to validate credential: {}", e);
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: err::Code::InvalidEmailAddressFormat as u32,
                    }),
                )
            })?;
        let _ = util::validator::password_validator::validate_password(&cred.password).map_err(
            |e| {
                error!("failed to validate credential: {}", e);
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiError {
                        code: err::Code::InvalidPasswordFormat as u32,
                    }),
                )
            },
        )?;
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
pub static JAPANESE_TIME_ZONE: Lazy<FixedOffset> =
    Lazy::new(|| FixedOffset::east_opt(9 * 3600).expect("failed to get FixedOffset"));

/// 時間単位での一時アカウントの有効期限<br>
/// [VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR] 丁度の期間は有効期限に含む
pub const VALID_PERIOD_OF_TEMP_ACCOUNT_IN_HOUR: i64 = 24;

/// 分単位でのパスワード変更要求の有効期限<br>
/// [VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE] 丁度の期間は有効期限に含む
pub const VALID_PERIOD_OF_PASSWORD_CHANGE_REQ_IN_MINUTE: i64 = 10;

/// 1アカウント当たりに登録可能な職務経歴情報の最大数
pub const MAX_NUM_OF_CAREER_PER_USER_ACCOUNT: u64 = 8;

/// 相談時間の長さ (分単位)
pub const LENGTH_OF_MEETING_IN_MINUTE: u32 = 60;

/// お知らせを取ってくる基準 (日単位)
///
/// 現在時刻から[NEWS_RETRIEVAL_CRITERIA_IN_DAYS] 日前までのお知らせを取得する
pub const NEWS_RETRIEVAL_CRITERIA_IN_DAYS: i64 = 180;

/// 受け付けた相談を承認する際、相談開始日時までに空いていなければならない最小期間（単位：秒）
pub const MIN_DURATION_BEFORE_CONSULTATION_ACCEPTANCE_IN_SECONDS: u32 = 21600;

// TODO: リリース前に値を調整する
/// パスワード、パスコードをハッシュ化する際のストレッチング回数（ストレッチングが2^[BCRYPT_COST]回実行される) <br>
/// <br>
/// NOTE:<br>
/// 適切な値は実行環境により異なる。実行環境が変わる際は計測し、適切な値を設定する。<br>
/// 下記リンクによると一回の処理に250ms以上計算にかかる値を選択するのが適切と紹介されている。<br>
/// 参考: https://security.stackexchange.com/questions/17207/recommended-of-rounds-for-bcrypt <br>
const BCRYPT_COST: u32 = 7;

/// アプリケーションサーバが保持可能な状態
///
/// アプリケーションサーバの起動時にインスタンスを作成、セットする。
/// そうすることで、ハンドラ関数やハンドラにたどり着く前に処理される関数にパラメータとして渡され、使用可能となる
#[derive(Clone, FromRef)]
pub struct AppState {
    pub store: RedisSessionStore,
    pub index_client: OpenSearch,
    pub pool: DatabaseConnection,
    pub key_for_signed_cookie: Key,
    pub smtp_client: SmtpClient,
    pub storage_client: StorageClient,
}

impl From<AppState> for Key {
    fn from(state: AppState) -> Self {
        state.key_for_signed_cookie
    }
}

/// HTTPリクエストにおいてログに残す要素を集めた構造体
pub struct RequestLogElements {
    method: String,
    uri: String,
    version: String,
    x_forwarded_for: String,
    x_real_ip: String,
    forwarded: String,
    user_agent: String,
}

impl RequestLogElements {
    pub fn new(request: &Request<Body>) -> Self {
        let method = request.method();
        let uri = request.uri();
        let version = request.version();
        let headers = request.headers();
        let x_forwarded_for = headers
            .get("x-forwarded-for")
            .map(|hv| match hv.to_str() {
                Ok(s) => s.to_string(),
                Err(e) => format!("{}", e),
            })
            .unwrap_or_else(|| "None".to_string());
        let x_real_ip = headers
            .get("x-real-ip")
            .map(|hv| match hv.to_str() {
                Ok(s) => s.to_string(),
                Err(e) => format!("{}", e),
            })
            .unwrap_or_else(|| "None".to_string());
        let forwarded = headers
            .get("forwarded")
            .map(|hv| match hv.to_str() {
                Ok(s) => s.to_string(),
                Err(e) => format!("{}", e),
            })
            .unwrap_or_else(|| "None".to_string());
        let user_agent = request
            .headers()
            .get("user-agent")
            .map(|hv| match hv.to_str() {
                Ok(s) => s.to_string(),
                Err(e) => format!("{}", e),
            })
            .unwrap_or_else(|| "None".to_string());
        RequestLogElements {
            method: format!("{}", method),
            uri: format!("{}", uri),
            version: format!("{:?}", version),
            x_forwarded_for,
            x_real_ip,
            forwarded,
            user_agent,
        }
    }

    pub fn method(&self) -> &str {
        self.method.as_str()
    }

    pub fn uri(&self) -> &str {
        self.uri.as_str()
    }

    pub fn version(&self) -> &str {
        self.version.as_str()
    }

    pub fn x_forwarded_for(&self) -> &str {
        self.x_forwarded_for.as_str()
    }

    pub fn x_real_ip(&self) -> &str {
        self.x_real_ip.as_str()
    }

    pub fn forwarded(&self) -> &str {
        self.forwarded.as_str()
    }

    pub fn user_agent(&self) -> &str {
        self.user_agent.as_str()
    }
}
