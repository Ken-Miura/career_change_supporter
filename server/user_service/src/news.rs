// Copyright 2023 Ken Miura

use axum::extract::State;
use common::RespResult;
use entity::sea_orm::DatabaseConnection;
use serde::Serialize;

pub(crate) async fn get_news(State(pool): State<DatabaseConnection>) -> RespResult<Vec<News>> {
    todo!()
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct News {
    title: String,
    body: String,
    published_date_in_jst: PublishedDate,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub(crate) struct PublishedDate {
    year: i32,
    month: u32,
    day: u32,
}
