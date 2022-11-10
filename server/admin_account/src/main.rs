// Copyright 2021 Ken Miura

use common::util::hash_password;
use common::util::validator::{
    email_address_validator::validate_email_address, password_validator::validate_password,
};
use dotenv::dotenv;
use entity::admin_account;
use entity::prelude::AdminAccount;
use entity::sea_orm::sea_query::Expr;
use entity::sea_orm::{
    ActiveModelTrait, ColumnTrait, Database, DatabaseConnection, EntityTrait, QueryFilter, Set,
    Value,
};
use std::fmt;
use std::{env::args, env::var, error::Error, process::exit};
use tokio::runtime::Runtime;

const KEY_TO_DATABASE_URL: &str = "DB_URL_FOR_ADMIN_APP";

const SUCCESS: i32 = 0;
const ENV_VAR_CAPTURE_FAILURE: i32 = 1;
const CONNECTION_ERROR: i32 = 2;
const INVALID_ARG_LENGTH: i32 = 3;
const INVALID_SUB_COMMAND: i32 = 4;
const APPLICATION_ERR: i32 = 5;

fn main() {
    let _ = dotenv().ok();
    let database_url = var(KEY_TO_DATABASE_URL).unwrap_or_else(|e| {
        println!(
            "failed to ge environment variable ({}): {}",
            KEY_TO_DATABASE_URL, e
        );
        exit(ENV_VAR_CAPTURE_FAILURE);
    });
    let client = connect(&database_url).unwrap_or_else(|e| {
        println!("failed to establish connection: {}", e);
        exit(CONNECTION_ERROR);
    });

    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        println!(
            "usage: {} [ create | list | update | delete ] [SUB_COMMAND_ARGS...]",
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
        match client.create_account(&args[2], &args[3]) {
            Ok(_) => exit(SUCCESS),
            Err(e) => {
                println!("application error: {}", e);
                exit(APPLICATION_ERR);
            }
        }
    } else if cmd == "list" {
        if args.len() != 2 {
            println!("usage: {} list", args[0]);
            println!("ex: {} list", args[0]);
            exit(INVALID_ARG_LENGTH);
        }
        match client.list_accounts() {
            Ok(email_addrs) => {
                email_addrs.iter().for_each(|email_addr| {
                    println!("{}", email_addr);
                });
                exit(SUCCESS);
            }
            Err(e) => {
                println!("application error: {}", e);
                exit(APPLICATION_ERR);
            }
        }
    } else if cmd == "update" {
        if args.len() != 4 {
            println!(
                "usage: {} update \"admin_email_address\" \"password\"",
                args[0]
            );
            println!("ex: {} update admin@test.com 1234abcdABCD", args[0]);
            exit(INVALID_ARG_LENGTH);
        }
        match client.update_account(&args[2], &args[3]) {
            Ok(_) => exit(SUCCESS),
            Err(e) => {
                println!("application error: {}", e);
                exit(APPLICATION_ERR);
            }
        }
    } else if cmd == "delete" {
        if args.len() != 3 {
            println!("usage: {} delete \"admin_email_address\"", args[0]);
            println!("ex: {} delete admin@test.com", args[0]);
            exit(INVALID_ARG_LENGTH);
        }
        match client.delete_account(&args[2]) {
            Ok(_) => exit(SUCCESS),
            Err(e) => {
                println!("application error: {}", e);
                exit(APPLICATION_ERR);
            }
        }
    } else {
        println!("invalid subcommand: {}", cmd);
        println!("valid subcommand [ create | list | update | delete ]");
        exit(INVALID_SUB_COMMAND);
    }
}

fn connect(db_url: &str) -> Result<AccountOperationClient, Box<dyn Error>> {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let conn = rt.block_on(async { Database::connect(db_url).await })?;
    Ok(AccountOperationClient { rt, conn })
}

// 下記のasyncを必要としないプログラムにasyncを利用する際の書き方を参考に実装
// https://tokio.rs/tokio/topics/bridging
struct AccountOperationClient {
    rt: Runtime,
    conn: DatabaseConnection,
}

impl AccountOperationClient {
    fn create_account(&self, email_addr: &str, password: &str) -> Result<(), Box<dyn Error>> {
        validate_email_address(email_addr)?;
        validate_password(password)?;
        let hashed_pwd = hash_password(password)?;
        let account = admin_account::ActiveModel {
            email_address: Set(email_addr.to_string()),
            hashed_password: Set(hashed_pwd),
            last_login_time: Set(None),
            ..Default::default()
        };
        let _ = self
            .rt
            .block_on(async { account.insert(&self.conn).await })?;
        Ok(())
    }

    fn list_accounts(&self) -> Result<Vec<String>, Box<dyn Error>> {
        let results = self
            .rt
            .block_on(async { AdminAccount::find().all(&self.conn).await })?;
        Ok(results
            .iter()
            .map(|model| model.email_address.to_string())
            .collect())
    }

    fn update_account(&self, email_addr: &str, password: &str) -> Result<(), Box<dyn Error>> {
        validate_email_address(email_addr)?;
        validate_password(password)?;
        let hashed_pwd = hash_password(password)?;
        let result = self.rt.block_on(async {
            admin_account::Entity::update_many()
                .col_expr(
                    admin_account::Column::HashedPassword,
                    Expr::value(Value::Bytes(Some(Box::new(hashed_pwd)))),
                )
                .filter(admin_account::Column::EmailAddress.eq(email_addr))
                .exec(&self.conn)
                .await
        })?;
        let num = result.rows_affected;
        if num == 0 {
            Err(Box::new(NoAccountFoundError(email_addr.to_string())))
        } else if num == 1 {
            Ok(())
        } else {
            panic!(
                "multiple admin accounts found! (email: {}, num: {})",
                email_addr, num
            );
        }
    }

    fn delete_account(&self, email_addr: &str) -> Result<(), Box<dyn Error>> {
        validate_email_address(email_addr)?;
        let result = self.rt.block_on(async {
            admin_account::Entity::delete_many()
                .filter(admin_account::Column::EmailAddress.eq(email_addr))
                .exec(&self.conn)
                .await
        })?;
        let num = result.rows_affected;
        if num == 0 {
            Err(Box::new(NoAccountFoundError(email_addr.to_string())))
        } else if num == 1 {
            Ok(())
        } else {
            panic!(
                "multiple admin accounts found! (email: {}, num: {})",
                email_addr, num
            );
        }
    }
}

#[derive(Debug, Clone)]
struct NoAccountFoundError(String);

impl fmt::Display for NoAccountFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "no email address ({}) found", self.0)
    }
}

impl Error for NoAccountFoundError {}
