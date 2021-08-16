// Copyright 2021 Ken Miura

// TODO: #[macro_use]なしでdieselのマクロが使えるように変更が入った際に取り除く
// https://github.com/diesel-rs/diesel/issues/1764
#[macro_use]
extern crate diesel;

pub mod model;
pub mod schema;
pub mod credential;