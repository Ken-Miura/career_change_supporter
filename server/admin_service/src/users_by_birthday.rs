// Copyright 2021 Ken Miura

use axum::async_trait;
use axum::{
    extract::{Extension, Query},
    http::StatusCode,
    Json,
};
use chrono::NaiveDate;
use common::{ApiError, ErrResp, RespResult};
use entity::sea_orm::DatabaseConnection;
use serde::{Deserialize, Serialize};

use crate::err::unexpected_err_resp;
use crate::err::Code::IllegalDate;
use crate::util::session::Admin;

pub(crate) async fn get_users_by_birthday(
    Admin { account_id: _ }: Admin, // 認証されていることを保証するために必須のパラメータ
    query: Query<Birthday>,
    Extension(pool): Extension<DatabaseConnection>,
) -> RespResult<Vec<User>> {
    let query = query.0;
    let op = UsersByBirthdayOperationImpl { pool };
    get_users_by_birthday_internal(query.year, query.month, query.day, op).await
}

#[derive(Deserialize)]
pub(crate) struct Birthday {
    pub(crate) year: i32,
    pub(crate) month: u32,
    pub(crate) day: u32,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
pub(crate) struct User {}

async fn get_users_by_birthday_internal(
    year: i32,
    month: u32,
    day: u32,
    op: impl UsersByBirthdayOperation,
) -> RespResult<Vec<User>> {
    let birthday_option = NaiveDate::from_ymd_opt(year, month, day);
    let birthday = birthday_option.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError {
                code: IllegalDate as u32,
            }),
        )
    })?;
    let users = op.get_users_by_birthday(&birthday).await?;
    Ok((StatusCode::OK, Json(users)))
}

#[async_trait]
trait UsersByBirthdayOperation {
    async fn get_users_by_birthday(&self, birthday: &NaiveDate) -> Result<Vec<User>, ErrResp>;
}

struct UsersByBirthdayOperationImpl {
    pool: DatabaseConnection,
}

#[async_trait]
impl UsersByBirthdayOperation for UsersByBirthdayOperationImpl {
    async fn get_users_by_birthday(&self, birthday: &NaiveDate) -> Result<Vec<User>, ErrResp> {
        todo!()
    }
}

#[cfg(test)]
mod tests {}
