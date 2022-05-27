// Copyright 2022 Ken Miura

use std::env::var;

use once_cell::sync::Lazy;

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
