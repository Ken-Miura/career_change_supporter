// Copyright 2023 Ken Miura

use axum::{http::StatusCode, Json};
use common::RespResult;
use serde::Serialize;

/// サーバが起動しているかの確認（ヘルスチェック）に利用する
///
/// https://devblog.thebase.in/entry/2019/03/06/110000
/// の Health Check Response Format for HTTP APIs を参考にし、簡易的実装を提供する。
pub(crate) async fn get_health() -> RespResult<HealthResult> {
    Ok((
        StatusCode::OK,
        Json(HealthResult {
            status: "pass".to_string(),
        }),
    ))
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct HealthResult {
    status: String,
}
