// Copyright 2022 Ken Miura

use std::env::var;

use axum::{http::StatusCode, Json};
use once_cell::sync::Lazy;
use opensearch::{http::transport::Transport, IndexParts, OpenSearch, UpdateParts};
use serde_json::Value;
use tracing::error;

use crate::{err::Code, ApiError, ErrResp};

pub const KEY_TO_OPENSEARCH_ENDPOINT_URI: &str = "OPENSEARCH_ENDPOINT_URI";
pub static OPENSEARCH_ENDPOINT_URI: Lazy<String> = Lazy::new(|| {
    var(KEY_TO_OPENSEARCH_ENDPOINT_URI).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" (example value: \"http://opensearch:9200\") must be set",
            KEY_TO_OPENSEARCH_ENDPOINT_URI
        );
    })
});

pub const INDEX_NAME: &str = "users";

pub async fn index_document(
    endpoint_uri: &str,
    index_name: &str,
    document_id: &str,
    json_value: &Value,
) -> Result<(), ErrResp> {
    let client = build_client(endpoint_uri)?;
    let response = client
        .index(IndexParts::IndexId(index_name, document_id))
        .body(json_value.clone())
        .send()
        .await
        .map_err(|e| {
            error!(
                "failed to index document (index_name: {}, document_id: {}, json_value: {}): {}",
                index_name, document_id, json_value, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: Code::UnexpectedErr as u32,
                }),
            )
        })?;
    let status_code = response.status_code();
    if !status_code.is_success() {
        error!("failed to request index (response: {:?})", response);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        ));
    }
    Ok(())
}

pub async fn update_document(
    endpoint_uri: &str,
    index_name: &str,
    document_id: &str,
    json_value: &Value,
) -> Result<(), ErrResp> {
    let client = build_client(endpoint_uri)?;
    let response = client
        .update(UpdateParts::IndexId(index_name, document_id))
        .body(json_value.clone())
        .send()
        .await
        .map_err(|e| {
            error!(
                "failed to update document (index_name: {}, document_id: {}, json_value: {}): {}",
                index_name, document_id, json_value, e
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    code: Code::UnexpectedErr as u32,
                }),
            )
        })?;
    let status_code = response.status_code();
    if !status_code.is_success() {
        error!(
            "failed to request document update (response: {:?})",
            response
        );
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        ));
    }
    Ok(())
}

fn build_client(endpoint_uri: &str) -> Result<OpenSearch, ErrResp> {
    let transport = Transport::single_node(endpoint_uri).map_err(|e| {
        error!(
            "failed to struct transport (endpoint_uri: {}): {}",
            endpoint_uri, e
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    Ok(OpenSearch::new(transport))
}
