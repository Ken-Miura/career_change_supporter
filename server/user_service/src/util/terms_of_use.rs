// Copyright 2021 Ken Miura

use axum::http::StatusCode;
use axum::Json;
use common::{
    model::user::TermsOfUse,
    schema::ccs_schema::terms_of_use::dsl::terms_of_use as terms_of_use_table,
};
use common::{ApiError, ErrResp};
use core::panic;
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection,
};
use diesel::{QueryDsl, RunQueryDsl};
use once_cell::sync::Lazy;
use std::env::var;

use crate::err_code::NOT_TERMS_OF_USE_AGREED_YET;
use crate::util::unexpected_err_resp;

pub(crate) const KEY_TO_TERMS_OF_USE_VERSION: &str = "TERMS_OF_USE_VERSION";
pub(crate) static TERMS_OF_USE_VERSION: Lazy<i32> = Lazy::new(|| {
    let terms_of_use_version_str = var(KEY_TO_TERMS_OF_USE_VERSION).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_TERMS_OF_USE_VERSION
        )
    });
    let terms_of_use_version = terms_of_use_version_str.parse().unwrap_or_else(|_| {
        panic!(
            "\"{}\" must be number: {}",
            KEY_TO_TERMS_OF_USE_VERSION, terms_of_use_version_str
        );
    });
    if terms_of_use_version < 1 {
        panic!(
            "\"{}\" must be positive: {}",
            KEY_TO_TERMS_OF_USE_VERSION, terms_of_use_version
        )
    }
    terms_of_use_version
});

pub(crate) trait TermsOfUseCheckOperation {
    fn check_if_user_has_already_agreed(
        &self,
        id: i32,
        terms_of_use_version: i32,
    ) -> Result<(), ErrResp>;
}

pub(crate) struct TermsOfUseCheckOperationImpl {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl TermsOfUseCheckOperationImpl {
    pub(crate) fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { conn }
    }
}

impl TermsOfUseCheckOperation for TermsOfUseCheckOperationImpl {
    fn check_if_user_has_already_agreed(
        &self,
        id: i32,
        terms_of_use_version: i32,
    ) -> Result<(), ErrResp> {
        let results = terms_of_use_table
            .find((id, terms_of_use_version))
            .load::<TermsOfUse>(&self.conn)
            .map_err(|e| {
                tracing::error!(
                    "failed to check if user agreed with terms of use (id: {}, version: {}): {}",
                    id,
                    terms_of_use_version,
                    e
                );
                unexpected_err_resp()
            })?;
        let len = results.len();
        if len == 0 {
            tracing::info!(
                "id ({}) has not agreed terms of use version ({}) yet",
                id,
                terms_of_use_version
            );
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ApiError {
                    code: NOT_TERMS_OF_USE_AGREED_YET,
                }),
            ));
        }
        if len > 1 {
            // NOTE: primary keyで検索しているため、ここを通るケースはdieselの障害
            panic!(
                "number of terms of use (id: {}, version: {}): {}",
                id, terms_of_use_version, len
            )
        }
        Ok(())
    }
}
