// Copyright 2023 Ken Miura
//! 管理者向けサービス (admin_service) 、管理者アカウント管理ツール (admin_account) と
//! その他定期実行ツールが共通で使うものをまとめたモジュール

use std::env::var;

use once_cell::sync::Lazy;

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
