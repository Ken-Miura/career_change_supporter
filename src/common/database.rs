// Copyright 2021 Ken Miura
use crate::common::error;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

pub(crate) type ConnectionPool = Pool<ConnectionManager<PgConnection>>;

impl error::Detail for r2d2::Error {
    fn code(&self) -> u32 {
        error::code::DB_CONNECTION_UNAVAILABLE
    }
    fn ui_message(&self) -> String {
        String::from("サーバでエラーが発生しました。一定時間後、再度お試しください。")
    }
}
