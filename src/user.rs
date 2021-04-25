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
            .service(crate::user::static_asset::user_app)
            .service(crate::user::static_asset::js)
            .service(crate::user::static_asset::css)
            .service(crate::user::static_asset::img)
            .service(crate::user::static_asset::temporary_accounts)
            .service(crate::user::account::temporary_account_creation)
            .service(crate::user::account::account_creation)
            .service(crate::user::authentication::login_request)
            .service(crate::user::authentication::logout_request)
            .service(crate::user::authentication::session_state)
            .service(crate::user::profile::profile_information)
            .default_service(web::route().to(crate::user::static_asset::serve_user_app)),
    );
}
