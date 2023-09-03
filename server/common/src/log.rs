// Copyright 2023 Ken Miura

use std::env::var;

use once_cell::sync::Lazy;

/// RUST_LOGに指定するログレベルを示す環境変数にアクセスするためのキー
pub const KEY_TO_LOG_LEVEL: &str = "LOG_LEVEL";
/// RUST_LOGに指定するログレベルを示す環境変数
pub static LOG_LEVEL: Lazy<String> =
    Lazy::new(|| var(KEY_TO_LOG_LEVEL).unwrap_or_else(|_| "info".to_string()));

/// ログ設定の初期化を行う
///
/// ログのフォーマットや制御文字利用の抑止を行っている。ログ出力の前（典型的にはプロジェクト実行の初めの方）で一度だけ呼び出しておく。
pub fn init_log() {
    // ログの出力、保管先にAWS CloudWatch Logsを仮定している。
    // AWS CloudWatch Logsでは色を示す制御文字は正しく扱えないため文字化けとなる。
    // 従って、色を示す制御文字を抑制するためにANSIを明示的に不使用にしている。
    let format = tracing_subscriber::fmt::format().with_ansi(false);
    tracing_subscriber::fmt()
        .event_format(format)
        .with_ansi(false)
        .init();
}
