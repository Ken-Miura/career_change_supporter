// Copyright 2021 Ken Miura

use chrono::{DateTime, FixedOffset};
use common::password::hash_password;
use common::util::validator::{
    email_address_validator::validate_email_address, password_validator::validate_password,
};
use common::JAPANESE_TIME_ZONE;
use dotenv::dotenv;
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectOptions, Database, DatabaseConnection,
    DatabaseTransaction, EntityTrait, QueryFilter, QuerySelect, Set, TransactionError,
    TransactionTrait,
};
use std::{env::args, env::var, error::Error, fmt, process::exit};

const KEY_TO_DATABASE_URL: &str = "DB_URL_FOR_ADMIN_APP";

const SUCCESS: i32 = 0;
const ENV_VAR_CAPTURE_FAILURE: i32 = 1;
const CONNECTION_ERROR: i32 = 2;
const INVALID_ARG_LENGTH: i32 = 3;
const INVALID_SUB_COMMAND: i32 = 4;
const APPLICATION_ERR: i32 = 5;

fn main() {
    let _ = dotenv().ok();
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_all()
        .build()
        .expect("failed to build Runtime")
        .block_on(main_internal())
}

async fn main_internal() {
    let database_url = var(KEY_TO_DATABASE_URL).unwrap_or_else(|e| {
        println!(
            "failed to ge environment variable ({}): {}",
            KEY_TO_DATABASE_URL, e
        );
        exit(ENV_VAR_CAPTURE_FAILURE);
    });
    let conn = connect(&database_url).await.unwrap_or_else(|e| {
        println!(
            "failed to establish connection (database_url: {}): {}",
            database_url, e
        );
        exit(CONNECTION_ERROR);
    });

    run_command(&conn).await;
}

async fn connect(database_url: &str) -> Result<DatabaseConnection, Box<dyn Error + Send + Sync>> {
    let mut opt = ConnectOptions::new(database_url.to_string());
    opt.max_connections(1).min_connections(1).sqlx_logging(true);
    let conn = Database::connect(opt).await.map_err(Box::new)?;
    Ok(conn)
}

async fn run_command(conn: &DatabaseConnection) {
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
        if args.len() != 4 {
            println!(
                "usage: {} create \"admin_email_address\" \"password\"",
                args[0]
            );
            println!("ex: {} create admin@test.com 1234abcdABCD", args[0]);
            exit(INVALID_ARG_LENGTH);
        }
        match create_account(conn, &args[2], &args[3]).await {
            Ok(_) => exit(SUCCESS),
            Err(e) => {
                println!("application error: {}", e);
                exit(APPLICATION_ERR);
            }
        };
    } else if cmd == "list" {
        if args.len() != 2 {
            println!("usage: {} list", args[0]);
            println!("ex: {} list", args[0]);
            exit(INVALID_ARG_LENGTH);
        }
        match list_accounts(conn).await {
            Ok(admin_accounts) => {
                println!("email_address, mfa_enabled, last_login");
                admin_accounts.iter().for_each(|admin_account| {
                    println!(
                        "{}, {}, {:?}",
                        admin_account.email_address,
                        admin_account.mfa_enabled_at.is_some(),
                        admin_account
                            .last_login_time
                            .map(|m| { m.with_timezone(&*JAPANESE_TIME_ZONE) })
                    );
                });
                exit(SUCCESS)
            }
            Err(e) => {
                println!("application error: {}", e);
                exit(APPLICATION_ERR);
            }
        };
    } else if cmd == "update" {
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
    } else if cmd == "delete" {
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
    } else if cmd == "mfa" {
        if args.len() != 4 {
            println!(
                "usage: {} mfa [ enable | disable ] \"admin_email_address\"",
                args[0]
            );
            println!("ex: {} mfa enable admin@test.com", args[0]);
            exit(INVALID_ARG_LENGTH);
        }
        if args[2] == "enable" {
            todo!("enable mfa account");
        } else if args[2] == "disable" {
            todo!("disable mfa account");
        } else {
            println!(
                "usage: {} mfa [ enable | disable ] \"admin_email_address\"",
                args[0]
            );
            println!("ex: {} mfa enable admin@test.com", args[0]);
            exit(INVALID_ARG_LENGTH);
        }
    } else {
        println!("invalid subcommand: {}", cmd);
        println!("valid subcommand [ create | list | update | delete | mfa ]");
        exit(INVALID_SUB_COMMAND);
    }
}

async fn create_account(
    conn: &DatabaseConnection,
    email_addr: &str,
    password: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    validate_email_address(email_addr)?;
    validate_password(password)?;
    let hashed_pwd = hash_password(password)?;
    let admin_account = entity::admin_account::ActiveModel {
        email_address: Set(email_addr.to_string()),
        hashed_password: Set(hashed_pwd),
        ..Default::default()
    };
    admin_account.insert(conn).await.map_err(Box::new)?;
    Ok(())
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

// async fn enable_mfa(
//     conn: &DatabaseConnection,
//     email_addr: &str,
//     current_date_time: &DateTime<FixedOffset>,
// ) -> Result<(), Box<dyn Error + Send + Sync>> {
//     validate_email_address(email_addr)?;
//     let model = get_admin_account_by_email_address(conn, email_addr).await?;
//     let admin_account_id = model.admin_account_id;
//     conn.transaction::<_, (), TxErr>(|txn| {
//         Box::pin(async move {
//             let admin_account_model = select_admin_account_for_update(txn, admin_account_id)
//                 .await
//                 .map_err(|e| TxErr(e))?;

//             let admin_mfa_info_active_model = entity::admin_mfa_info::ActiveModel {
//                 admin_account_id: Set(admin_account_id),
//                 base32_encoded_secret: todo!(),
//             };

//             Ok(())
//         })
//     })
//     .await
//     .map_err(|e| match e {
//         TransactionError::Connection(db_err) => Box::new(db_err),
//         TransactionError::Transaction(tx_err) => tx_err.0,
//     })?;
//     Ok(())
// }

// async fn select_admin_account_for_update(
//     conn: &DatabaseTransaction,
//     admin_account_id: i64,
// ) -> Result<Option<entity::admin_account::Model>, Box<dyn Error + Send + Sync>> {
//     let model = entity::admin_account::Entity::find_by_id(admin_account_id)
//         .lock_exclusive()
//         .one(conn)
//         .await
//         .map_err(Box::new)?;
//     Ok(model)
// }

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
