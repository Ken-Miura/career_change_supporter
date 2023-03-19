use axum::{extract::State, Json};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use totp_rs::{Algorithm, Secret, TOTP};

use crate::err::unexpected_err_resp;

// "TestSecretSuperSecret"
// セットアップキーのキーは、下記のBase32エンコード済のものを使う
// アカウント名は任意。キーの種類は時間ベースとなる。
const SECRET: &str = "KRSXG5CTMVRXEZLUKN2XAZLSKNSWG4TFOQ";

pub(crate) async fn mfa(State(pool): State<DatabaseConnection>) -> RespResult<MfaResult> {
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded(SECRET.to_string()).to_bytes().unwrap(),
        Some("Github".to_string()),
        "Ken-Miura@github.com".to_string(),
    )
    .unwrap();
    let code = totp.get_qr().expect("failed to get Ok");
    println!("{}", code);
    Ok((StatusCode::OK, Json(MfaResult { qr: code })))
}

#[derive(Serialize, Debug)]
pub(crate) struct MfaResult {
    qr: String,
}

pub(crate) async fn check(
    State(pool): State<DatabaseConnection>,
    Json(totp_code): Json<TotpCode>,
) -> RespResult<CheckResult> {
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded(SECRET.to_string()).to_bytes().unwrap(),
        Some("Github".to_string()),
        "Ken-Miura@github.com".to_string(),
    )
    .unwrap();

    let is_valid = totp
        .check_current(totp_code.code.as_str())
        .expect("failed to get Ok");

    print!("!!! is_valid: {} !!!", is_valid);
    if is_valid {
        Ok((StatusCode::OK, Json(CheckResult {})))
    } else {
        Err(unexpected_err_resp())
    }
}

#[derive(Deserialize)]
pub(crate) struct TotpCode {
    pub(crate) code: String,
}

#[derive(Serialize, Debug)]
pub(crate) struct CheckResult {}
