// Copyright 2021 Ken Miura

use axum::{body::Body, http::Request};

pub(crate) async fn get_refresh(_req: Request<Body>) {}
