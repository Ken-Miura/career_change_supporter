// Copyright 2021 Ken Miura

use axum::{http::StatusCode, Json};
use common::{DatabaseConnection, RespResult, ValidCred};
use serde::Serialize;

/// 一時アカウントを作成する。<br>
/// <br>
///
/// # Errors
///
async fn _post_temp_accounts(
    ValidCred(_cred): ValidCred,
    DatabaseConnection(_conn): DatabaseConnection,
) -> RespResult<TempAccount> {
    let ret = (
        StatusCode::OK,
        Json(TempAccount {
            email_address: "test@test.com".to_string(),
        }),
    );
    Ok(ret)
}

#[derive(Serialize)]
struct TempAccount {
    email_address: String,
}
