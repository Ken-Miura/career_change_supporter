// Copyright 2023 Ken Miura

use dotenv::dotenv;
use std::process::exit;

use common::{
    db::{KEY_TO_DB_HOST, KEY_TO_DB_NAME, KEY_TO_DB_PORT},
    util::check_env_vars,
};

const KEY_TO_DB_ADMIN_NAME: &str = "DB_ADMIN_NAME";
const KEY_TO_DB_ADMIN_PASSWORD: &str = "DB_ADMIN_PASSWORD";
const KEY_TO_NUM_OF_MAX_TARGET_RECORDS: &str = "NUM_OF_MAX_TARGET_RECORDS";

const SUCCESS: i32 = 0;
const ENV_VAR_CAPTURE_FAILURE: i32 = 1;
const CONNECTION_ERROR: i32 = 2;
const APPLICATION_ERR: i32 = 3;

fn main() {
    let _ = dotenv().ok();
    let result = check_env_vars(vec![
        KEY_TO_DB_HOST.to_string(),
        KEY_TO_DB_PORT.to_string(),
        KEY_TO_DB_NAME.to_string(),
        KEY_TO_DB_ADMIN_NAME.to_string(),
        KEY_TO_DB_ADMIN_PASSWORD.to_string(),
        KEY_TO_NUM_OF_MAX_TARGET_RECORDS.to_string(),
    ]);
    if result.is_err() {
        println!("failed to resolve mandatory env vars (following env vars are needed)");
        println!("{:?}", result.unwrap_err());
        exit(ENV_VAR_CAPTURE_FAILURE);
    }

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("failed to build Runtime")
        .block_on(main_internal())
}

async fn main_internal() {}
