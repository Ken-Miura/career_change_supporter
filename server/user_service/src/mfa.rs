use axum::{extract::State, Json};
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use hyper::StatusCode;
use serde::Serialize;
use totp_rs::{Algorithm, Secret, TOTP};

pub(crate) async fn mfa(State(pool): State<DatabaseConnection>) -> RespResult<MfaResult> {
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        Secret::Encoded("KRSXG5CTMVRXEZLUKN2XAZLSKNSWG4TFOQ".to_string())
            .to_bytes()
            .unwrap(),
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
