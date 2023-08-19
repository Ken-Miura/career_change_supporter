// Copyright 2023 Ken Miura

use chrono::{DateTime, FixedOffset};
use dotenv::dotenv;
use entity::sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::{env::var, error::Error, process::exit};

use common::{
    db::{create_db_url, KEY_TO_DB_HOST, KEY_TO_DB_NAME, KEY_TO_DB_PORT},
    smtp::{
        SendMail, SmtpClient, AWS_SES_ACCESS_KEY_ID, AWS_SES_ENDPOINT_URI, AWS_SES_REGION,
        AWS_SES_SECRET_ACCESS_KEY, KEY_TO_ADMIN_EMAIL_ADDRESS, KEY_TO_AWS_SES_ACCESS_KEY_ID,
        KEY_TO_AWS_SES_ENDPOINT_URI, KEY_TO_AWS_SES_REGION, KEY_TO_AWS_SES_SECRET_ACCESS_KEY,
        KEY_TO_SYSTEM_EMAIL_ADDRESS,
    },
    util::check_env_vars,
    JAPANESE_TIME_ZONE,
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
        KEY_TO_ADMIN_EMAIL_ADDRESS.to_string(),
        KEY_TO_SYSTEM_EMAIL_ADDRESS.to_string(),
        KEY_TO_AWS_SES_REGION.to_string(),
        KEY_TO_AWS_SES_ACCESS_KEY_ID.to_string(),
        KEY_TO_AWS_SES_SECRET_ACCESS_KEY.to_string(),
        KEY_TO_AWS_SES_ENDPOINT_URI.to_string(),
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

async fn main_internal() {
    let current_date_time = chrono::Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));

    let num_of_max_target_records = var(KEY_TO_NUM_OF_MAX_TARGET_RECORDS)
        .unwrap_or_else(|_| {
            panic!(
                "Not environment variable found: environment variable \"{}\" must be set",
                KEY_TO_NUM_OF_MAX_TARGET_RECORDS
            )
        })
        .parse()
        .expect("failed to get Ok");

    let database_url = construct_db_url(
        KEY_TO_DB_HOST,
        KEY_TO_DB_PORT,
        KEY_TO_DB_NAME,
        KEY_TO_DB_ADMIN_NAME,
        KEY_TO_DB_ADMIN_PASSWORD,
    );
    let mut opt = ConnectOptions::new(database_url.clone());
    opt.max_connections(1).min_connections(1).sqlx_logging(true);
    let pool = Database::connect(opt).await.unwrap_or_else(|e| {
        println!("failed to connect database: {}", e);
        exit(CONNECTION_ERROR)
    });
    let op = DeleteExpiredTempAccountsOperationImpl { pool };

    let smtp_client = SmtpClient::new(
        AWS_SES_REGION.as_str(),
        AWS_SES_ACCESS_KEY_ID.as_str(),
        AWS_SES_SECRET_ACCESS_KEY.as_str(),
        AWS_SES_ENDPOINT_URI.as_str(),
    )
    .await;

    let result = delete_expired_temp_accounts(
        current_date_time,
        num_of_max_target_records,
        &op,
        &smtp_client,
    )
    .await;

    if result.is_err() {
        println!(
            "failed to delete expired temp accounts: {}",
            result.expect_err("failed to get Err")
        );
        exit(APPLICATION_ERR)
    }
    exit(SUCCESS)
}

fn construct_db_url(
    key_to_db_host: &str,
    key_to_db_port: &str,
    key_to_db_name: &str,
    key_to_db_admin_name: &str,
    key_to_db_admin_password: &str,
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
    let db_admin_name = var(key_to_db_admin_name).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            key_to_db_admin_name
        )
    });
    let db_admin_password = var(key_to_db_admin_password).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            key_to_db_admin_password
        )
    });
    create_db_url(
        &db_host,
        &db_port,
        &db_name,
        &db_admin_name,
        &db_admin_password,
    )
}

async fn delete_expired_temp_accounts(
    current_date_time: DateTime<FixedOffset>,
    num_of_max_target_records: u64,
    op: &impl DeleteExpiredTempAccountsOperation,
    send_mail: &impl SendMail,
) -> Result<(), Box<dyn Error>> {
    Ok(())
}

trait DeleteExpiredTempAccountsOperation {}

struct DeleteExpiredTempAccountsOperationImpl {
    pool: DatabaseConnection,
}

impl DeleteExpiredTempAccountsOperation for DeleteExpiredTempAccountsOperationImpl {}
