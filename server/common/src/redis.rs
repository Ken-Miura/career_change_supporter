// Copyright 2021 Ken Miura

use std::env::var;

pub const KEY_TO_REDIS_HOST: &str = "REDIS_HOST";
pub const KEY_TO_REDIS_PORT: &str = "REDIS_PORT";

/// 環境変数のからRedisのURLを構築する
///
/// # panics
/// パラメータに指定した環境変数がない場合
pub fn construct_redis_url(key_to_redis_host: &str, key_to_redis_port: &str) -> String {
    let redis_host = var(key_to_redis_host).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            key_to_redis_host
        )
    });
    let redis_port = var(key_to_redis_port).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            key_to_redis_port
        )
    });
    create_redis_url(&redis_host, &redis_port)
}

fn create_redis_url(host: &str, port: &str) -> String {
    let redis_url = format!("redis://{}:{}", host, port);
    redis_url
}
