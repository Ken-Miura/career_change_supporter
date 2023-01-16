// Copyright 2023 Ken Miura

use std::env;

use once_cell::sync::Lazy;

pub(crate) mod consultant_side_consultation;
pub(crate) mod user_side_consultation;

pub(crate) const KEY_TO_SKY_WAY_SECRET_KEY: &str = "SKY_WAY_SECRET_KEY";
/// SkyWayのPeer生成に使うcredentialを生成する際に利用するキー
pub(crate) static SKY_WAY_SECRET_KEY: Lazy<String> = Lazy::new(|| {
    let sky_way_secret_key = env::var(KEY_TO_SKY_WAY_SECRET_KEY).unwrap_or_else(|_| {
        panic!(
            "Not environment variable found: environment variable \"{}\" must be set",
            KEY_TO_SKY_WAY_SECRET_KEY
        )
    });
    sky_way_secret_key
});
