// Copyright 2023 Ken Miura

use common::payment_platform::{
    construct_access_info, AccessInfo, KEY_TO_PAYMENT_PLATFORM_API_PASSWORD,
    KEY_TO_PAYMENT_PLATFORM_API_URL, KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
};
use once_cell::sync::Lazy;

pub(super) const PLATFORM_FEE_RATE_IN_PERCENTAGE: &str = "30.00";

/// PAY.JPにアクセスするための情報を保持する変数
pub(super) static ACCESS_INFO: Lazy<AccessInfo> = Lazy::new(|| {
    construct_access_info(
        KEY_TO_PAYMENT_PLATFORM_API_URL,
        KEY_TO_PAYMENT_PLATFORM_API_USERNAME,
        KEY_TO_PAYMENT_PLATFORM_API_PASSWORD,
    )
    .expect("failed to get Ok")
});
