// Copyright 2021 Ken Miura

use derive_more::Display;

#[derive(Display, Debug)]
pub(crate) enum Error {
    R2d2Err(r2d2::Error),
}

// NOTE: Use negative value because positive value is used for handled error
pub(super) const INTERNAL_SERVER_ERROR: i32 = -1;
