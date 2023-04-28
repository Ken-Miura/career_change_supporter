// Copyright 2023 Ken Miura

use common::payment_platform::{
    AccessInfo, KEY_TO_PAYMENT_PLATFORM_API_PASSWORD, KEY_TO_PAYMENT_PLATFORM_API_URL,
    KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
};
use once_cell::sync::Lazy;
use std::env::var;

pub(super) const PLATFORM_FEE_RATE_IN_PERCENTAGE: &str = "30.00";

/// PAY.JPにアクセスするための情報を保持する変数
pub(super) static ACCESS_INFO: Lazy<AccessInfo> = Lazy::new(|| {
    let url_without_path = var(KEY_TO_PAYMENT_PLATFORM_API_URL).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_PAYMENT_PLATFORM_API_URL
        )
    });
    let username = var(KEY_TO_PAYMENT_PLATFORM_API_USERNAME).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_PAYMENT_PLATFORM_API_USERNAME
        )
    });
    let password = var(KEY_TO_PAYMENT_PLATFORM_API_PASSWORD).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_PAYMENT_PLATFORM_API_PASSWORD
        )
    });
    let access_info = AccessInfo::new(url_without_path, username, password);
    access_info.expect("failed to get Ok")
});
