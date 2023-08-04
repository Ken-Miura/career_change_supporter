// Copyright 2023 Ken Miura

pub const KEY_TO_DB_HOST: &str = "DB_HOST";
pub const KEY_TO_DB_PORT: &str = "DB_PORT";
pub const KEY_TO_DB_NAME: &str = "DB_NAME";

pub fn create_db_url(
    host: &str,
    port: &str,
    db_name: &str,
    user_name: &str,
    password: &str,
) -> String {
    let db_url = format!(
        "postgres://{}:{}@{}:{}/{}",
        user_name, password, host, port, db_name
    );
    db_url
}
