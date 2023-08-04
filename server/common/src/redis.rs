// Copyright 2021 Ken Miura

pub const KEY_TO_REDIS_HOST: &str = "REDIS_HOST";
pub const KEY_TO_REDIS_PORT: &str = "REDIS_PORT";

pub fn create_redis_url(host: &str, port: &str) -> String {
    let redis_url = format!("redis://{}:{}", host, port);
    redis_url
}
