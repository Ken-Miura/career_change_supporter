// Copyright 2021 Ken Miura

pub(crate) mod credential;
pub(crate) mod error;

use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

pub(crate) type ConnectionPool = Pool<ConnectionManager<PgConnection>>;

impl error::ToCode for r2d2::Error {
    fn to_code(&self) -> u32 {
        error::code::INTERNAL_SERVER_ERROR
    }
}

impl error::ToMessage for r2d2::Error {
    fn to_message(&self) -> String {
        String::from(error::INTERNAL_SERVER_ERROR_MESSAGE)
    }
}
