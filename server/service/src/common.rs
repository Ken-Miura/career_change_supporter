// Copyright 2021 Ken Miura

pub(crate) mod credential;
pub(crate) mod error;
pub(crate) mod util;

use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;
use once_cell::sync::Lazy;
use std::env;

pub(crate) static PAYJP_TEST_SECRET_KEY: Lazy<String> =
    Lazy::new(|| env::var("PAYJP_TEST_SECRET_KEY").expect("PAYJP_TEST_SECRET_KEY must be set"));

pub(crate) static PAYJP_TEST_PASSWORD: Lazy<String> =
    Lazy::new(|| env::var("PAYJP_TEST_PASSWORD").expect("PAYJP_TEST_PASSWORD must be set"));

pub(crate) const PACKAGE_NAME: &str = env!("CARGO_PKG_NAME");

pub(crate) type ConnectionPool = Pool<ConnectionManager<PgConnection>>;

// TODO: 環境変数、もしくは他の設定から読み込むように変更する
pub(crate) const DOMAIN: &str = "localhost";
pub(crate) const PORT: u16 = 8080;
pub(crate) const SMTP_SERVER_ADDR: ([u8; 4], u16) = ([127, 0, 0, 1], 1025);
pub(crate) const CACHE_SERVER_ADDR: &str = "127.0.0.1:6379";
