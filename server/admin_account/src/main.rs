// Copyright 2021 Ken Miura

use chrono::{DateTime, FixedOffset};
use common::admin::{KEY_TO_ADMIN_TOTP_ISSUER, KEY_TO_DB_ADMIN_NAME, KEY_TO_DB_ADMIN_PASSWORD};
use common::db::{construct_db_url, KEY_TO_DB_HOST, KEY_TO_DB_NAME, KEY_TO_DB_PORT};
use common::mfa::{generate_base32_encoded_secret, generate_base64_encoded_qr_code};
use common::password::hash_password;
use common::util::check_env_vars;
use common::util::validator::{
    email_address_validator::validate_email_address, password_validator::validate_password,
};
use common::JAPANESE_TIME_ZONE;
use dotenv::dotenv;
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectOptions, Database, DatabaseConnection,
    DatabaseTransaction, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, Set,
    TransactionError, TransactionTrait,
};
use std::{env::args, env::var, error::Error, fmt, process::exit};

const SUCCESS: i32 = 0;
const ENV_VAR_CAPTURE_FAILURE: i32 = 1;
const CONNECTION_ERROR: i32 = 2;
const INVALID_ARG_LENGTH: i32 = 3;
const INVALID_SUB_COMMAND: i32 = 4;
const APPLICATION_ERR: i32 = 5;

