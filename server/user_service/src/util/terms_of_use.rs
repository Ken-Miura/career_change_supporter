// Copyright 2021 Ken Miura

use common::ErrResp;
use common::{
    model::user::TermsOfUse,
    schema::ccs_schema::terms_of_use::dsl::terms_of_use as terms_of_use_table,
};
use core::panic;
use diesel::{
    r2d2::{ConnectionManager, PooledConnection},
    PgConnection,
};
use diesel::{QueryDsl, RunQueryDsl};
use once_cell::sync::Lazy;
use std::env::var;

use crate::err::unexpected_err_resp;

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

pub(crate) trait TermsOfUseLoadOperation {
    fn load(&self, id: i32, terms_of_use_version: i32) -> Result<Vec<TermsOfUse>, ErrResp>;
}

pub(crate) struct TermsOfUseLoadOperationImpl {
    conn: PooledConnection<ConnectionManager<PgConnection>>,
}

impl TermsOfUseLoadOperationImpl {
    pub(crate) fn new(conn: PooledConnection<ConnectionManager<PgConnection>>) -> Self {
        Self { conn }
    }
}

impl TermsOfUseLoadOperation for TermsOfUseLoadOperationImpl {
    fn load(&self, id: i32, terms_of_use_version: i32) -> Result<Vec<TermsOfUse>, ErrResp> {
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
        Ok(results)
    }
}
