// Copyright 2021 Ken Miura
use crate::common::error;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use std::fmt;

pub(crate) type ConnectionPool = Pool<ConnectionManager<PgConnection>>;

pub(crate) fn get(
    pool: &ConnectionPool,
) -> Result<
    r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
    DatabaseError,
> {
    let result = pool.get()?;
    Ok(result)
}

#[derive(Debug)]
pub(crate) enum DatabaseError {
    R2d2Error { code: u32, error: r2d2::Error },
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DatabaseError::R2d2Error { code, error } => {
                write!(
                    f,
                    "failed to get connection from pool (error code: {}): {}",
                    code, error
                )
            }
        }
    }
}

impl From<r2d2::Error> for DatabaseError {
    fn from(error: r2d2::Error) -> Self {
        DatabaseError::R2d2Error {
            code: error::code::DB_CONNECTION_UNAVAILABLE,
            error,
        }
    }
}

impl error::Detail for DatabaseError {
    fn code(&self) -> u32 {
        match self {
            DatabaseError::R2d2Error { code, error: _ } => *code,
        }
    }
    fn ui_message(&self) -> String {
        match self {
            DatabaseError::R2d2Error { code: _, error: _ } => {
                String::from("サーバでエラーが発生しました。一定時間後、再度お試しください。")
            }
        }
    }
}