fn main() {
    let _ = dotenv().ok();
    let result = check_env_vars(vec![
        KEY_TO_DB_HOST.to_string(),
        KEY_TO_DB_PORT.to_string(),
        KEY_TO_DB_NAME.to_string(),
        KEY_TO_DB_ADMIN_NAME.to_string(),
        KEY_TO_DB_ADMIN_PASSWORD.to_string(),
        KEY_TO_ADMIN_TOTP_ISSUER.to_string(),
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
    // 標準出力に出力される内容を見やすくしたい＋管理者向けのアカウント操作しかしないのでログで見る必要のある重要な箇所もない
    // 従ってログの初期化は行わない

    let database_url = construct_db_url(
        KEY_TO_DB_HOST,
        KEY_TO_DB_PORT,
        KEY_TO_DB_NAME,
        KEY_TO_DB_ADMIN_NAME,
        KEY_TO_DB_ADMIN_PASSWORD,
    );
    let conn = connect(&database_url).await.unwrap_or_else(|e| {
        println!(
            "failed to establish connection (database_url: {}): {}",
            database_url, e
        );
        exit(CONNECTION_ERROR);
    });

    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        println!(
            "usage: {} [ create | list | update | delete | mfa ] [SUB_COMMAND_ARGS...]",
            args[0]
        );
        exit(INVALID_ARG_LENGTH);
    }
    let cmd = &args[1];
    if cmd == "create" {
        create(&conn, args).await;
    } else if cmd == "list" {
        list(&conn, args).await;
    } else if cmd == "update" {
        update(&conn, args).await;
    } else if cmd == "delete" {
        delete(&conn, args).await;
    } else if cmd == "mfa" {
        mfa(&conn, args).await;
    } else {
        println!("invalid subcommand: {}", cmd);
        println!("valid subcommand [ create | list | update | delete | mfa ]");
        exit(INVALID_SUB_COMMAND);
    }
}

async fn connect(database_url: &str) -> Result<DatabaseConnection, Box<dyn Error + Send + Sync>> {
    let mut opt = ConnectOptions::new(database_url.to_string());
    opt.max_connections(1).min_connections(1).sqlx_logging(true);
    let conn = Database::connect(opt).await.map_err(Box::new)?;
    Ok(conn)
}

async fn create(conn: &DatabaseConnection, args: Vec<String>) {
    if args.len() != 4 {
        println!(
            "usage: {} create \"admin_email_address\" \"password\"",
            args[0]
        );
        println!("ex: {} create admin@test.com 1234abcdABCD", args[0]);
        exit(INVALID_ARG_LENGTH);
    }
    let exists = check_if_admin_account_already_exists(conn, &args[2])
        .await
        .unwrap_or_else(|e| {
            println!("application error: {}", e);
            exit(APPLICATION_ERR);
        });
    if exists {
        println!("application error: \"{}\" already exists", &args[2]);
        exit(APPLICATION_ERR);
    }
    let current_date_time = chrono::Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
    match create_account(conn, &args[2], &args[3], &current_date_time).await {
        Ok(_) => exit(SUCCESS),
        Err(e) => {
            println!("application error: {}", e);
            exit(APPLICATION_ERR);
        }
    };
}

async fn check_if_admin_account_already_exists(
    conn: &DatabaseConnection,
    email_addr: &str,
) -> Result<bool, Box<dyn Error + Send + Sync>> {
    let count = entity::admin_account::Entity::find()
        .filter(entity::admin_account::Column::EmailAddress.eq(email_addr))
        .count(conn)
        .await
        .map_err(Box::new)?;
    Ok(count != 0)
}

async fn create_account(
    conn: &DatabaseConnection,
    email_addr: &str,
    password: &str,
    current_date_time: &DateTime<FixedOffset>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    validate_email_address(email_addr)?;
    validate_password(password)?;
    let hashed_pwd = hash_password(password)?;
    let admin_account = entity::admin_account::ActiveModel {
        email_address: Set(email_addr.to_string()),
        hashed_password: Set(hashed_pwd),
        created_at: Set(*current_date_time),
        ..Default::default()
    };
    admin_account.insert(conn).await.map_err(Box::new)?;
    Ok(())
}

async fn list(conn: &DatabaseConnection, args: Vec<String>) {
    if args.len() != 2 {
        println!("usage: {} list", args[0]);
        println!("ex: {} list", args[0]);
        exit(INVALID_ARG_LENGTH);
    }
    match list_accounts(conn).await {
        Ok(admin_accounts) => {
            println!("email_address, mfa_enabled, last_login, created_at");
            admin_accounts.iter().for_each(|admin_account| {
                let last_login_time = match admin_account.last_login_time {
                    Some(t) => t.with_timezone(&*JAPANESE_TIME_ZONE).to_rfc3339(),
                    None => "None".to_string(),
                };
                println!(
                    "{}, {}, \"{}\", \"{}\"",
                    admin_account.email_address,
                    admin_account.mfa_enabled_at.is_some(),
                    last_login_time,
                    admin_account
                        .created_at
                        .with_timezone(&*JAPANESE_TIME_ZONE)
                        .to_rfc3339()
                );
            });
            exit(SUCCESS)
        }
        Err(e) => {
            println!("application error: {}", e);
            exit(APPLICATION_ERR);
        }
    };
}

async fn list_accounts(
    conn: &DatabaseConnection,
) -> Result<Vec<entity::admin_account::Model>, Box<dyn Error + Send + Sync>> {
    let admin_accounts = entity::admin_account::Entity::find()
        .all(conn)
        .await
        .map_err(Box::new)?;
    Ok(admin_accounts)
}

async fn update(conn: &DatabaseConnection, args: Vec<String>) {
    if args.len() != 4 {
        println!(
            "usage: {} update \"admin_email_address\" \"password\"",
            args[0]
        );
        println!("ex: {} update admin@test.com 1234abcdABCD", args[0]);
        exit(INVALID_ARG_LENGTH);
    }
    match update_account(conn, &args[2], &args[3]).await {
        Ok(_) => exit(SUCCESS),
        Err(e) => {
            println!("application error: {}", e);
            exit(APPLICATION_ERR);
        }
    };
}

async fn update_account(
    conn: &DatabaseConnection,
    email_addr: &str,
    password: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    validate_email_address(email_addr)?;
    validate_password(password)?;
    let hashed_pwd = hash_password(password)?;
    let model = get_admin_account_by_email_address(conn, email_addr).await?;
    let mut active_model: entity::admin_account::ActiveModel = model.into();
    active_model.hashed_password = Set(hashed_pwd);
    let _ = active_model.update(conn).await.map_err(Box::new)?;
    Ok(())
}

async fn get_admin_account_by_email_address(
    conn: &DatabaseConnection,
    email_addr: &str,
) -> Result<entity::admin_account::Model, Box<dyn Error + Send + Sync>> {
    let results = entity::admin_account::Entity::find()
        .filter(entity::admin_account::Column::EmailAddress.eq(email_addr))
        .all(conn)
        .await
        .map_err(Box::new)?;
    if results.len() != 1 {
        return Err(Box::new(NoAccountFoundError(email_addr.to_string())));
    }
    Ok(results[0].clone())
}

async fn delete(conn: &DatabaseConnection, args: Vec<String>) {
    if args.len() != 3 {
        println!("usage: {} delete \"admin_email_address\"", args[0]);
        println!("ex: {} delete admin@test.com", args[0]);
        exit(INVALID_ARG_LENGTH);
    }
    match delete_account(conn, &args[2]).await {
        Ok(_) => exit(SUCCESS),
        Err(e) => {
            println!("application error: {}", e);
            exit(APPLICATION_ERR);
        }
    };
}

async fn delete_account(
    conn: &DatabaseConnection,
    email_addr: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    validate_email_address(email_addr)?;
    let model = get_admin_account_by_email_address(conn, email_addr).await?;
    let admin_account_id = model.admin_account_id;
    conn.transaction::<_, (), TxErr>(|txn| {
        Box::pin(async move {
            let _ = entity::admin_mfa_info::Entity::delete_by_id(admin_account_id)
                .exec(txn)
                .await
                .map_err(|e| TxErr(Box::new(e)))?;

            let _ = entity::admin_account::Entity::delete_by_id(admin_account_id)
                .exec(txn)
                .await
                .map_err(|e| TxErr(Box::new(e)))?;
            Ok(())
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(db_err) => Box::new(db_err),
        TransactionError::Transaction(tx_err) => tx_err.0,
    })?;
    Ok(())
}

async fn mfa(conn: &DatabaseConnection, args: Vec<String>) {
    if args.len() != 4 {
        println!(
            "usage: {} mfa [ enable | disable ] \"admin_email_address\"",
            args[0]
        );
        println!("ex: {} mfa enable admin@test.com", args[0]);
        exit(INVALID_ARG_LENGTH);
    }
    if args[2] == "enable" {
        let base32_encoded_secret = generate_base32_encoded_secret().unwrap_or_else(|e| {
            let err = UnexpectedError(format!("{:?}", e));
            println!(
                "application error: failed to generate base 32 encoded secret: {}",
                err
            );
            exit(APPLICATION_ERR);
        });
        let issuer = var(KEY_TO_ADMIN_TOTP_ISSUER).unwrap_or_else(|e| {
            println!(
                "failed to ge environment variable ({}): {}",
                KEY_TO_ADMIN_TOTP_ISSUER, e
            );
            exit(ENV_VAR_CAPTURE_FAILURE);
        });
        let current_date_time = chrono::Utc::now().with_timezone(&(*JAPANESE_TIME_ZONE));
        let result = enable_mfa(
            conn,
            &args[3],
            &base32_encoded_secret,
            &issuer,
            &current_date_time,
        )
        .await;
        match result {
            Ok(base_64_encoded_qr_code) => {
                print_base_64_encoded_qr_code_by_html(base_64_encoded_qr_code);
                exit(SUCCESS)
            }
            Err(e) => {
                println!("application error: {}", e);
                exit(APPLICATION_ERR);
            }
        };
    } else if args[2] == "disable" {
        match disable_mfa(conn, &args[3]).await {
            Ok(_) => exit(SUCCESS),
            Err(e) => {
                println!("application error: {}", e);
                exit(APPLICATION_ERR);
            }
        };
    } else {
        println!(
            "usage: {} mfa [ enable | disable ] \"admin_email_address\"",
            args[0]
        );
        println!("ex: {} mfa enable admin@test.com", args[0]);
        exit(INVALID_SUB_COMMAND);
    }
}

async fn enable_mfa(
    conn: &DatabaseConnection,
    email_addr: &str,
    base32_encoded_secret: &str,
    issuer: &str,
    current_date_time: &DateTime<FixedOffset>,
) -> Result<String, Box<dyn Error + Send + Sync>> {
    validate_email_address(email_addr)?;
    let model = get_admin_account_by_email_address(conn, email_addr).await?;
    let admin_account_id = model.admin_account_id;
    let admin_mfa_info = entity::admin_mfa_info::Entity::find_by_id(admin_account_id)
        .one(conn)
        .await
        .map_err(Box::new)?;
    if admin_mfa_info.is_some() {
        return Err(Box::new(MfaAlreadyEnabled));
    }
    let email_addr = email_addr.to_string();
    let base32_encoded_secret = base32_encoded_secret.to_string();
    let issuer = issuer.to_string();
    let current_date_time = *current_date_time;
    let base_64_encoded_qr_code = conn
        .transaction::<_, String, TxErr>(|txn| {
            Box::pin(async move {
                let admin_account_model = select_admin_account_for_update(txn, admin_account_id)
                    .await
                    .map_err(TxErr)?;
                let admin_account_model = admin_account_model
                    .ok_or_else(|| TxErr(Box::new(NoAccountFoundError(email_addr))))?;

                let admin_mfa_info_active_model = entity::admin_mfa_info::ActiveModel {
                    admin_account_id: Set(admin_account_id),
                    base32_encoded_secret: Set(base32_encoded_secret.to_string()),
                };
                let _ = admin_mfa_info_active_model
                    .insert(txn)
                    .await
                    .map_err(|e| TxErr(Box::new(e)))?;

                let mut admin_account_active_model: entity::admin_account::ActiveModel =
                    admin_account_model.into();
                admin_account_active_model.mfa_enabled_at = Set(Some(current_date_time));
                let _ = admin_account_active_model
                    .update(txn)
                    .await
                    .map_err(|e| TxErr(Box::new(e)))?;

                let base_64_encoded_qr_code = generate_base64_encoded_qr_code(
                    model.admin_account_id,
                    &base32_encoded_secret,
                    &issuer,
                )
                .map_err(|e| {
                    let err = UnexpectedError(format!("{:?}", e));
                    TxErr(Box::new(err))
                })?;

                Ok(base_64_encoded_qr_code)
            })
        })
        .await
        .map_err(|e| match e {
            TransactionError::Connection(db_err) => Box::new(db_err),
            TransactionError::Transaction(tx_err) => tx_err.0,
        })?;
    Ok(base_64_encoded_qr_code)
}

async fn select_admin_account_for_update(
    conn: &DatabaseTransaction,
    admin_account_id: i64,
) -> Result<Option<entity::admin_account::Model>, Box<dyn Error + Send + Sync>> {
    let model = entity::admin_account::Entity::find_by_id(admin_account_id)
        .lock_exclusive()
        .one(conn)
        .await
        .map_err(Box::new)?;
    Ok(model)
}

fn print_base_64_encoded_qr_code_by_html(base_64_encoded_qr_code: String) {
    println!(
        r#"<!-- Create file, then copy and paste following code on it to capture secret by auth app like Google Authenticator -->
<html>
  <head>
    <meta charset="utf-8">
    <title>base_64_encoded_qr_code</title>
  </head>
  <body>
    <img src="data:image/png;base64,{}" />
  </body>
</html>"#,
        base_64_encoded_qr_code
    );
}

async fn disable_mfa(
    conn: &DatabaseConnection,
    email_addr: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    validate_email_address(email_addr)?;
    let model = get_admin_account_by_email_address(conn, email_addr).await?;
    let admin_account_id = model.admin_account_id;
    let email_addr = email_addr.to_string();
    conn.transaction::<_, (), TxErr>(|txn| {
        Box::pin(async move {
            let admin_account_model = select_admin_account_for_update(txn, admin_account_id)
                .await
                .map_err(TxErr)?;
            let admin_account_model = admin_account_model
                .ok_or_else(|| TxErr(Box::new(NoAccountFoundError(email_addr))))?;

            let _ = entity::admin_mfa_info::Entity::delete_by_id(admin_account_id)
                .exec(txn)
                .await
                .map_err(|e| TxErr(Box::new(e)));

            let mut admin_account_active_model: entity::admin_account::ActiveModel =
                admin_account_model.into();
            admin_account_active_model.mfa_enabled_at = Set(None);
            let _ = admin_account_active_model
                .update(txn)
                .await
                .map_err(|e| TxErr(Box::new(e)))?;

            Ok(())
        })
    })
    .await
    .map_err(|e| match e {
        TransactionError::Connection(db_err) => Box::new(db_err),
        TransactionError::Transaction(tx_err) => tx_err.0,
    })?;
    Ok(())
}

#[derive(Debug, Clone)]
struct NoAccountFoundError(String);

impl fmt::Display for NoAccountFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "no email address ({}) found", self.0)
    }
}

impl Error for NoAccountFoundError {}

#[derive(Debug)]
struct TxErr(Box<dyn Error + Send + Sync>);

impl fmt::Display for TxErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "error in transaction: {}", self.0)
    }
}

impl Error for TxErr {}

#[derive(Debug, Clone)]
struct MfaAlreadyEnabled;

impl fmt::Display for MfaAlreadyEnabled {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "mfa already enabled")
    }
}

impl Error for MfaAlreadyEnabled {}

#[derive(Debug, Clone)]
struct UnexpectedError(String);

impl fmt::Display for UnexpectedError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "unxepected error: {}", self.0)
    }
}

impl Error for UnexpectedError {}
