// Copyright 2021 Ken Miura

pub(crate) mod credential;
pub(crate) mod error;

use diesel::r2d2::ConnectionManager;
use diesel::r2d2::Pool;
use diesel::PgConnection;

pub(crate) type ConnectionPool = Pool<ConnectionManager<PgConnection>>;

// TODO: 環境変数、もしくは他の設定から読み込むように変更する
const DOMAIN: &str = "localhost";
const PORT: u16 = 8080;
