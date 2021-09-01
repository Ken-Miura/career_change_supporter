// Copyright 2021 Ken Miura

use axum::extract::Query;
use serde::Deserialize;

/// アカウントを作成する
pub(crate) async fn get_accounts(temp_account: Query<TempAccount>) -> String {
    temp_account.temp_account_id.clone()
}

#[derive(Deserialize)]
pub(crate) struct TempAccount {
    #[serde(rename="temp-account-id")]
    temp_account_id: String
}
