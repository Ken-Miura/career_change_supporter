// Copyright 2023 Ken Miura
//! 管理者向けサービス (admin_service) 、管理者アカウント管理ツール (admin_account) と
//! その他定期実行ツールが共通で使うものをまとめたモジュール

use std::{env::var, error::Error, fmt::Display};

use once_cell::sync::Lazy;
use tokio::time::sleep;

/// 管理者向けサービス (admin_service) 、管理者アカウント管理ツール (admin_account) と
/// その他定期実行ツールがDBアクセスする際に利用するユーザー名を示す環境変数名
pub const KEY_TO_DB_ADMIN_NAME: &str = "DB_ADMIN_NAME";

/// 管理者向けサービス (admin_service) 、管理者アカウント管理ツール (admin_account) と
/// その他定期実行ツールがDBアクセスする際に利用するパスワードを示す環境変数名
pub const KEY_TO_DB_ADMIN_PASSWORD: &str = "DB_ADMIN_PASSWORD";

/// 管理者サービス向けの二段階認証で利用するTOTPの発行者を表す環境変数名
pub const KEY_TO_ADMIN_TOTP_ISSUER: &str = "ADMIN_TOTP_ISSUER";

/// 定期実行ツールがDBアクセスする際、処理レコードの最大数を表す環境変数名
pub const KEY_TO_NUM_OF_MAX_TARGET_RECORDS: &str = "NUM_OF_MAX_TARGET_RECORDS";
/// 定期実行ツールがDBアクセスする際、処理レコードの最大数を表す値
///
/// 環境変数を指定しない場合は0となり、0は無制限を意味する。
/// 処理レコード数が多すぎて、定期実行ツールの実行が終了しないケースが発生する場合がありえる。
/// そういった場合に処理レコード数を制限するために利用する。
pub static NUM_OF_MAX_TARGET_RECORDS: Lazy<u64> = Lazy::new(|| {
    let num_of_max_target_records =
        var(KEY_TO_NUM_OF_MAX_TARGET_RECORDS).unwrap_or_else(|_| "0".to_string());
    num_of_max_target_records
        .parse()
        .expect("failed to parse NUM_OF_MAX_TARGET_RECORDS")
});

/// 外部サービスに繰り返しアクセスする際、レート制限に引っかからないように各繰り返しでどれくらい待つかを表す環境変数
pub const KEY_TO_DURATION_FOR_WAITING_DEPENDENT_SERVICE_RATE_LIMIT_IN_MILLI_SECONDS: &str =
    "DURATION_WAITING_FOR_DEPENDENT_SERVICE_RATE_LIMIT_IN_MILLI_SECONDS";
/// 外部サービスに繰り返しアクセスする際、レート制限に引っかからないように各繰り返しでどれくらい待つかを表す値
///
/// 定期実行ツールのようなバッチ処理を行うものは、外部サービスにアクセスする際に短時間で大量のアクセスをする可能性がある。そのため、
/// バッチ処理がレート制限に引っかからないで完了できるよう必要に応じてこの環境変数を調整する。環境変数を指定しない場合は0（待たない）となる。
pub static DURATION_WAITING_FOR_DEPENDENT_SERVICE_RATE_LIMIT_IN_MILLI_SECONDS: Lazy<u64> =
    Lazy::new(|| {
        let duration_in_milli_seconds =
            var(KEY_TO_DURATION_FOR_WAITING_DEPENDENT_SERVICE_RATE_LIMIT_IN_MILLI_SECONDS)
                .unwrap_or_else(|_| "0".to_string());
        duration_in_milli_seconds.parse().expect(
            "failed to parse DURATION_WAITING_FOR_DEPENDENT_SERVICE_RATE_LIMIT_IN_MILLI_SECONDS",
        )
    });

/// [DURATION_WAITING_FOR_DEPENDENT_SERVICE_RATE_LIMIT_IN_MILLI_SECONDS] を利用する際に一緒に使うユーティリティ関数
pub async fn wait_for(duration_in_milli_seconds: u64) {
    sleep(std::time::Duration::from_millis(duration_in_milli_seconds)).await;
}

/// 定期実行ツールがトランザクション内の処理でエラーを起こした際に返す型
#[derive(Debug)]
pub struct TransactionExecutionError {
    pub message: String,
}

impl Display for TransactionExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for TransactionExecutionError {}
