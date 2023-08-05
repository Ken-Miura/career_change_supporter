// Copyright 2022 Ken Miura

use std::error::Error;

use axum::{http::StatusCode, Json};
use opensearch::{
    auth::Credentials,
    http::transport::{SingleNodeConnectionPool, TransportBuilder},
    DeleteParts, IndexParts, OpenSearch, SearchParts, UpdateParts,
};
use serde_json::Value;
use tracing::error;

use crate::{err::Code, ApiError, ErrResp};

pub const KEY_TO_OPENSEARCH_ENDPOINT_URI: &str = "OPENSEARCH_ENDPOINT_URI";
pub const KEY_TO_OPENSEARCH_AUTH: &str = "OPENSEARCH_AUTH";
pub const KEY_TO_OPENSEARCH_USERNAME: &str = "OPENSEARCH_USERNAME";
pub const KEY_TO_OPENSEARCH_PASSWORD: &str = "OPENSEARCH_PASSWORD";

pub const INDEX_NAME: &str = "users";

pub async fn index_document(
    index_name: &str,
    document_id: &str,
    json_value: &Value,
    client: &OpenSearch,
) -> Result<(), ErrResp> {
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
    index_name: &str,
    document_id: &str,
    json_value: &Value,
    client: &OpenSearch,
) -> Result<(), ErrResp> {
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

#[derive(Debug, Clone)]
pub struct Sort {
    pub key: String,
    pub order: String,
}

pub async fn search_documents(
    index_name: &str,
    from: i64,
    size: i64,
    sort: Option<Sort>,
    query: &Value,
    client: &OpenSearch,
) -> Result<Value, ErrResp> {
    let result = if let Some(s) = sort.clone() {
        let pair = format!("{}:{}", s.key, s.order);
        client
            .search(SearchParts::Index(&[index_name]))
            .from(from)
            .size(size)
            .sort(&[pair.as_str()])
            .body(query.clone())
            .send()
            .await
    } else {
        client
            .search(SearchParts::Index(&[index_name]))
            .from(from)
            .size(size)
            .body(query.clone())
            .send()
            .await
    };
    let response = result.map_err(|e| {
        error!(
            "failed to search documents (index_name: {}, from: {}, size: {}, query: {}): {}",
            index_name, from, size, query, e
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    let status_code = response.status_code();
    let response_body = response.json::<Value>().await.map_err(|e| {
        error!(
            "failed to read body as json (index_name: {}, from: {}, size: {}, sort: {:?}, query: {}): {}",
            index_name, from, size, sort, query, e
        );
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        )
    })?;
    if !status_code.is_success() {
        error!("failed to search documents (index_name: {}, from: {}, size: {}, sort: {:?}, query: {}, response_body: {})", index_name, from, size, sort, query, response_body);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiError {
                code: Code::UnexpectedErr as u32,
            }),
        ));
    }
    Ok(response_body)
}

pub async fn delete_document(
    index_name: &str,
    document_id: &str,
    client: &OpenSearch,
) -> Result<(), ErrResp> {
    let response = client
        .delete(DeleteParts::IndexId(index_name, document_id))
        .send()
        .await
        .map_err(|e| {
            error!(
                "failed to delete document (index_name: {}, document_id: {}): {}",
                index_name, document_id, e
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
            "failed to request document delete (response: {:?})",
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

/// OpenSearchノードへアクセスするためのクライアントを作成する
///
/// # Panics
/// endpoint_uriがスキームにhttpまたはhttpsを指定していない場合<br>
/// endpoint_uriのスキームにhttpを指定して、かつauthがtrueの場合（認証を使う場合はhttps以外許可しないため）<br>
pub fn create_client(
    endpoint_uri: &str,
    auth: bool,
    username: &str,
    password: &str,
) -> Result<OpenSearch, Box<dyn Error>> {
    if !(endpoint_uri.starts_with("http://") || endpoint_uri.starts_with("https://")) {
        panic!("supported scheme is http or https: {}", endpoint_uri)
    }
    if endpoint_uri.starts_with("http://") && auth {
        panic!("Use https to enable auth: {}", endpoint_uri)
    }
    let url = opensearch::http::Url::parse(endpoint_uri)?;
    let conn_pool = SingleNodeConnectionPool::new(url);
    let builder = if auth {
        let credentials = Credentials::Basic(username.to_string(), password.to_string());
        TransportBuilder::new(conn_pool).auth(credentials)
    } else {
        TransportBuilder::new(conn_pool)
    };
    let transport = builder.build()?;
    Ok(OpenSearch::new(transport))
}
