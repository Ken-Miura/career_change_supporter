// Copyright 2023 Ken Miura

use std::env::var;

pub const KEY_TO_DB_HOST: &str = "DB_HOST";
pub const KEY_TO_DB_PORT: &str = "DB_PORT";
pub const KEY_TO_DB_NAME: &str = "DB_NAME";

/// 環境変数のからDBのURLを構築する
///
/// # panics
/// パラメータに指定した環境変数がない場合
pub fn construct_db_url(
    key_to_db_host: &str,
    key_to_db_port: &str,
    key_to_db_name: &str,
    key_to_db_role_name: &str,
    key_to_db_role_password: &str,
) -> String {
    let db_host = var(key_to_db_host).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            key_to_db_host
        )
    });
    let db_port = var(key_to_db_port).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            key_to_db_port
        )
    });
    let db_name = var(key_to_db_name).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            key_to_db_name
        )
    });
    let db_role_name = var(key_to_db_role_name).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            key_to_db_role_name
        )
    });
    let db_role_password = var(key_to_db_role_password).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            key_to_db_role_password
        )
    });
    create_db_url(
        &db_host,
        &db_port,
        &db_name,
        &db_role_name,
        &db_role_password,
    )
}

fn create_db_url(
    host: &str,
    port: &str,
    db_name: &str,
    role_name: &str,
    role_password: &str,
) -> String {
    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        role_name, role_password, host, port, db_name
    );
    db_url
}
