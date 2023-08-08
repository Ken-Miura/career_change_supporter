// Copyright 2023 Ken Miura

use std::env::var;

use once_cell::sync::Lazy;

/// RUST_LOGに指定するログレベルを示す環境変数にアクセスするためのキー
pub const KEY_TO_LOG_LEVEL: &str = "LOG_LEVEL";
/// RUST_LOGに指定するログレベルを示す環境変数
pub static LOG_LEVEL: Lazy<String> =
    Lazy::new(|| var(KEY_TO_LOG_LEVEL).unwrap_or_else(|_| "info".to_string()));
