// Copyright 2021 Ken Miura

use axum::async_trait;
use common::ErrResp;
use core::panic;
use entity::{
    prelude::TermsOfUse,
    sea_orm::{DatabaseConnection, EntityTrait},
};
use once_cell::sync::Lazy;
use std::env::var;
use tracing::error;

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

#[async_trait]
pub(crate) trait TermsOfUseLoadOperation {
    async fn find(
        &self,
        account_id: i64,
        terms_of_use_version: i32,
    ) -> Result<Option<TermsOfUseData>, ErrResp>;
}

pub(crate) struct TermsOfUseLoadOperationImpl<'a> {
    pool: &'a DatabaseConnection,
}

impl<'a> TermsOfUseLoadOperationImpl<'a> {
    pub(crate) fn new(pool: &'a DatabaseConnection) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl<'a> TermsOfUseLoadOperation for TermsOfUseLoadOperationImpl<'a> {
    async fn find(
        &self,
        account_id: i64,
        terms_of_use_version: i32,
    ) -> Result<Option<TermsOfUseData>, ErrResp> {
        let model = TermsOfUse::find_by_id((account_id, terms_of_use_version))
            .one(self.pool)
            .await
            .map_err(|e| {
                error!(
                    "failed to find terms_of_use (user_account_id: {}, ver: {}): {}",
                    account_id, terms_of_use_version, e
                );
                unexpected_err_resp()
            })?;
        Ok(model.map(|_m| TermsOfUseData {}))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TermsOfUseData {}
