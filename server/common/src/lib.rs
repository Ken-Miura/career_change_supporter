// Copyright 2021 Ken Miura

// TODO: #[macro_use]なしでdieselのマクロが使えるように変更が入った際に取り除く
// https://github.com/diesel-rs/diesel/issues/1764
#[macro_use]
extern crate diesel;

pub mod credential;
pub mod model;
pub mod schema;
pub mod util;

use axum::{
    async_trait,
    extract::{Extension, FromRequest, RequestParts},
    http::StatusCode,
};

use diesel::{
    r2d2::{ConnectionManager, Pool, PooledConnection},
    PgConnection,
};

pub type ConnectionPool = Pool<ConnectionManager<PgConnection>>;

pub struct DatabaseConnection(pub PooledConnection<ConnectionManager<PgConnection>>);

#[async_trait]
impl<B> FromRequest<B> for DatabaseConnection
where
    B: Send,
{
    type Rejection = StatusCode;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Extension(pool) = Extension::<ConnectionPool>::from_request(req)
            .await
            .map_err(|e| {
                tracing::error!("failed to extract connection pool from req: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        let conn = pool.get().map_err(|e| {
            tracing::error!("failed to get connection from pool: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
        Ok(Self(conn))
    }
}
