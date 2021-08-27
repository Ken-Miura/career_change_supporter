// Copyright 2021 Ken Miura

use axum::Json;
use common::credential::Credential;
use common::DatabaseConnection;

/// 一時アカウントを作成する。<br>
/// <br>
///
/// # Errors
///
async fn _post_temp_accounts(
    Json(_req_body): Json<Credential>,
    DatabaseConnection(_conn): DatabaseConnection,
) {
}
