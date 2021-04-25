// Copyright 2021 Ken Miura

mod account;
mod authentication;
mod model;
mod profile;
mod static_asset;

use actix_web::web;

pub(super) fn user_config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/user")
            .service(crate::user::static_asset::temporary_accounts)
            .service(crate::user::account::temporary_account_creation)
            .service(crate::user::account::account_creation)
            .service(crate::user::authentication::login_request)
            .service(crate::user::authentication::logout_request)
            .service(crate::user::authentication::session_state)
            .service(crate::user::profile::profile_information)
            // NOTE: 下記のrefに従い、"/"は最後に記載する
            // ref: https://docs.rs/actix-files/0.5.0/actix_files/struct.Files.html#implementation-notes
            .service(
                actix_files::Files::new(
                    "/",
                    crate::user::static_asset::USER_ASSETS_DIR.to_string(),
                )
                .prefer_utf8(true)
                .index_file("user_app.html")
                .default_handler(web::route().to(crate::user::static_asset::serve_user_app)),
            ),
    );
}
