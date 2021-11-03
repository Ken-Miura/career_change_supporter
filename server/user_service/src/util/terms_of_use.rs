// Copyright 2021 Ken Miura

use once_cell::sync::Lazy;
use std::env::var;

pub(crate) const KEY_TO_TERMS_OF_USE_VERSION: &str = "TERMS_OF_USE_VERSION";
pub(crate) static TERMS_OF_USE_VERSION: Lazy<u32> = Lazy::new(|| {
    let terms_of_use_str = var(KEY_TO_TERMS_OF_USE_VERSION).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_TERMS_OF_USE_VERSION
        )
    });
    terms_of_use_str.parse().unwrap_or_else(|_| {
        panic!(
            "\"{}\" must be number: {}",
            KEY_TO_TERMS_OF_USE_VERSION, terms_of_use_str
        );
    })
});
