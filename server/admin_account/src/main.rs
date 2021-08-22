// Copyright 2021 Ken Miura

use common::util::validator::{
    validate_email_address, validate_password, EmailAddressValidationError, PasswordValidationError,
};
use diesel::connection::Connection;
use diesel::pg::PgConnection;
use diesel::ConnectionError;
use std::fmt::Display;
use std::{env::args, env::var, error::Error, process::exit};

const KEY_TO_DATABASE_URL: &str = "DB_URL_FOR_ADMIN_ACCOUNT_APP";

const SUCCESS: i32 = 0;
const NO_ENV_VAR_FOUND: i32 = 1;
const INVALID_ARG_LENGTH: i32 = 2;
const INVALID_SUB_COMMAND: i32 = 3;
const APPLICATION_ERR: i32 = 4;

fn main() {
    // check and get db url
    let result = var(KEY_TO_DATABASE_URL);
    if let Err(_) = result {
        println!(
            "environment variable \"{}\" must be set",
            KEY_TO_DATABASE_URL
        );
        exit(NO_ENV_VAR_FOUND);
    }
    let database_url = result.expect(&format!("failed to get value of {}", KEY_TO_DATABASE_URL));

    // get connection
    let result = establish_connection(database_url);
    if let Err(e) = result {
        println!("application error: {}", e);
        exit(APPLICATION_ERR);
    }
    let conn = result.expect("failed to get Connection");

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
        let result = create_admin_account(&args[2], &args[3], conn);
        match result {
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
    } else if cmd == "update" {
        if args.len() != 4 {
            println!(
                "usage: {} update \"admin_email_address\" \"password\"",
                args[0]
            );
            println!("ex: {} update admin@test.com 1234abcdABCD", args[0]);
            exit(INVALID_ARG_LENGTH);
        }
    } else if cmd == "delete" {
        if args.len() != 3 {
            println!("usage: {} delete \"admin_email_address\"", args[0]);
            println!("ex: {} delete admin@test.com", args[0]);
            exit(INVALID_ARG_LENGTH);
        }
    } else {
        println!("invalid subcommand: {}", cmd);
        println!("valid subcommand [ create | list | update | delete ]");
        exit(INVALID_SUB_COMMAND);
    }
}

fn establish_connection(database_url: String) -> Result<impl Connection, ApplicationError> {
    let result = PgConnection::establish(&database_url);
    match result {
        Ok(conn) => Ok(conn),
        Err(e) => Err(ApplicationError::ConnectionErr(e)),
    }
}

fn create_admin_account(
    email_address: &str,
    password: &str,
    connection: impl Connection,
) -> Result<(), ApplicationError> {
    let _ = validate_email_address(email_address)?;
    let _ = validate_password(password)?;
    Ok(())
}

#[derive(Debug)]
enum ApplicationError {
    ConnectionErr(ConnectionError),
    EmailAddrErr(EmailAddressValidationError),
    PasswordErr(PasswordValidationError),
}

impl Display for ApplicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApplicationError::ConnectionErr(e) => {
                write!(f, "failed to establish connection: {}", e)
            }
            ApplicationError::EmailAddrErr(e) => write!(f, "email address error: {}", e),
            ApplicationError::PasswordErr(e) => write!(f, "password error: {}", e),
        }
    }
}

impl Error for ApplicationError {}

impl From<EmailAddressValidationError> for ApplicationError {
    fn from(e: EmailAddressValidationError) -> Self {
        ApplicationError::EmailAddrErr(e)
    }
}

impl From<PasswordValidationError> for ApplicationError {
    fn from(e: PasswordValidationError) -> Self {
        ApplicationError::PasswordErr(e)
    }
}
